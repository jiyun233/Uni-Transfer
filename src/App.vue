<script setup lang="ts">
import { computed } from "vue"
import { NConfigProvider, darkTheme, zhCN, dateZhCN } from "naive-ui"
import { useTheme } from "./composables/useTheme"
import { AppShell } from "./components/layout"

const { theme } = useTheme()
const naiveTheme = computed(() => (theme.value === "dark" ? darkTheme : null))
</script>

<template>
  <NConfigProvider :theme="naiveTheme" :locale="zhCN" :date-locale="dateZhCN">
    <AppShell />
  </NConfigProvider>
</template>
