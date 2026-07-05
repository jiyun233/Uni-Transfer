<script setup lang="ts">
import { ref, onMounted, onUnmounted, inject } from "vue"
import { NIcon, NText, NTag, NScrollbar } from "naive-ui"
import { PhonePortraitOutline, DesktopOutline, PersonOutline } from "@vicons/ionicons5"
import { invoke } from "@tauri-apps/api/core"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"

defineProps<{ size: number }>()

// 从父组件注入选中会话的 setter
const selectSession = inject<(s: ActiveSession | null) => void>("selectSession", () => {})

interface ActiveSession {
  session_id: string
  username: string
  ip: string
  connected_at: string
  is_anonymous: boolean
}

const sessions = ref<ActiveSession[]>([])
const selectedId = ref<string | null>(null)

async function fetchSessions() {
  try {
    sessions.value = await invoke<ActiveSession[]>("get_active_sessions")
  } catch (e) {
    console.error("获取会话列表失败:", e)
  }
}

function onSelect(s: ActiveSession) {
  selectedId.value = s.session_id === selectedId.value ? null : s.session_id
  selectSession(selectedId.value ? s : null)
}

let unlisten: UnlistenFn | undefined

onMounted(async () => {
  await fetchSessions()
  listen<ActiveSession[]>("sessions-changed", (event) => {
    sessions.value = event.payload
    // 如果当前选中的会话已断开，清除选择
    if (selectedId.value && !event.payload.find(s => s.session_id === selectedId.value)) {
      selectedId.value = null
      selectSession(null)
    }
  }).then(fn => { unlisten = fn })
})

onUnmounted(() => { unlisten?.() })
</script>

<template>
  <div class="device-list">
    <div class="panel-header">
      <NIcon :size="20" :component="PhonePortraitOutline" />
      <NText strong style="font-size: 15px">设备列表</NText>
    </div>

    <div class="device-stats">
      <div class="stat-item">
        <NText depth="3" style="font-size: 11px">在线</NText>
        <NText strong style="font-size: 20px; color: var(--n-color-target)">{{ sessions.length }}</NText>
      </div>
      <div class="stat-item">
        <NText depth="3" style="font-size: 11px">匿名</NText>
        <NText strong style="font-size: 20px">{{ sessions.filter(s => s.is_anonymous).length }}</NText>
      </div>
      <div class="stat-item">
        <NText depth="3" style="font-size: 11px">认证用户</NText>
        <NText strong style="font-size: 20px">{{ sessions.filter(s => !s.is_anonymous).length }}</NText>
      </div>
    </div>

    <NScrollbar style="flex: 1">
      <div class="session-list">
        <div v-if="sessions.length === 0" class="empty-sessions">
          <NIcon :size="36" :component="DesktopOutline" color="var(--n-text-color-3)" />
          <NText depth="3" style="font-size: 13px">暂无连接会话</NText>
          <NText depth="3" style="font-size: 11px">启动 FTP 服务器后，客户端连接将显示在这里</NText>
        </div>

        <div
          v-for="s in sessions"
          :key="s.session_id"
          class="session-item"
          :class="{ 'session-item--selected': selectedId === s.session_id }"
          @click="onSelect(s)"
        >
          <NIcon
            :size="20"
            :component="s.is_anonymous ? PersonOutline : DesktopOutline"
            :color="selectedId === s.session_id ? 'var(--n-color-target)' : 'var(--n-text-color-2)'"
          />
          <div class="session-info">
            <div class="session-name-row">
              <NText strong style="font-size: 13px">{{ s.username }}</NText>
              <NTag :type="s.is_anonymous ? 'default' : 'info'" size="tiny" :bordered="false">
                {{ s.is_anonymous ? '匿名' : '用户' }}
              </NTag>
            </div>
            <NText depth="3" style="font-size: 11px; font-family: monospace">{{ s.ip }}</NText>
          </div>
          <div class="session-meta">
            <NText depth="3" style="font-size: 10px; font-family: monospace">{{ s.connected_at }}</NText>
            <NText depth="3" style="font-size: 10px; font-family: monospace; opacity: 0.6">{{ s.session_id }}</NText>
          </div>
        </div>
      </div>
    </NScrollbar>
  </div>
</template>

<style scoped>
.device-list {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 16px;
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 14px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--n-border-color);
  flex-shrink: 0;
}

.device-stats {
  display: flex;
  justify-content: space-around;
  padding: 6px 8px;
  margin-bottom: 14px;
  border-radius: 8px;
  background: var(--n-color-embedded);
  flex-shrink: 0;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

/* ---- 会话列表 ---- */
.session-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.empty-sessions {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 32px 16px;
  text-align: center;
}

.session-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 8px;
  background: var(--n-color-embedded);
  cursor: pointer;
  transition: background 0.15s, box-shadow 0.15s;
}

.session-item:hover {
  background: var(--n-color-hover);
}

.session-item--selected {
  background: var(--n-color-target-09, rgba(74, 144, 217, 0.09));
  box-shadow: inset 0 0 0 1px var(--n-color-target);
}

.session-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.session-name-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.session-meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
  flex-shrink: 0;
}
</style>
