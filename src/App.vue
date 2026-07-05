<script setup lang="ts">
import { ref, computed, onMounted, onErrorCaptured } from "vue"
import { NConfigProvider, darkTheme, zhCN, dateZhCN } from "naive-ui"
import { useTheme } from "./composables/useTheme"
import LoadingPage from "./components/LoadingPage.vue"
import ErrorPage from "./components/ErrorPage.vue"
import { AppShell } from "./components/layout"

const { theme } = useTheme()
const naiveTheme = computed(() => (theme.value === "dark" ? darkTheme : null))

type AppState = "loading" | "ready" | "error"
const appState = ref<AppState>("loading")
const errorMessage = ref("")

async function initApp() {
  appState.value = "loading"
  errorMessage.value = ""
  try {
    // 确保 loading 页面先渲染一帧
    await new Promise((r) => requestAnimationFrame(r))
    // 短暂延迟让过渡动画更自然
    await new Promise((r) => setTimeout(r, 400))
    appState.value = "ready"
  } catch (e: unknown) {
    console.error("[App] 初始化失败:", e)
    appState.value = "error"
    errorMessage.value = e instanceof Error ? e.message : String(e)
  }
}

function handleRetry() {
  initApp()
}

// 全局错误边界：捕获子组件未处理的异常
onErrorCaptured((err, _instance, info) => {
  const detail = err instanceof Error ? `${err.message}\n${err.stack || ""}` : String(err)
  console.error("[App] 捕获到组件错误:", detail, info)
  appState.value = "error"
  errorMessage.value = detail
  return false // 阻止继续冒泡
})

onMounted(() => {
  initApp()
})
</script>

<template>
  <NConfigProvider :theme="naiveTheme" :locale="zhCN" :date-locale="dateZhCN">
    <Transition name="app-fade" mode="out-in">
      <LoadingPage v-if="appState === 'loading'" key="loading" />
      <ErrorPage
        v-else-if="appState === 'error'"
        key="error"
        :error="errorMessage"
        :retry="handleRetry"
      />
      <AppShell v-else key="app" />
    </Transition>
  </NConfigProvider>
</template>

<style>
/* 全局过渡动画：loading → app / error → app */
.app-fade-enter-active,
.app-fade-leave-active {
  transition: opacity 0.35s ease;
}
.app-fade-enter-from,
.app-fade-leave-to {
  opacity: 0;
}
</style>
