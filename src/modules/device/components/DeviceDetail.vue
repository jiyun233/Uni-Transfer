<script setup lang="ts">
import { inject, type Ref } from "vue"
import {
  NIcon, NText, NScrollbar, NTag, NDivider, NSpace, NEmpty,
} from "naive-ui"
import {
  InformationCircleOutline,
  PersonOutline,
  GlobeOutline,
  TimeOutline,
  KeyOutline,
} from "@vicons/ionicons5"

defineProps<{ size: number }>()

interface ActiveSession {
  session_id: string
  username: string
  ip: string
  connected_at: string
  is_anonymous: boolean
}

const selected = inject<Ref<ActiveSession | null>>("selectedSession")

function detailRows(s: ActiveSession) {
  return [
    { icon: PersonOutline, label: "用户名", value: s.username },
    { icon: KeyOutline, label: "认证方式", value: s.is_anonymous ? "匿名访问" : "密码认证" },
    { icon: GlobeOutline, label: "IP 地址", value: s.ip },
    { icon: TimeOutline, label: "连接时间", value: s.connected_at },
    { icon: InformationCircleOutline, label: "会话 ID", value: s.session_id },
  ]
}
</script>

<template>
  <div class="device-detail">
    <div class="panel-header">
      <NIcon :size="20" :component="InformationCircleOutline" />
      <NText strong style="font-size: 15px">设备详情</NText>
    </div>

    <NScrollbar v-if="selected" style="flex: 1">
      <div class="detail-body">
        <!-- 会话摘要 -->
        <div class="detail-hero">
          <div class="hero-avatar" :class="{ 'hero-avatar--anon': selected.is_anonymous }">
            <NIcon :size="28" :component="PersonOutline" />
          </div>
          <div class="hero-info">
            <NText strong style="font-size: 18px">{{ selected.username }}</NText>
            <NTag :type="selected.is_anonymous ? 'default' : 'info'" size="small" :bordered="false">
              {{ selected.is_anonymous ? '匿名用户' : '认证用户' }}
            </NTag>
          </div>
        </div>

        <NDivider style="margin: 16px 0" />

        <!-- 详细信息行 -->
        <NSpace vertical :size="12">
          <div v-for="row in detailRows(selected)" :key="row.label" class="detail-row">
            <div class="detail-row-icon">
              <NIcon :size="18" :component="row.icon" color="var(--n-text-color-2)" />
            </div>
            <div class="detail-row-text">
              <NText depth="3" style="font-size: 11px">{{ row.label }}</NText>
              <NText strong style="font-size: 13px; font-family: monospace">{{ row.value }}</NText>
            </div>
          </div>
        </NSpace>

        <NDivider style="margin: 16px 0" />

        <!-- 会话权限 -->
        <NText depth="3" style="font-size: 11px; text-transform: uppercase; letter-spacing: 1px">访问权限</NText>
        <div style="margin-top: 8px; display: flex; gap: 8px; flex-wrap: wrap">
          <NTag type="success" size="small" :bordered="false">读取</NTag>
          <NTag :type="selected.is_anonymous ? 'default' : 'success'" size="small" :bordered="false">
            {{ selected.is_anonymous ? '仅公共目录' : '私有目录' }}
          </NTag>
          <NTag type="success" size="small" :bordered="false">被动模式</NTag>
        </div>
      </div>
    </NScrollbar>

    <NScrollbar v-else style="flex: 1">
      <div class="detail-empty">
        <NEmpty description="选择左侧会话以查看详情" />
      </div>
    </NScrollbar>
  </div>
</template>

<style scoped>
.device-detail {
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

.detail-body {
  padding: 4px 0;
}

.detail-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
}

/* ---- Hero ---- */
.detail-hero {
  display: flex;
  align-items: center;
  gap: 14px;
}

.hero-avatar {
  width: 52px;
  height: 52px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--n-color-target-09, rgba(74, 144, 217, 0.12));
  color: var(--n-color-target);
}

.hero-avatar--anon {
  background: var(--n-color-embedded);
  color: var(--n-text-color-2);
}

.hero-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

/* ---- 详情行 ---- */
.detail-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-radius: 8px;
  background: var(--n-color-embedded);
}

.detail-row-icon {
  flex-shrink: 0;
}

.detail-row-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}
</style>
