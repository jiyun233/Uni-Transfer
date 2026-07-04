<script setup lang="ts">
import { ResizableSplitPane } from "../../components/resizable-split-pane"
import DeviceList from "./components/DeviceList.vue"
import DeviceDetail from "./components/DeviceDetail.vue"
</script>

<template>
  <div class="device-module">
    <ResizableSplitPane
      :initial-size="40"
      :min-size="20"
      :max-size="80"
      :divider-size="6"
      :allow-reset="true"
    >
      <template #primary="{ size }">
        <DeviceList :size="size" />
      </template>

      <template #secondary="{ size }">
        <DeviceDetail :size="size" />
      </template>
    </ResizableSplitPane>
  </div>
</template>

<style scoped>
.device-module {
  width: 100%;
  height: 100%;
}
</style>
