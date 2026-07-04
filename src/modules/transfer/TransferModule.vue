<script setup lang="ts">
import { NIcon, NText } from "naive-ui"
import { SwapHorizontalOutline } from "@vicons/ionicons5"
</script>

<template>
  <div class="empty-module">
    <div class="empty-icon">
      <NIcon :size="48" :component="SwapHorizontalOutline" color="var(--n-text-color-3)" />
    </div>
    <NText tag="h2" strong>传输任务</NText>
    <NText depth="3">功能开发中...</NText>
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
