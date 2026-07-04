<script setup lang="ts">
import { NIcon, NText } from "naive-ui"
import { FolderOpenOutline } from "@vicons/ionicons5"
</script>

<template>
  <div class="empty-module">
    <div class="empty-icon">
      <NIcon :size="48" :component="FolderOpenOutline" color="var(--n-text-color-3)" />
    </div>
    <NText tag="h2" strong>文件浏览</NText>
    <NText depth="3">功能开发中...</NText>
    <!-- 在此处编写文件浏览内容 -->
  </div>
</template>

<style scoped>
.empty-module {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 24px;
}

.empty-icon {
  margin-bottom: 8px;
  opacity: 0.5;
}
</style>
