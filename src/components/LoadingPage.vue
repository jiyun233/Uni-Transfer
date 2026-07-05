<script setup lang="ts">
import { NIcon } from "naive-ui"
import { SyncOutline } from "@vicons/ionicons5"
</script>

<template>
  <div class="loading-page">
    <div class="loading-content">
      <NIcon :size="48" :component="SyncOutline" class="spinner" />
      <p class="app-title">Uni-Transfer</p>
      <p class="loading-text">正在加载...</p>
    </div>
  </div>
</template>

<style scoped>
.loading-page {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--n-color);
  z-index: 9999;
}

.loading-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  color: var(--n-text-color);
}

.spinner {
  animation: spin 1.2s linear infinite;
  color: var(--n-color-target, #4a90d9);
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}

.app-title {
  font-size: 24px;
  font-weight: 600;
  margin: 0;
  color: var(--n-text-color);
}

.loading-text {
  font-size: 13px;
  margin: 0;
  opacity: 0.5;
}
</style>
