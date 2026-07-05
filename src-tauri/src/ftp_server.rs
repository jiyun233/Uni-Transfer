use std::collections::HashMap;
use std::fmt::Debug;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio::io::AsyncRead;
use unftp_core::auth::{
    AuthenticationError, Authenticator, Credentials, Principal, UserDetail, UserDetailError,
    UserDetailProvider,
};
use unftp_core::storage::{Fileinfo, Metadata as StorageMetadata, StorageBackend};

// ============================================================
// 配置数据结构
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpUser {
    pub username: String,
    pub password: String,
    pub root_dir: String,
    pub read_only: bool,
    pub enabled: bool,
}

impl Default for FtpUser {
    fn default() -> Self {
        Self { username: String::new(), password: String::new(), root_dir: String::new(), read_only: false, enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpConfig {
    pub ip: String,
    pub port: u16,
    pub root_dir: String,
    pub anonymous: bool,
    pub anonymous_read_only: bool,
    pub users: Vec<FtpUser>,
}

impl Default for FtpConfig {
    fn default() -> Self {
        Self {
            ip: "0.0.0.0".to_string(), port: 2121,
            root_dir: default_root_dir(), anonymous: true,
            anonymous_read_only: false, users: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpStatus {
    pub running: bool,
    pub config: Option<FtpConfig>,
    pub address: Option<String>,
}

fn default_root_dir() -> String {
    #[cfg(target_os = "windows")]
    { std::env::var("USERPROFILE").map(|p| format!("{}\\UniTransfer", p)).unwrap_or_else(|_| ".".to_string()) }
    #[cfg(not(target_os = "windows"))]
    { std::env::var("HOME").map(|p| format!("{}/UniTransfer", p)).unwrap_or_else(|_| ".".to_string()) }
}

// ============================================================
// 日志事件
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct FtpLogEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub message: String,
    pub detail: Option<String>,
}

impl FtpLogEvent {
    fn info(msg: impl Into<String>) -> Self {
        Self { event_type: "info".into(), message: msg.into(), detail: None }
    }
    fn success(msg: impl Into<String>) -> Self {
        Self { event_type: "success".into(), message: msg.into(), detail: None }
    }
    fn error(msg: impl Into<String>) -> Self {
        Self { event_type: "error".into(), message: msg.into(), detail: None }
    }
}

static LOG_BUFFER: Mutex<Vec<FtpLogEvent>> = Mutex::new(Vec::new());

fn push_log(event: FtpLogEvent) {
    if let Ok(mut buf) = LOG_BUFFER.lock() {
        eprintln!("[FTP] {} - {}", event.event_type, event.message);
        buf.push(event);
    }
}

#[tauri::command]
pub fn drain_ftp_logs() -> Vec<FtpLogEvent> {
    let mut buf = LOG_BUFFER.lock().unwrap_or_else(|e| e.into_inner());
    std::mem::take(&mut *buf)
}

// ============================================================
// 活跃会话 & 传输历史 共享状态
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSession {
    pub session_id: String,
    pub username: String,
    pub ip: String,
    pub connected_at: String,
    pub is_anonymous: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRecord {
    pub id: u64,
    pub session_id: String,
    pub username: String,
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub direction: String, // "upload" | "download"
    pub status: String,    // "transferring" | "done" | "failed"
    pub timestamp: String,
}

static ACTIVE_SESSIONS: Mutex<Vec<ActiveSession>> = Mutex::new(Vec::new());
static TRANSFER_HISTORY: Mutex<Vec<TransferRecord>> = Mutex::new(Vec::new());
static NEXT_TRANSFER_ID: Mutex<u64> = Mutex::new(0);

// 认证时记录的 IP（trace_id → ip），供 PresenceListener 查找
static RECENT_IPS: LazyLock<Mutex<HashMap<String, (String, Instant)>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn record_auth_ip(username: &str, ip: &str) {
    if let Ok(mut map) = RECENT_IPS.lock() {
        map.insert(username.to_string(), (ip.to_string(), Instant::now()));
        // 清理 30s 以上的旧记录
        map.retain(|_, (_, t)| t.elapsed().as_secs() < 30);
    }
}

fn take_auth_ip(username: &str) -> String {
    if let Ok(mut map) = RECENT_IPS.lock() {
        map.remove(username).map(|(ip, _)| ip).unwrap_or_else(|| "?".to_string())
    } else {
        "?".to_string()
    }
}

fn add_session(sid: &str, username: &str, is_anon: bool) {
    let now = chrono_now();
    let ip = take_auth_ip(username);
    if let Ok(mut sessions) = ACTIVE_SESSIONS.lock() {
        sessions.push(ActiveSession {
            session_id: sid.to_string(),
            username: username.to_string(),
            ip,
            connected_at: now,
            is_anonymous: is_anon,
        });
        push_log(FtpLogEvent::success(format!(
            "会话创建: user='{}', session={}, ip={}", username, sid,
            sessions.last().map(|s| s.ip.as_str()).unwrap_or("?")
        )));
    }
}

fn remove_session(sid: &str) {
    if let Ok(mut sessions) = ACTIVE_SESSIONS.lock() {
        if let Some(pos) = sessions.iter().position(|s| s.session_id == sid) {
            let removed = sessions.remove(pos);
            push_log(FtpLogEvent::info(format!(
                "会话结束: user='{}', session={}, ip={}", removed.username, removed.session_id, removed.ip
            )));
        }
    }
}

fn add_transfer_start(username: &str, direction: &str, path: &str) -> TransferRecord {
    let id = {
        let mut next = NEXT_TRANSFER_ID.lock().unwrap_or_else(|e| e.into_inner());
        let id = *next;
        *next += 1;
        id
    };
    let filename = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    let record = TransferRecord {
        id,
        session_id: String::new(),
        username: username.to_string(),
        filename,
        path: path.to_string(),
        size: 0,
        direction: direction.to_string(),
        status: "transferring".to_string(),
        timestamp: chrono_now(),
    };
    if let Ok(mut history) = TRANSFER_HISTORY.lock() {
        history.push(record.clone());
        if history.len() > 500 { history.remove(0); }
    }
    record
}

fn complete_transfer(username: &str, direction: &str, path: &str, size: u64) -> TransferRecord {
    if let Ok(mut history) = TRANSFER_HISTORY.lock() {
        // 找到匹配的进行中传输并更新
        for rec in history.iter_mut().rev() {
            if rec.username == username && rec.direction == direction && rec.path == path && rec.status == "transferring" {
                rec.size = size;
                rec.status = "done".to_string();
                rec.timestamp = chrono_now();
                let updated = rec.clone();
                return updated;
            }
        }
    }
    // 没找到 pending 记录，创建新的
    let id = {
        let mut next = NEXT_TRANSFER_ID.lock().unwrap_or_else(|e| e.into_inner());
        let id = *next;
        *next += 1;
        id
    };
    let filename = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    let record = TransferRecord {
        id,
        session_id: String::new(),
        username: username.to_string(),
        filename,
        path: path.to_string(),
        size,
        direction: direction.to_string(),
        status: "done".to_string(),
        timestamp: chrono_now(),
    };
    if let Ok(mut history) = TRANSFER_HISTORY.lock() {
        history.push(record.clone());
        if history.len() > 500 { history.remove(0); }
    }
    record
}

fn chrono_now() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = now.as_secs();
        // 转换为本地时间字符串
        let offset = 8 * 3600; // UTC+8 for China, simplified
        let total = secs + offset;
        let h = (total / 3600) % 24;
        let m = (total / 60) % 60;
        let s = total % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    }
    #[cfg(target_arch = "wasm32")]
    { String::from("--:--:--") }
}

#[tauri::command]
pub fn get_active_sessions() -> Vec<ActiveSession> {
    ACTIVE_SESSIONS.lock()
        .map(|s| s.clone())
        .unwrap_or_else(|e| e.into_inner().clone())
}

#[tauri::command]
pub fn get_transfer_history() -> Vec<TransferRecord> {
    TRANSFER_HISTORY.lock()
        .map(|h| h.clone())
        .unwrap_or_else(|e| e.into_inner().clone())
}

#[tauri::command]
pub fn clear_transfer_history() {
    if let Ok(mut h) = TRANSFER_HISTORY.lock() { h.clear(); }
}

// ============================================================
// 认证器
// ============================================================

#[derive(Debug)]
pub struct AppUserDetail {
    pub username: String,
    pub home_dir: Option<PathBuf>,
}

impl UserDetail for AppUserDetail {
    fn account_enabled(&self) -> bool { true }
    fn home(&self) -> Option<&Path> { self.home_dir.as_deref() }
}

impl std::fmt::Display for AppUserDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.username)
    }
}

#[derive(Debug, Clone)]
struct ConfigAuthenticator {
    users: Vec<FtpUser>,
    anonymous: bool,
}

#[async_trait]
impl Authenticator for ConfigAuthenticator {
    async fn authenticate(&self, username: &str, creds: &Credentials) -> Result<Principal, AuthenticationError> {
        let source = creds.source_ip;
        let has_password = creds.password.is_some();

        push_log(FtpLogEvent::info(format!(
            "认证请求: user='{}', password={}, ip={}", username, has_password, source
        )));

        let anon_names = ["anonymous", "ftp", ""];
        if self.anonymous && (anon_names.contains(&username.to_lowercase().as_str()) || username.to_lowercase().starts_with("anon")) {
            record_auth_ip("anonymous", &source.to_string());
            push_log(FtpLogEvent::success(format!("匿名用户认证通过 (ip: {})", source)));
            return Ok(Principal { username: "anonymous".to_string() });
        }

        if let Some(user) = self.users.iter().find(|u| u.username == username) {
            if !user.enabled {
                push_log(FtpLogEvent::error(format!("用户 '{}' 已被禁用 (ip: {})", username, source)));
                return Err(AuthenticationError::BadUser);
            }
            let password = creds.password.as_deref().unwrap_or("");
            if user.password == password {
                record_auth_ip(username, &source.to_string());
                push_log(FtpLogEvent::success(format!("用户 '{}' 认证通过 (ip: {})", username, source)));
                return Ok(Principal { username: username.to_string() });
            }
            push_log(FtpLogEvent::error(format!(
                "用户 '{}' 密码错误 (期望 '{}', 收到 '{}')", username, user.password, password
            )));
            return Err(AuthenticationError::BadPassword);
        }

        if self.anonymous {
            record_auth_ip("anonymous", &source.to_string());
            push_log(FtpLogEvent::success(format!("用户 '{}' 未匹配，以匿名放行", username)));
            return Ok(Principal { username: "anonymous".to_string() });
        }

        push_log(FtpLogEvent::error(format!("用户 '{}' 不存在，匿名已禁用", username)));
        Err(AuthenticationError::BadUser)
    }

