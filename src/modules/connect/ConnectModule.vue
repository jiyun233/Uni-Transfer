<script setup lang="ts">
import {
  NText,
  NSpace,
  NButton,
  NDivider,
  NInput,
  NInputNumber,
  NSelect,
  NSwitch,
  NTag,
  NIcon,
  NCollapse,
  NCollapseItem,
  NGrid,
  NGi,
  NScrollbar,
  useMessage,
} from "naive-ui"
import {
  ServerOutline,
  GlobeOutline,
  AddCircleOutline,
  TrashOutline,
  EyeOffOutline,
  FolderOpenOutline,
  PeopleOutline,
  DocumentTextOutline,
  CheckmarkCircleOutline,
  CloseCircleOutline,
  InformationCircleOutline,
} from "@vicons/ionicons5"
import { ref, onMounted, onUnmounted, computed, nextTick } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"
import { open } from "@tauri-apps/plugin-dialog"
import { ResizableSplitPane } from "../../components/resizable-split-pane"
import { useFtpServer } from "./useFtpServer"

const ips = ref<Record<string, string>>({})

let message: ReturnType<typeof useMessage>
try {
  message = useMessage()
} catch {
  message = {
    success: (msg: string) => console.log("[FTP]", msg),
    error: (msg: string) => console.error("[FTP]", msg),
  } as ReturnType<typeof useMessage>
}

const {
  isRunning: ftpRunning,
  config: ftpConfig,
  statusMessage: ftpStatusMessage,
  fetchStatus: fetchFtpStatus,
  start: startFtp,
  stop: stopFtp,
  addUser,
  removeUser,
} = useFtpServer()

const ipCount = computed(() => Object.keys(ips.value).length)

const ipOptions = computed(() => {
  const options: Array<{ label: string; value: string }> = [
    { label: "0.0.0.0（所有接口）", value: "0.0.0.0" },
  ]
  for (const [name, ip] of Object.entries(ips.value)) {
    options.push({ label: `${name} — ${ip}`, value: ip })
  }
  return options
})

const userCount = computed(() => ftpConfig.value.users.length)

// ---- 连接日志 ----
interface LogEntry {
  time: string
  text: string
  type: "info" | "success" | "error"
}

const logs = ref<LogEntry[]>([])
const logScrollRef = ref<InstanceType<typeof NScrollbar> | null>(null)

function addLog(text: string, type: LogEntry["type"] = "info") {
  const now = new Date()
  const time = `${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}:${String(now.getSeconds()).padStart(2, "0")}`
  logs.value.push({ time, text, type })
  // 限制日志数量
  if (logs.value.length > 200) {
    logs.value.shift()
  }
  // 滚动到底部
  nextTick(() => {
    logScrollRef.value?.scrollTo({ top: 999999, behavior: "smooth" })
  })
}

const logIcon = (type: LogEntry["type"]) => {
  switch (type) {
    case "success": return CheckmarkCircleOutline
    case "error":   return CloseCircleOutline
    default:        return InformationCircleOutline
  }
}

// ---- 业务逻辑 ----

async function getAllIPv4Addresses() {
  try {
    const result = await invoke<Record<string, string>>("get_all_ipv4")
    if (result && typeof result === "object") {
      ips.value = result
      if (
        !ftpConfig.value.ip ||
        (!result[ftpConfig.value.ip] && ftpConfig.value.ip !== "0.0.0.0")
      ) {
        const ipValues = Object.values(result)
        if (ipValues.length > 0) {
          ftpConfig.value.ip = ipValues[0]
        }
      }
    }
    addLog(`检测到 ${Object.keys(result || {}).length} 个网络接口`)
  } catch (error) {
    addLog(`获取 IP 地址失败: ${error}`, "error")
  }
}

async function toggleFtp() {
  if (ftpRunning.value) {
    try {
      await stopFtp()
      message?.success?.("FTP 服务器已停止")
    } catch (e) {
      addLog(`停止失败: ${e}`, "error")
      message?.error?.(String(e))
    }
  } else {
    try {
      await startFtp()
      message?.success?.("FTP 服务器启动成功")
    } catch (e) {
      addLog(`启动失败: ${e}`, "error")
      message?.error?.(String(e))
    }
  }
}

