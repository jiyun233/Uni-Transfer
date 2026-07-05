<script setup lang="ts">
const props = defineProps<{
  error?: unknown
  retry?: () => void
}>()

function handleRetry() {
  if (props.retry) {
    props.retry()
  } else {
    window.location.reload()
  }
}
</script>

<template>
  <div class="error-page">
    <div class="error-content">
      <!-- 错误图标 -->
      <div class="error-icon">
        <svg viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
          <circle cx="32" cy="32" r="28" stroke="currentColor" stroke-width="3" />
          <path d="M22 22L42 42M42 22L22 42" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
        </svg>
      </div>

      <h2 class="error-title">加载失败</h2>
      <p class="error-message">{{ String(error || "应用初始化时出现错误，请重试") }}</p>
      <details v-if="error" class="error-details">
        <summary>错误详情</summary>
        <pre>{{ error }}</pre>
      </details>

      <button class="retry-button" @click="handleRetry">
        重新加载
      </button>
    </div>
  </div>
</template>

<style scoped>
.error-page {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f8f9fa;
  z-index: 9998;
}

:root[data-theme="dark"] .error-page {
  background: #1a1a2e;
  color: #e0e0e0;
}

.error-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  text-align: center;
  animation: fadeInUp 0.5s ease;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.error-icon {
  width: 72px;
  height: 72px;
  color: #e74c3c;
  animation: errorShake 0.6s ease;
}

@keyframes errorShake {
  0%, 100% { transform: translateX(0); }
  20% { transform: translateX(-8px); }
  40% { transform: translateX(8px); }
  60% { transform: translateX(-4px); }
  80% { transform: translateX(4px); }
}

.error-title {
  font-size: 22px;
  font-weight: 700;
  margin: 0;
  color: #e74c3c;
}

.error-message {
  font-size: 14px;
  opacity: 0.6;
  margin: 0;
  max-width: 320px;
  line-height: 1.6;
}

.retry-button {
  margin-top: 8px;
  padding: 10px 28px;
  border: none;
  border-radius: 8px;
  background: linear-gradient(135deg, #4a90d9, #6366f1);
  color: white;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
}

.retry-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 16px rgba(74, 144, 217, 0.4);
}

.retry-button:active {
  transform: translateY(0);
}

.error-details {
  margin-top: 4px;
  text-align: left;
  max-width: 400px;
  width: 100%;
}

.error-details summary {
  cursor: pointer;
  font-size: 12px;
  opacity: 0.5;
  margin-bottom: 8px;
}

.error-details pre {
  font-size: 11px;
  background: rgba(0, 0, 0, 0.06);
  padding: 10px 12px;
  border-radius: 6px;
  overflow: auto;
  max-height: 160px;
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
}

:root[data-theme="dark"] .error-details pre {
  background: rgba(255, 255, 255, 0.06);
}
</style>