    async fn cert_auth_sufficient(&self, _username: &str) -> bool { self.anonymous }
}

#[derive(Debug, Clone)]
struct ConfigUserDetailProvider {
    users: Vec<FtpUser>,
    base_root: PathBuf,
}

#[async_trait]
impl UserDetailProvider for ConfigUserDetailProvider {
    type User = AppUserDetail;

    async fn provide_user_detail(&self, principal: &Principal) -> Result<AppUserDetail, UserDetailError> {
        push_log(FtpLogEvent::info(format!("查询用户详情: '{}'", principal.username)));

        if principal.username == "anonymous" {
            return Ok(AppUserDetail { username: "anonymous".to_string(), home_dir: Some(self.base_root.clone()) });
        }

        if let Some(user) = self.users.iter().find(|u| u.username == principal.username) {
            let home = if user.root_dir.trim().is_empty() {
                self.base_root.join(&user.username)
            } else {
                let p = PathBuf::from(&user.root_dir);
                if p.is_absolute() { p } else { self.base_root.join(&p) }
            };
            if !home.exists() { std::fs::create_dir_all(&home).ok(); }
            let home_dir = if home == self.base_root { None } else { Some(home) };
            return Ok(AppUserDetail { username: user.username.clone(), home_dir });
        }

        Err(UserDetailError::UserNotFound { username: principal.username.clone() })
    }
}

// ============================================================
// FTP 事件监听器（管理会话 & 传输历史）
// ============================================================

use libunftp::notification::{DataEvent, DataListener, EventMeta, PresenceEvent, PresenceListener};

#[derive(Debug, Clone)]
struct FtpLogListener {
    app_handle: tauri::AppHandle,
}

#[async_trait]
impl DataListener for FtpLogListener {
    async fn receive_data_event(&self, event: DataEvent, meta: EventMeta) {
        match &event {
            DataEvent::Got { path, bytes } => {
                let rec = complete_transfer(&meta.username, "download", path, *bytes);
                push_log(FtpLogEvent::info(format!(
                    "📥 {} 下载 {} ({} bytes)", meta.username, rec.filename, bytes
                )));
                let _ = self.app_handle.emit("transfers-changed", &rec);
            }
            DataEvent::Put { path, bytes } => {
                let rec = complete_transfer(&meta.username, "upload", path, *bytes);
                push_log(FtpLogEvent::info(format!(
                    "📤 {} 上传 {} ({} bytes)", meta.username, rec.filename, bytes
                )));
                let _ = self.app_handle.emit("transfers-changed", &rec);
            }
            _ => {
                // 其他事件只记录日志
            }
        }
    }
}

#[async_trait]
impl PresenceListener for FtpLogListener {
    async fn receive_presence_event(&self, event: PresenceEvent, meta: EventMeta) {
        let sid = meta.trace_id[..8.min(meta.trace_id.len())].to_string();
        match event {
            PresenceEvent::LoggedIn => {
                let is_anon = meta.username == "anonymous";
                add_session(&sid, &meta.username, is_anon);
                let _ = self.app_handle.emit("sessions-changed", get_active_sessions());
            }
            PresenceEvent::LoggedOut => {
                remove_session(&sid);
                let _ = self.app_handle.emit("sessions-changed", get_active_sessions());
            }
        }
    }
}

// ============================================================
// StorageBackend 包装器 — 拦截 get/put 以追踪传输开始
// ============================================================

use unftp_sbe_fs::Filesystem;

struct TrackedFs {
    inner: Filesystem,
    app_handle: tauri::AppHandle,
}

impl std::fmt::Debug for TrackedFs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrackedFs").finish()
    }
}

