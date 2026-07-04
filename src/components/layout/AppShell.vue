<script setup lang="ts">
import { NLayout, NLayoutHeader, NLayoutContent } from "naive-ui"
import "../../modules"
import { useModuleRegistry } from "../../modules/registry"
import TopMenuBar from "./TopMenuBar.vue"

const { activeModule } = useModuleRegistry()
</script>

<template>
  <NLayout position="absolute">
    <NLayoutHeader bordered>
      <TopMenuBar />
    </NLayoutHeader>

    <NLayoutContent>
      <component :is="activeModule?.component ?? null" class="module-viewport" />
    </NLayoutContent>
  </NLayout>
</template>

<style scoped>
.module-viewport {
  width: 100%;
  height: 100%;
}
</style>
