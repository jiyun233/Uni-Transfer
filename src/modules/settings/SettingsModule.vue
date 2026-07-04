<script setup lang="ts">
import {
  NIcon,
  NText,
  NCard,
  NSpace,
  NButton,
  NDivider,
} from "naive-ui"
import { SunnyOutline, MoonOutline } from "@vicons/ionicons5"
import { useTheme } from "../../composables/useTheme"

const { theme, toggle } = useTheme()
</script>

<template>
  <div class="settings-module">
    <NText tag="h2" strong style="margin-bottom: 24px">设置</NText>

    <NSpace vertical :size="16" style="max-width: 520px">
      <!-- 外观设置 -->
      <NCard title="外观" size="small">
        <div class="setting-row">
          <div class="setting-info">
            <NText strong>主题模式</NText>
            <NText depth="3" style="font-size: 13px">当前：{{ theme === "light" ? "浅色" : "深色" }}</NText>
          </div>
          <NButton @click="toggle" quaternary>
            <template #icon>
              <NIcon :component="theme === 'light' ? MoonOutline : SunnyOutline" />
            </template>
            切换到{{ theme === "light" ? "深色" : "浅色" }}模式
          </NButton>
        </div>
      </NCard>

      <!-- 关于 -->
      <NCard title="关于" size="small">
        <NSpace vertical :size="8">
          <div class="setting-row">
            <NText depth="3">应用名称</NText>
            <NText strong>Transfer</NText>
          </div>
          <NDivider style="margin: 4px 0" />
          <div class="setting-row">
            <NText depth="3">版本</NText>
            <NText>0.1.0</NText>
          </div>
          <NDivider style="margin: 4px 0" />
          <div class="setting-row">
            <NText depth="3">技术栈</NText>
            <NText>Vue 3 + NaiveUI + Tauri 2</NText>
          </div>
        </NSpace>
      </NCard>
    </NSpace>
  </div>
</template>

<style scoped>
.settings-module {
  height: 100%;
  padding: 24px;
  overflow: auto;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
</style>