#[async_trait]
impl<User: UserDetail> StorageBackend<User> for TrackedFs {
    type Metadata = unftp_sbe_fs::Meta;

    fn enter(&mut self, user_detail: &User) -> std::io::Result<()> {
        self.inner.enter(user_detail)
    }

    async fn metadata<P: AsRef<Path> + Send + Debug>(&self, user: &User, path: P) -> Result<Self::Metadata, unftp_core::storage::Error> {
        self.inner.metadata(user, path).await
    }

    async fn list<P: AsRef<Path> + Send + Debug>(&self, user: &User, path: P) -> Result<Vec<Fileinfo<PathBuf, Self::Metadata>>, unftp_core::storage::Error>
    where <Self as StorageBackend<User>>::Metadata: StorageMetadata
    {
        self.inner.list(user, path).await
    }

    async fn get<P: AsRef<Path> + Send + Debug>(&self, user: &User, path: P, start_pos: u64) -> Result<Box<dyn AsyncRead + Send + Sync + Unpin>, unftp_core::storage::Error> {
        let p = path.as_ref();
        let name = format!("{}", user);
        let rec = add_transfer_start(&name, "download", &p.to_string_lossy());
        push_log(FtpLogEvent::info(format!("⬇ 开始下载: {} (by {})", rec.filename, name)));
        let _ = self.app_handle.emit("transfers-changed", &rec);
        self.inner.get(user, path, start_pos).await
    }