async function pickFolder(target: { root_dir: string }) {
  try {
    const selected = await open({ directory: true, multiple: false })
    if (selected && typeof selected === "string") {
      target.root_dir = selected
      addLog(`已选择目录: ${selected}`)
    }
  } catch (e) {
    addLog(`目录选择失败: ${e}`, "error")
  }
}

onMounted(async () => {
  await getAllIPv4Addresses()
  await fetchFtpStatus()
  if (ftpRunning.value) {
    addLog("检测到 FTP 服务器正在运行", "success")
  }

  // 监听 Rust 后端直接推送的 FTP 日志事件（start/stop 等）
  let unlisten: UnlistenFn | undefined
  listen<{ type: string; message: string; detail?: string }>("ftp-log", (event) => {
    const { type, message, detail } = event.payload
    addLog(detail ? `${message}（${detail}）` : message, type as LogEntry["type"])
  }).then((fn) => {
    unlisten = fn
  })

  // 定时轮询认证器 / 监听器写入的缓冲日志
  const pollTimer = setInterval(async () => {
    try {
      const entries = await invoke<Array<{ type: string; message: string; detail?: string }>>("drain_ftp_logs")
      for (const entry of entries) {
        addLog(
          entry.detail ? `${entry.message}（${entry.detail}）` : entry.message,
          entry.type as LogEntry["type"]
        )
      }
    } catch {
      // 轮询失败静默忽略
    }
  }, 1000)

  // 组件卸载时取消监听和轮询
  onUnmounted(() => {
    unlisten?.()
    clearInterval(pollTimer)
  })
})
</script>

