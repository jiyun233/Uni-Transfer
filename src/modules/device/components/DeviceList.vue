<script setup lang="ts">
import { NIcon, NText } from "naive-ui"
import { PhonePortraitOutline } from "@vicons/ionicons5"

defineProps<{ size: number }>()
</script>

<template>
  <div class="device-list">
    <div class="panel-header">
      <NIcon :size="20" :component="PhonePortraitOutline" />
      <NText strong style="font-size: 15px">设备列表</NText>
    </div>
    <div class="panel-body">
      <p class="muted-hint">暂无设备连接</p>
      <!-- 在此处编写设备列表内容 -->
    </div>
  </div>
</template>

<style scoped>
.device-list {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 16px;
  overflow: auto;
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--n-border-color);
}

.panel-body {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.muted-hint {
  color: var(--n-text-color-3);
  font-size: 13px;
}
</style>
