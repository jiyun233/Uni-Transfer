import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"

/** 单个 FTP 用户 */
export interface FtpUser {
  username: string
  password: string
  root_dir: string
  read_only: boolean
  enabled: boolean
}

/** FTP 服务器配置 */
export interface FtpConfig {
  ip: string
  port: number
  root_dir: string
  anonymous: boolean
  anonymous_read_only: boolean
  users: FtpUser[]
}

/** FTP 服务器运行状态 */
export interface FtpStatus {
  running: boolean
  config: FtpConfig | null
  address: string | null
}

/** 创建默认用户 */
export function defaultUser(): FtpUser {
  return {
    username: "",
    password: "",
    root_dir: "",
    read_only: false,
    enabled: true,
  }
}

/** 默认配置 */
export function defaultFtpConfig(): FtpConfig {
  return {
    ip: "0.0.0.0",
    port: 2121,
    root_dir: "",
    anonymous: true,
    anonymous_read_only: false,
    users: [],
  }
}

export function useFtpServer() {
  const isRunning = ref(false)
  const config = ref<FtpConfig>(defaultFtpConfig())
  const address = ref<string | null>(null)
  const statusMessage = ref("")

  /** 获取当前状态 */
  async function fetchStatus() {
    try {
      const status = await invoke<FtpStatus>("get_ftp_status")
      isRunning.value = status.running
      address.value = status.address
      if (status.config) {
        config.value = status.config
      }
      statusMessage.value = status.running
        ? `运行中 — ftp://${status.address}`
        : "已停止"
    } catch (error) {
      console.error("获取 FTP 状态失败:", error)
      statusMessage.value = "状态获取失败"
    }
  }

  /** 启动 FTP 服务器 */
  async function start() {
    try {
      const msg = await invoke<string>("start_ftp", {
        config: {
          ip: config.value.ip,
          port: config.value.port,
          root_dir: config.value.root_dir,
          anonymous: config.value.anonymous,
          anonymous_read_only: config.value.anonymous_read_only,
          users: config.value.users,
        },
      })
      statusMessage.value = msg
      await fetchStatus()
    } catch (error) {
      statusMessage.value = `启动失败: ${error}`
      console.error("启动 FTP 服务器失败:", error)
      throw error
    }
  }

  /** 停止 FTP 服务器 */
  async function stop() {
    try {
      const msg = await invoke<string>("stop_ftp")
      statusMessage.value = msg
      await fetchStatus()
    } catch (error) {
      statusMessage.value = `停止失败: ${error}`
      console.error("停止 FTP 服务器失败:", error)
      throw error
    }
  }

  // ---- 用户管理 ----

  function addUser() {
    config.value.users.push(defaultUser())
  }

  function removeUser(index: number) {
    config.value.users.splice(index, 1)
  }

  return {
    isRunning,
    config,
    address,
    statusMessage,
    fetchStatus,
    start,
    stop,
    addUser,
    removeUser,
  }
}