<template>
  <div class="connect-module">
    <!-- ========== IP 地址横条 ========== -->
    <div class="ip-bar">
      <div class="ip-bar-header">
        <NIcon :size="18" :component="GlobeOutline" color="var(--n-color-target)" />
        <NText depth="3" style="font-size: 12px">本机 IPv4</NText>
        <NTag :type="ipCount > 0 ? 'success' : 'error'" size="tiny" :bordered="false">
          {{ ipCount }}
        </NTag>
      </div>
      <div v-if="ipCount > 0" class="ip-bar-list">
        <div v-for="(ip, name) in ips" :key="name" class="ip-chip">
          <NText depth="3" style="font-size: 11px">{{ name }}</NText>
          <NText strong style="font-size: 13px; color: var(--n-color-target)">{{ ip }}</NText>
        </div>
      </div>
      <NText v-else depth="3" style="font-size: 12px; color: #e74c3c">
        未检测到可用网络
      </NText>
    </div>

    <!-- ========== 左右分栏 ========== -->
    <div class="split-wrapper">
      <ResizableSplitPane
        :initial-size="38"
        :min-size="28"
        :max-size="55"
        :divider-size="6"
        :allow-reset="true"
      >
        <!-- ======== 左侧：FTP 服务器设置 ======== -->
        <template #primary>
          <div class="panel panel-left">
            <div class="panel-header">
              <NIcon :size="18" :component="ServerOutline" color="var(--n-color-target)" />
              <NText strong style="font-size: 14px">FTP 服务器</NText>
              <NTag :type="ftpRunning ? 'success' : 'default'" size="tiny" :bordered="false">
                {{ ftpRunning ? '运行中' : '已停止' }}
              </NTag>
            </div>

            <NScrollbar style="flex: 1">
              <div class="panel-body">
                <NSpace vertical :size="12">
                  <div>
                    <NText depth="3" style="font-size: 11px">绑定地址</NText>
                    <NSelect
                      v-model:value="ftpConfig.ip"
                      :options="ipOptions"
                      :disabled="ftpRunning"
                      size="small"
                      style="margin-top: 2px"
                    />
                  </div>
                  <div>
                    <NText depth="3" style="font-size: 11px">端口号</NText>
                    <NInputNumber
                      v-model:value="ftpConfig.port"
                      :min="1024"
                      :max="65535"
                      :step="1"
                      :disabled="ftpRunning"
                      size="small"
                      style="margin-top: 2px; width: 100%"
                    />
                  </div>
                  <div>
                    <NText depth="3" style="font-size: 11px">根目录</NText>
                    <div class="dir-picker" style="margin-top: 2px">
                      <NInput
                        v-model:value="ftpConfig.root_dir"
                        placeholder="留空 = 默认目录"
                        :disabled="ftpRunning"
                        size="small"
                        style="flex: 1"
                      />
                      <NButton size="small" quaternary :disabled="ftpRunning" @click="pickFolder(ftpConfig)">
                        <template #icon>
                          <NIcon :size="16" :component="FolderOpenOutline" />
                        </template>
                      </NButton>
                    </div>
                  </div>

                  <NDivider style="margin: 4px 0" />

                  <div class="toggle-row">
                    <NText depth="3" style="font-size: 12px">匿名访问</NText>
                    <NSwitch v-model:value="ftpConfig.anonymous" :disabled="ftpRunning" size="small" />
                  </div>
                  <div class="toggle-row">
                    <NText depth="3" style="font-size: 12px">匿名只读</NText>
                    <NSwitch v-model:value="ftpConfig.anonymous_read_only" :disabled="ftpRunning || !ftpConfig.anonymous" size="small" />
                  </div>

                  <NDivider style="margin: 4px 0" />

                  <NText v-if="ftpStatusMessage" depth="2" style="font-size: 12px">
                    {{ ftpStatusMessage }}
                  </NText>

                  <NButton
                    :type="ftpRunning ? 'error' : 'primary'"
                    @click="toggleFtp"
                    block
                    size="small"
                  >
                    {{ ftpRunning ? '停止服务器' : '启动服务器' }}
                  </NButton>
                </NSpace>
              </div>
            </NScrollbar>
          </div>
        </template>

        <!-- ======== 右侧：用户管理 + 连接日志 ======== -->
        <template #secondary>
          <div class="panel panel-right">
            <!-- ---- 用户管理 ---- -->
            <div class="right-top">
              <div class="panel-header">
                <NIcon :size="18" :component="PeopleOutline" color="var(--n-color-target)" />
                <NText strong style="font-size: 14px">用户管理</NText>
                <NTag size="tiny" :bordered="false">{{ userCount }}</NTag>
                <div style="flex: 1" />
                <NButton size="tiny" quaternary :disabled="ftpRunning" @click="addUser">
                  <template #icon>
                    <NIcon :component="AddCircleOutline" />
                  </template>
                  添加
                </NButton>
              </div>

              <NScrollbar style="flex: 1">
                <div class="panel-body">
                  <div v-if="userCount === 0" class="no-users-hint">
                    <NIcon :size="28" :component="EyeOffOutline" color="var(--n-text-color-3)" />
                    <NText depth="3" style="font-size: 12px">未配置用户，仅使用匿名访问</NText>
                  </div>

                  <NCollapse v-else>
                    <NCollapseItem
                      v-for="(user, index) in ftpConfig.users"
                      :key="index"
                      :name="String(index)"
                    >
                      <template #header>
                        <div class="user-collapse-header">
                          <NText strong style="font-size: 13px">{{ user.username || '新用户' }}</NText>
                          <NTag :type="user.enabled ? 'success' : 'default'" size="tiny" :bordered="false">
                            {{ user.enabled ? '启用' : '禁用' }}
                          </NTag>
                        </div>
                      </template>

                      <NSpace vertical :size="8">
                        <NGrid :cols="2" :x-gap="10" :y-gap="0">
                          <NGi>
                            <NText depth="3" style="font-size: 11px">用户名</NText>
                            <NInput v-model:value="user.username" placeholder="用户名" size="small" :disabled="ftpRunning" />
                          </NGi>
                          <NGi>
                            <NText depth="3" style="font-size: 11px">密码</NText>
                            <NInput v-model:value="user.password" placeholder="留空 = 无密码" type="password" show-password-on="click" size="small" :disabled="ftpRunning" />
                          </NGi>
                        </NGrid>

                        <div>
                          <NText depth="3" style="font-size: 11px">用户目录</NText>
                          <div class="dir-picker" style="margin-top: 2px">
                            <NInput v-model:value="user.root_dir" placeholder="留空 = 根目录/用户名" size="small" :disabled="ftpRunning" style="flex: 1" />
                            <NButton size="small" quaternary :disabled="ftpRunning" @click="pickFolder(user)">
                              <template #icon>
                                <NIcon :size="16" :component="FolderOpenOutline" />
                              </template>
                            </NButton>
                          </div>
                        </div>

                        <div class="user-toggles">
                          <div class="toggle-row">
                            <NText depth="3" style="font-size: 12px">只读</NText>
                            <NSwitch v-model:value="user.read_only" :disabled="ftpRunning" size="small" />
                          </div>
                          <div class="toggle-row">
                            <NText depth="3" style="font-size: 12px">启用</NText>
                            <NSwitch v-model:value="user.enabled" :disabled="ftpRunning" size="small" />
                          </div>
                          <NButton size="tiny" type="error" quaternary :disabled="ftpRunning" @click.stop="removeUser(index)">
                            <template #icon>
                              <NIcon :component="TrashOutline" />
                            </template>
                            删除
                          </NButton>
                        </div>
                      </NSpace>
                    </NCollapseItem>
                  </NCollapse>
                </div>
              </NScrollbar>
            </div>

            <!-- ---- 连接日志 ---- -->
            <div class="right-bottom">
              <div class="panel-header panel-header--compact">
                <NIcon :size="16" :component="DocumentTextOutline" color="var(--n-text-color-2)" />
                <NText depth="2" style="font-size: 13px">连接日志</NText>
                <NTag size="tiny" :bordered="false">{{ logs.length }}</NTag>
              </div>

              <NScrollbar ref="logScrollRef" style="flex: 1">
                <div class="log-list">
                  <div v-if="logs.length === 0" class="log-empty">
                    <NText depth="3" style="font-size: 11px">暂无日志</NText>
                  </div>
                  <div
                    v-for="(entry, i) in logs"
                    :key="i"
                    class="log-entry"
                  >
                    <NIcon
                      :size="14"
                      :component="logIcon(entry.type)"
                      :color="entry.type === 'success' ? '#2ecc71' : entry.type === 'error' ? '#e74c3c' : 'var(--n-text-color-3)'"
                    />
                    <NText depth="3" style="font-size: 10px; flex-shrink: 0; font-family: monospace">{{ entry.time }}</NText>
                    <NText depth="2" style="font-size: 11px">{{ entry.text }}</NText>
                  </div>
                </div>
              </NScrollbar>
            </div>
          </div>
        </template>
      </ResizableSplitPane>
    </div>
  </div>