    async fn put<P: AsRef<Path> + Send + Debug, R: AsyncRead + Send + Sync + Unpin + 'static>(
        &self, user: &User, input: R, path: P, start_pos: u64,
    ) -> Result<u64, unftp_core::storage::Error> {
        let p = path.as_ref();
        let name = format!("{}", user);
        let rec = add_transfer_start(&name, "upload", &p.to_string_lossy());
        push_log(FtpLogEvent::info(format!("⬆ 开始上传: {} (by {})", rec.filename, name)));
        let _ = self.app_handle.emit("transfers-changed", &rec);
        self.inner.put(user, input, path, start_pos).await
    }

    async fn del<P: AsRef<Path> + Send + Debug>(&self, user: &User, path: P) -> Result<(), unftp_core::storage::Error> {
        self.inner.del(user, path).await
    }

    async fn mkd<P: AsRef<Path> + Send + Debug>(&self, user: &User, path: P) -> Result<(), unftp_core::storage::Error> {
        self.inner.mkd(user, path).await
    }

    async fn rename<P: AsRef<Path> + Send + Debug>(&self, user: &User, from: P, to: P) -> Result<(), unftp_core::storage::Error> {
        self.inner.rename(user, from, to).await
    }

    async fn rmd<P: AsRef<Path> + Send + Debug>(&self, user: &User, path: P) -> Result<(), unftp_core::storage::Error> {
        self.inner.rmd(user, path).await
    }

    async fn cwd<P: AsRef<Path> + Send + Debug>(&self, user: &User, path: P) -> Result<(), unftp_core::storage::Error> {
        self.inner.cwd(user, path).await
    }
}

