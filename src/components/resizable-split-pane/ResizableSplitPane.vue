<script setup lang="ts">
import { watch, computed, useTemplateRef } from "vue"
import { useSplitPane } from "./useSplitPane"
import DividerHandle from "./DividerHandle.vue"
import type {
  ResizePayload,
  SplitDirection,
} from "./types"
import {
  DEF_MIN_SIZE,
  DEF_MAX_SIZE,
  DEF_INITIAL_SIZE,
  DEF_DIVIDER_SIZE,
} from "./types"

const props = withDefaults(defineProps<{
  direction?: SplitDirection
  initialSize?: number
  minSize?: number
  maxSize?: number
  dividerSize?: number
  /** 是否允许双击分割线重置大小 */
  allowReset?: boolean
}>(), {
  direction: "horizontal",
  initialSize: DEF_INITIAL_SIZE,
  minSize: DEF_MIN_SIZE,
  maxSize: DEF_MAX_SIZE,
  dividerSize: DEF_DIVIDER_SIZE,
  allowReset: true,
})

const emit = defineEmits<{
  (e: "resize-start"): void
  (e: "resize", payload: ResizePayload): void
  (e: "resize-end", payload: ResizePayload): void
}>()

// ---- 响应式 props 包装 ----
const directionRef = computed(() => props.direction)
const initialSizeRef = computed(() => props.initialSize)
const minSizeRef = computed(() => props.minSize)
const maxSizeRef = computed(() => props.maxSize)

const containerRef = useTemplateRef<HTMLElement>("container")

// ---- 拖拽光标全局类名 ----
const draggingClass = computed(() =>
  props.direction === "horizontal"
    ? "split-pane--dragging"
    : "split-pane--dragging-vertical",
)

function syncBodyClass(isDragging: boolean) {
  if (!import.meta.env.SSR) {
    document.body.classList.toggle(draggingClass.value, isDragging)
  }
}

// ---- 核心逻辑 ----
const {
  primarySize,
  secondarySize,
  isDragging,
  onDividerMouseDown,
  onDividerTouchStart,
  reset,
} = useSplitPane({
  direction: directionRef,
  initialSize: initialSizeRef,
  minSize: minSizeRef,
  maxSize: maxSizeRef,
  containerRef,
  onResizeStart() {
    syncBodyClass(true)
    emit("resize-start")
  },
  onResize(payload) {
    emit("resize", payload)
  },
  onResizeEnd(payload) {
    syncBodyClass(false)
    emit("resize-end", payload)
  },
})

// drag 结束时额外做一次兜底：确保 body class 被移除
watch(isDragging, (val) => {
  if (!val) syncBodyClass(false)
})

// ---- 双击重置 ----
function onDividerDblClick() {
  if (props.allowReset) {
    reset()
    emit("resize-end", {
      left: props.initialSize,
      right: 100 - props.initialSize,
    })
  }
}

// ---- CSS 变量 ----
const containerStyle = computed(() => {
  const isHorizontal = props.direction === "horizontal"
  return {
    "--primary-size": `${primarySize.value}%`,
    "--secondary-size": `${secondarySize.value}%`,
    "--divider-size": `${props.dividerSize}px`,
    flexDirection: isHorizontal ? ("row" as const) : ("column" as const),
  }
})
</script>

<template>
  <div
    ref="container"
    class="split-pane"
    :class="{
      'split-pane--horizontal': direction === 'horizontal',
      'split-pane--vertical': direction === 'vertical',
      'split-pane--dragging': isDragging,
    }"
    :style="containerStyle"
  >
    <!-- 主面板（左 / 上） -->
    <div class="split-pane__panel split-pane__panel--primary">
      <slot name="primary" :size="primarySize" />
    </div>

    <!-- 分割线 -->
    <DividerHandle
      :direction="direction"
      :size="dividerSize"
      :is-dragging="isDragging"
      @mousedown="onDividerMouseDown"
      @touchstart="onDividerTouchStart"
      @dblclick.stop="onDividerDblClick"
    />

    <!-- 次面板（右 / 下） -->
    <div class="split-pane__panel split-pane__panel--secondary">
      <slot name="secondary" :size="secondarySize" />
    </div>
  </div>
</template>

<style scoped>
.split-pane {
  display: flex;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

/* 拖拽时禁止 iframe/图片等内容的指针事件，防止"粘滞" */
.split-pane--dragging {
  pointer-events: none;
}

.split-pane--dragging .split-pane__panel {
  pointer-events: none;
}

/* ---- 面板 ---- */
.split-pane__panel {
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.split-pane__panel--primary {
  flex: 0 0 var(--primary-size);
}

.split-pane__panel--secondary {
  flex: 1 1 auto;
  min-width: 0;
  min-height: 0;
}
</style>