</template>

<style scoped>
.connect-module {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 0;
}

/* ---- IP 横条 ---- */
.ip-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 16px;
  background: var(--n-color-embedded);
  flex-shrink: 0;
  flex-wrap: wrap;
  border-bottom: 1px solid var(--n-border-color);
}

.ip-bar-header {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.ip-bar-list {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.ip-chip {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 2px 8px;
  border-radius: 4px;
  background: var(--n-color);
}

/* ---- 分栏容器 ---- */
.split-wrapper {
  flex: 1;
  min-height: 0;
}

/* ---- 面板通用 ---- */
.panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--n-border-color);
  flex-shrink: 0;
}

.panel-header--compact {
  padding: 6px 12px;
}

.panel-body {
  padding: 12px 14px;
}

/* ---- 左侧面板 ---- */
.panel-left {
  border-right: none;
}

/* ---- 右侧面板 ---- */
.panel-right {
  display: flex;
  flex-direction: column;
}

.right-top {
  flex: 1 1 55%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-bottom: 1px solid var(--n-border-color);
}

.right-bottom {
  flex: 1 1 45%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 100px;
}

/* ---- 目录选择器 ---- */
.dir-picker {
  display: flex;
  align-items: center;
  gap: 2px;
}

/* ---- 开关 ---- */
.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

/* ---- 用户 ---- */
.user-collapse-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.no-users-hint {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 16px 0;
}

.user-toggles {
  display: flex;
  align-items: center;
  gap: 14px;
}

/* ---- 日志 ---- */
.log-list {
  padding: 4px 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.log-empty {
  text-align: center;
  padding: 20px 0;
}

.log-entry {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 6px;
  border-radius: 4px;
  transition: background 0.15s;
}

.log-entry:hover {
  background: var(--n-color-embedded);
}
</style>