// ============================================================
// 服务器生命周期
// ============================================================

struct FtpServerState {
    handle: tokio::task::JoinHandle<()>,
    config: FtpConfig,
}

static FTP_STATE: Mutex<Option<FtpServerState>> = Mutex::new(None);

fn common_ancestor(paths: &[PathBuf]) -> PathBuf {
    if paths.is_empty() { return PathBuf::from("/"); }
    let mut ancestor = paths[0].clone();
    for p in &paths[1..] {
        while !p.starts_with(&ancestor) {
            if let Some(parent) = ancestor.parent() { ancestor = parent.to_path_buf(); }
            else { return PathBuf::from(if cfg!(windows) { "C:\\" } else { "/" }); }
        }
    }
    if ancestor.exists() && !ancestor.is_dir() {
        ancestor = ancestor.parent().map(Path::to_path_buf).unwrap_or(ancestor);
    }
    ancestor
}

#[tauri::command]
pub async fn start_ftp(app_handle: tauri::AppHandle, config: FtpConfig) -> Result<String, String> {
    // 清理上一次运行留下的状态
    if let Ok(mut s) = ACTIVE_SESSIONS.lock() { s.clear(); }
    if let Ok(mut h) = TRANSFER_HISTORY.lock() { h.clear(); }
    {
        let mut state = FTP_STATE.lock().map_err(|e| format!("锁获取失败: {}", e))?;
        if let Some(existing) = state.take() {
            existing.handle.abort();
            let _ = app_handle.emit("ftp-log", FtpLogEvent::info("已停止旧的服务器实例"));
        }
    }

    let server_root = if config.root_dir.trim().is_empty() { default_root_dir() } else { config.root_dir.clone() };
    let root_path = PathBuf::from(&server_root);

    push_log(FtpLogEvent::info(format!("初始化 - 根目录: {}", root_path.display())));

    if !root_path.exists() {
        std::fs::create_dir_all(&root_path).map_err(|e| format!("无法创建根目录: {}", e))?;
        push_log(FtpLogEvent::info("已创建根目录"));
    }
    if !root_path.is_dir() { return Err(format!("'{}' 不是一个有效的目录", server_root)); }

    let enabled_users: Vec<_> = config.users.iter().filter(|u| u.enabled).collect();
    push_log(FtpLogEvent::info(format!(
        "配置: {} 个用户, {} 启用, 匿名: {}",
        config.users.len(), enabled_users.len(),
        if config.anonymous { "是" } else { "否" }
    )));

    let mut all_paths: Vec<PathBuf> = vec![root_path.clone()];
    for user in &enabled_users {
        let user_home = if user.root_dir.trim().is_empty() {
            root_path.join(&user.username)
        } else {
            let p = PathBuf::from(&user.root_dir);
            if p.is_absolute() { p } else { root_path.join(&p) }
        };
        if !user_home.exists() { std::fs::create_dir_all(&user_home).ok(); }
        push_log(FtpLogEvent::info(format!(
            "用户 '{}': dir={}, read_only={}", user.username, user_home.display(), user.read_only
        )));
        all_paths.push(user_home);
    }

    let fs_root = common_ancestor(&all_paths);
    push_log(FtpLogEvent::info(format!("FS 根: {}", fs_root.display())));

    unftp_sbe_fs::Filesystem::new(fs_root.clone()).map_err(|e| {
        let msg = format!("文件系统失败: {}", e);
        push_log(FtpLogEvent::error(&msg));
        msg
    })?;

    let passive_ip: Option<Ipv4Addr> = if config.ip == "0.0.0.0" { None } else { config.ip.parse().ok() };

    let authenticator = ConfigAuthenticator { users: config.users.clone(), anonymous: config.anonymous };
    let user_detail_provider = ConfigUserDetailProvider { users: config.users.clone(), base_root: root_path.clone() };
    let log_listener = FtpLogListener { app_handle: app_handle.clone() };
    let fs_root_for_fs = fs_root;
    let ah_for_fs = app_handle.clone();

    let mut builder = libunftp::ServerBuilder::with_authenticator(
        Box::new(move || TrackedFs {
            inner: unftp_sbe_fs::Filesystem::new(fs_root_for_fs.clone()).expect("FS init fail"),
            app_handle: ah_for_fs.clone(),
        }),
        std::sync::Arc::new(authenticator),
    )
    .user_detail_provider(std::sync::Arc::new(user_detail_provider))
    .notify_data(log_listener.clone())
    .notify_presence(log_listener.clone())
    .greeting("Welcome to Uni-Transfer FTP Server")
    .passive_ports(50000..=65535);

    if let Some(ip) = passive_ip {
        builder = builder.passive_host(ip);
        push_log(FtpLogEvent::info(format!("被动模式 IP: {}", ip)));
    }

    let server = builder.build().map_err(|e| {
        let msg = format!("构建失败: {}", e);
        push_log(FtpLogEvent::error(&msg));
        msg
    })?;

    let bind_addr = format!("{}:{}", config.ip, config.port);
    let bind_display = bind_addr.clone();
    push_log(FtpLogEvent::info(format!("绑定 {} ...", bind_display)));

    let ah = app_handle.clone();
    let handle = tokio::spawn(async move {
        println!("[FTP] 启动: {}", bind_display);
        if let Err(e) = server.listen(bind_display).await {
            let _ = ah.emit("ftp-log", FtpLogEvent::error(format!("服务器错误: {}", e)));
            eprintln!("[FTP] 错误: {}", e);
        }
        // 停止时清理会话
        if let Ok(mut s) = ACTIVE_SESSIONS.lock() { s.clear(); }
        let _ = ah.emit("sessions-changed", Vec::<ActiveSession>::new());
    });

    {
        let mut state = FTP_STATE.lock().map_err(|e| format!("锁获取失败: {}", e))?;
        *state = Some(FtpServerState { handle, config });
    }

    let addr = format!("ftp://{}", bind_addr);
    push_log(FtpLogEvent::success(format!("启动成功 → {}", addr)));
    let _ = app_handle.emit("ftp-log", FtpLogEvent::success(format!("服务器就绪: {}", addr)));
    let _ = app_handle.emit("ftp-log", FtpLogEvent::info("被动端口 50000-65535，请确保防火墙放行"));

    Ok(format!("FTP 服务器已启动: {}", addr))
}

