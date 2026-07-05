<script setup lang="ts">
import { ref, onErrorCaptured } from "vue"
import { NLayout, NLayoutHeader, NLayoutContent, NText, NButton, NSpace } from "naive-ui"
import "../../modules"
import { useModuleRegistry } from "../../modules/registry"
import TopMenuBar from "./TopMenuBar.vue"

const { activeModule } = useModuleRegistry()

const moduleError = ref<string | null>(null)

onErrorCaptured((err, _instance, info) => {
  console.error("[AppShell] 模块渲染错误:", err, info)
  moduleError.value = String(err)
  return false
})

function clearModuleError() {
  moduleError.value = null
}
</script>

<template>
  <NLayout position="absolute" class="app-layout">
    <NLayoutHeader bordered>
      <TopMenuBar />
    </NLayoutHeader>

    <NLayoutContent class="app-content">
      <template v-if="!moduleError">
        <component :is="activeModule?.component ?? null" class="module-viewport" />
      </template>

      <div v-else class="module-viewport module-error-fallback">
        <NSpace vertical align="center" :size="16">
          <div class="error-icon-circle">!</div>
          <NText tag="h3" strong>模块加载失败</NText>
          <NText depth="3" class="error-detail">{{ moduleError }}</NText>
          <NSpace :size="12">
            <NButton type="primary" size="small" @click="clearModuleError">重试</NButton>
            <NButton size="small" @click="moduleError = null">忽略</NButton>
          </NSpace>
        </NSpace>
      </div>
    </NLayoutContent>
  </NLayout>
</template>

<style>
/* 全局：确保 html/body/app 高度链条完整 */
html, body, #app {
  height: 100%;
  margin: 0;
  padding: 0;
  overflow: hidden;
}
</style>

<style scoped>
.app-layout {
  width: 100%;
  height: 100%;
}

.app-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.module-viewport {
  width: 100%;
  height: 100%;
  /* 关键：viewport 本身不滚动，内部组件自行处理溢出 */
}

.module-error-fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  text-align: center;
}

.error-icon-circle {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: rgba(231, 76, 60, 0.15);
  color: #e74c3c;
  font-size: 22px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
}

.error-detail {
  max-width: 420px;
  word-break: break-all;
  font-size: 12px;
}
</style>
