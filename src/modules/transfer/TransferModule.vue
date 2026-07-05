<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue"
import {
  NIcon, NText, NTag, NScrollbar, NButton, NDivider, NSpace,
} from "naive-ui"
import {
  SwapHorizontalOutline,
  CloudDownloadOutline,
  CloudUploadOutline,
  DocumentOutline,
  TrashOutline,
  TimeOutline,
  FolderOpenOutline,
  PersonOutline,
} from "@vicons/ionicons5"
import { invoke } from "@tauri-apps/api/core"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"
import { ResizableSplitPane } from "../../components/resizable-split-pane"

interface TransferRecord {
  id: number
  session_id: string
  username: string
  filename: string
  path: string
  size: number
  direction: string
  status: string  // "transferring" | "done" | "failed"
  timestamp: string
}

const tasks = ref<TransferRecord[]>([])
const selectedTask = ref<TransferRecord | null>(null)

// 统计只计算已完成的
const completedTasks = computed(() => tasks.value.filter(t => t.status === "done"))


function formatSize(bytes: number): string {
  if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(2)} GB`
  if (bytes >= 1e6) return `${(bytes / 1e6).toFixed(1)} MB`
  if (bytes >= 1e3) return `${(bytes / 1e3).toFixed(0)} KB`
  return `${bytes} B`
}

const totalUpload = computed(() =>
  completedTasks.value.filter(t => t.direction === "upload").reduce((s, t) => s + t.size, 0)
)
const totalDownload = computed(() =>
  completedTasks.value.filter(t => t.direction === "download").reduce((s, t) => s + t.size, 0)
)
const activeCount = computed(() =>
  tasks.value.filter(t => t.status === "transferring").length
)

async function fetchHistory() {
  try {
    tasks.value = await invoke<TransferRecord[]>("get_transfer_history")
  } catch (e) {
    console.error("获取传输历史失败:", e)
  }
}

async function clearHistory() {
  try {
    await invoke("clear_transfer_history")
    tasks.value = []
    selectedTask.value = null
  } catch (e) {
    console.error("清除历史失败:", e)
  }
}

let unlisten: UnlistenFn | undefined

onMounted(async () => {
  await fetchHistory()
  listen<TransferRecord>("transfers-changed", (event) => {
    const rec = event.payload
    // 查找是否已有同 ID 的记录（从 transferring → done 更新）
    const existing = tasks.value.findIndex(t => t.id === rec.id)
    if (existing >= 0) {
      // 原地更新（状态、大小、时间）
      tasks.value[existing] = { ...tasks.value[existing], ...rec }
    } else {
      tasks.value.unshift(rec)
      if (tasks.value.length > 500) tasks.value.pop()
    }
    // 如果当前选中的是这个任务，也更新选中引用
    if (selectedTask.value?.id === rec.id) {
      selectedTask.value = { ...selectedTask.value, ...rec }
    }
  }).then(fn => { unlisten = fn })
})

onUnmounted(() => { unlisten?.() })

function selectTask(t: TransferRecord) {
  selectedTask.value = selectedTask.value?.id === t.id ? null : t
}
</script>

<template>
  <div class="transfer-module">
    <ResizableSplitPane
      :initial-size="42"
      :min-size="28"
      :max-size="65"
      :divider-size="6"
      :allow-reset="true"
    >
      <!-- 左侧：任务列表 -->
      <template #primary>
        <div class="panel panel-left">
          <div class="panel-header">
            <NIcon :size="18" :component="SwapHorizontalOutline" color="var(--n-color-target)" />
            <NText strong style="font-size: 14px">传输任务</NText>
            <NTag size="tiny" :bordered="false">{{ tasks.length }}</NTag>
            <div style="flex:1" />
            <NButton size="tiny" quaternary @click="clearHistory">
              <template #icon><NIcon :component="TrashOutline" /></template>
              清空
            </NButton>
          </div>

          <!-- 汇总 -->
          <div class="transfer-summary">
            <div class="summary-chip upload">
              <NIcon :size="14" :component="CloudUploadOutline" />
              <NText depth="3" style="font-size: 11px">上传</NText>
              <NText strong style="font-size: 12px">{{ formatSize(totalUpload) }}</NText>
            </div>
            <div class="summary-chip download">
              <NIcon :size="14" :component="CloudDownloadOutline" />
              <NText depth="3" style="font-size: 11px">下载</NText>
              <NText strong style="font-size: 12px">{{ formatSize(totalDownload) }}</NText>
            </div>
            <div v-if="activeCount > 0" class="summary-chip active">
              <NIcon :size="14" :component="SwapHorizontalOutline" />
              <NText depth="3" style="font-size: 11px">进行中</NText>
              <NText strong style="font-size: 12px; color: var(--n-color-target)">{{ activeCount }}</NText>
            </div>
          </div>

          <NScrollbar style="flex: 1">
            <div class="task-list">
              <div v-if="tasks.length === 0" class="empty-tasks">
                <NIcon :size="40" :component="SwapHorizontalOutline" color="var(--n-text-color-3)" />
                <NText depth="3" style="font-size: 13px">暂无传输记录</NText>
                <NText depth="3" style="font-size: 11px">FTP 文件传输记录将自动显示在这里</NText>
              </div>

              <div
                v-for="t in tasks"
                :key="t.id"
                class="task-item"
                :class="{
                  'task-item--selected': selectedTask?.id === t.id,
                  'task-item--active': t.status === 'transferring',
                }"
                @click="selectTask(t)"
              >
                <div class="task-icon" :class="{ 'task-icon--spinning': t.status === 'transferring' }">
                  <NIcon
                    :size="20"
                    :component="t.direction === 'upload' ? CloudUploadOutline : CloudDownloadOutline"
                    :color="t.direction === 'upload' ? '#e67e22' : 'var(--n-color-target)'"
                  />
                </div>
                <div class="task-info">
                  <NText strong style="font-size: 12px; word-break: break-all">{{ t.filename }}</NText>
                  <div class="task-meta-line">
                    <NText depth="3" style="font-size: 11px">
                      {{ t.direction === 'upload' ? '上传' : '下载' }}
                    </NText>
                    <NText depth="3" style="font-size: 11px">·</NText>
                    <NText depth="3" style="font-size: 11px">{{ t.username }}</NText>
                    <NText v-if="t.status === 'done'" depth="3" style="font-size: 11px">·</NText>
                    <NText v-if="t.status === 'done'" depth="3" style="font-size: 11px">{{ formatSize(t.size) }}</NText>
                  </div>
                </div>
                <div class="task-right">
                  <NTag
                    :type="t.status === 'transferring' ? 'info' : t.status === 'done' ? 'success' : 'error'"
                    size="tiny"
                    :bordered="false"
                  >
                    {{ t.status === 'transferring' ? '传输中' : t.status === 'done' ? '已完成' : '失败' }}
                  </NTag>
                  <NText depth="3" style="font-size: 10px; font-family: monospace">
                    {{ t.timestamp }}
                  </NText>
                </div>
              </div>
            </div>
          </NScrollbar>
        </div>
      </template>

      <!-- 右侧：任务详情 -->
      <template #secondary>
        <div class="panel panel-right">
          <div class="panel-header">
            <NIcon :size="18" :component="DocumentOutline" color="var(--n-text-color-2)" />
            <NText strong style="font-size: 14px">任务详情</NText>
          </div>

          <NScrollbar v-if="selectedTask" style="flex: 1">
            <div class="detail-body">
              <div class="detail-hero">
                <div class="hero-dir-icon" :class="{ 'hero-dir-icon--active': selectedTask.status === 'transferring' }">
                  <NIcon
                    :size="24"
                    :component="selectedTask.direction === 'upload' ? CloudUploadOutline : CloudDownloadOutline"
                    :color="selectedTask.direction === 'upload' ? '#e67e22' : 'var(--n-color-target)'"
                  />
                </div>
                <div class="hero-text">
                  <NText strong style="font-size: 16px; word-break: break-all">{{ selectedTask.filename }}</NText>
                  <div style="display: flex; gap: 6px; flex-wrap: wrap">
                    <NTag
                      :type="selectedTask.direction === 'upload' ? 'warning' : 'info'"
                      size="small"
                      :bordered="false"
                    >
                      {{ selectedTask.direction === 'upload' ? '上传' : '下载' }}
                    </NTag>
                    <NTag
                      :type="selectedTask.status === 'transferring' ? 'info' : selectedTask.status === 'done' ? 'success' : 'error'"
                      size="small"
                      :bordered="false"
                    >
                      {{ selectedTask.status === 'transferring' ? '传输中...' : selectedTask.status === 'done' ? '已完成' : '失败' }}
                    </NTag>
                  </div>
                </div>
              </div>

              <NDivider style="margin: 14px 0" />

              <NSpace vertical :size="10">
                <div class="detail-row">
                  <NIcon :size="16" :component="PersonOutline" color="var(--n-text-color-2)" />
                  <NText depth="3" style="font-size: 11px; min-width: 50px">用户</NText>
                  <NText strong style="font-size: 13px">{{ selectedTask.username }}</NText>
                </div>
                <div class="detail-row">
                  <NIcon :size="16" :component="DocumentOutline" color="var(--n-text-color-2)" />
                  <NText depth="3" style="font-size: 11px; min-width: 50px">大小</NText>
                  <NText strong style="font-size: 13px">
                    {{ selectedTask.status === 'done' ? formatSize(selectedTask.size) : '传输中...' }}
                  </NText>
                </div>
                <div class="detail-row">
                  <NIcon :size="16" :component="TimeOutline" color="var(--n-text-color-2)" />
                  <NText depth="3" style="font-size: 11px; min-width: 50px">时间</NText>
                  <NText strong style="font-size: 13px; font-family: monospace">{{ selectedTask.timestamp }}</NText>
                </div>
                <div class="detail-row">
                  <NIcon :size="16" :component="FolderOpenOutline" color="var(--n-text-color-2)" />
                  <NText depth="3" style="font-size: 11px; min-width: 50px">路径</NText>
                  <NText strong style="font-size: 12px; word-break: break-all; font-family: monospace">
                    {{ selectedTask.path }}
                  </NText>
                </div>
              </NSpace>
            </div>
          </NScrollbar>

          <NScrollbar v-else style="flex: 1">
            <div class="detail-empty">
              <NIcon :size="40" :component="DocumentOutline" color="var(--n-text-color-3)" />
              <NText depth="3" style="font-size: 13px">选择左侧任务查看详情</NText>
            </div>
          </NScrollbar>
        </div>
      </template>
    </ResizableSplitPane>
  </div>
</template>

<style scoped>
.transfer-module {
  width: 100%;
  height: 100%;
  overflow: hidden;
}

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

/* ---- 传输汇总 ---- */
.transfer-summary {
  display: flex;
  gap: 8px;
  padding: 8px 14px;
  border-bottom: 1px solid var(--n-border-color);
  flex-shrink: 0;
}

.summary-chip {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 6px;
  background: var(--n-color-embedded);
}

/* ---- 任务列表 ---- */
.task-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 10px;
}

.empty-tasks {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 40px 16px;
  text-align: center;
}

.task-item {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 8px 10px;
  border-radius: 6px;
  background: var(--n-color-embedded);
  cursor: pointer;
  transition: background 0.15s;
}

.task-item:hover {
  background: var(--n-color-hover);
}

.task-item--selected {
  background: var(--n-color-target-09, rgba(74, 144, 217, 0.09));
  box-shadow: inset 0 0 0 1px var(--n-color-target);
}

.task-item--active {
  background: rgba(74, 144, 217, 0.06);
}

.task-icon {
  flex-shrink: 0;
}

.task-icon--spinning {
  animation: transfer-spin 1.5s linear infinite;
}

@keyframes transfer-spin {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}

.task-right {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 3px;
  flex-shrink: 0;
}

.task-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.task-meta-line {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
}

/* ---- 详情 ---- */
.panel-right { }

.detail-body {
  padding: 14px;
}

.detail-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 48px 24px;
  text-align: center;
}

.detail-hero {
  display: flex;
  align-items: center;
  gap: 12px;
}

.hero-dir-icon {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--n-color-embedded);
}

.hero-dir-icon--active {
  animation: hero-pulse 1.2s ease-in-out infinite;
}

@keyframes hero-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.hero-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.detail-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 6px;
  background: var(--n-color-embedded);
}
</style>