#[tauri::command]
pub async fn stop_ftp(app_handle: tauri::AppHandle) -> Result<String, String> {
    push_log(FtpLogEvent::info("正在停止..."));
    let mut state = FTP_STATE.lock().map_err(|e| format!("锁获取失败: {}", e))?;
    match state.take() {
        Some(existing) => {
            existing.handle.abort();
            if let Ok(mut s) = ACTIVE_SESSIONS.lock() { s.clear(); }
            let _ = app_handle.emit("sessions-changed", Vec::<ActiveSession>::new());
            push_log(FtpLogEvent::info("已停止"));
            let _ = app_handle.emit("ftp-log", FtpLogEvent::success("服务器已停止"));
            Ok("FTP 服务器已停止".to_string())
        }
        None => Err("FTP 服务器未在运行".to_string()),
    }
}

#[tauri::command]
pub async fn get_ftp_status() -> Result<FtpStatus, String> {
    let state = FTP_STATE.lock().map_err(|e| format!("锁获取失败: {}", e))?;
    match state.as_ref() {
        Some(s) => {
            let addr = format!("{}:{}", s.config.ip, s.config.port);
            Ok(FtpStatus { running: true, config: Some(s.config.clone()), address: Some(addr) })
        }
        None => Ok(FtpStatus { running: false, config: None, address: None }),
    }
}
