<script setup lang="ts">
import { ref, provide } from "vue"
import { ResizableSplitPane } from "../../components/resizable-split-pane"
import DeviceList from "./components/DeviceList.vue"
import DeviceDetail from "./components/DeviceDetail.vue"

interface ActiveSession {
  session_id: string
  username: string
  ip: string
  connected_at: string
  is_anonymous: boolean
}

const selectedSession = ref<ActiveSession | null>(null)

function selectSession(s: ActiveSession | null) {
  selectedSession.value = s
}

provide("selectedSession", selectedSession)
provide("selectSession", selectSession)
</script>

<template>
  <div class="device-module">
    <ResizableSplitPane
      :initial-size="40"
      :min-size="25"
      :max-size="65"
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
  overflow: hidden;
}
</style>
