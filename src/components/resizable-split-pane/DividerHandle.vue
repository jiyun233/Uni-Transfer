<script setup lang="ts">
import type { SplitDirection } from "./types"

withDefaults(
  defineProps<{
    direction: SplitDirection
    size: number
    isDragging: boolean
  }>(),
  {
    direction: "horizontal",
    size: 6,
    isDragging: false,
  },
)

const emit = defineEmits<{
  (e: "mousedown", payload: MouseEvent): void
  (e: "touchstart", payload: TouchEvent): void
}>()
</script>

<template>
  <div
    class="split-divider"
    :class="{
      'split-divider--horizontal': direction === 'horizontal',
      'split-divider--vertical': direction === 'vertical',
      'split-divider--dragging': isDragging,
    }"
    :style="{ '--divider-size': `${size}px` }"
    role="separator"
    :aria-orientation="direction"
    tabindex="0"
  >
    <div
      class="split-divider__hit-area"
      @mousedown.prevent="emit('mousedown', $event)"
      @touchstart.prevent="emit('touchstart', $event)"
    />
  </div>
</template>

<style scoped>
.split-divider {
  --divider-gutter: 4px;

  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  background: transparent;
  z-index: 10;
  user-select: none;
  touch-action: none;
}

/* ---- 水平分割（左右）---- */
.split-divider--horizontal {
  width: var(--divider-size);
  cursor: col-resize;
}

.split-divider--horizontal .split-divider__hit-area {
  position: absolute;
  inset: 0;
  /* 向外扩展点击热区 */
  margin: 0 calc(-1 * var(--divider-gutter));
  cursor: col-resize;
}

.split-divider--horizontal::after {
  content: "";
  width: 2px;
  height: 36px;
  border-radius: 1px;
  background: var(--n-border-color);
  transition: background 0.2s, height 0.2s;
}

/* ---- 垂直分割（上下）---- */
.split-divider--vertical {
  height: var(--divider-size);
  cursor: row-resize;
}

.split-divider--vertical .split-divider__hit-area {
  position: absolute;
  inset: 0;
  margin: calc(-1 * var(--divider-gutter)) 0;
  cursor: row-resize;
}

.split-divider--vertical::after {
  content: "";
  height: 2px;
  width: 36px;
  border-radius: 1px;
  background: var(--n-border-color);
  transition: background 0.2s, width 0.2s;
}

/* ---- 状态 ---- */
.split-divider--dragging::after,
.split-divider:hover::after {
  background: var(--n-color-target);
}

.split-divider--horizontal.split-divider--dragging::after,
.split-divider--horizontal:hover::after {
  height: 48px;
}

.split-divider--vertical.split-divider--dragging::after,
.split-divider--vertical:hover::after {
  width: 48px;
}

/* 拖拽时覆盖全屏光标，防止快速移动时光标闪烁 */
:global(body.split-pane--dragging) {
  cursor: col-resize !important;
  user-select: none !important;
}

:global(body.split-pane--dragging-vertical) {
  cursor: row-resize !important;
  user-select: none !important;
}
</style>
