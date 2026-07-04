import { ref, computed, onBeforeUnmount, type Ref, type ComputedRef } from "vue"
import type { SplitDirection, ResizePayload } from "./types"

interface UseSplitPaneOptions {
  direction: Ref<SplitDirection>
  initialSize: Ref<number>
  minSize: Ref<number>
  maxSize: Ref<number>
  containerRef: Ref<HTMLElement | null>
  onResizeStart?: () => void
  onResize?: (payload: ResizePayload) => void
  onResizeEnd?: (payload: ResizePayload) => void
}

interface UseSplitPaneReturn {
  /** 主面板当前占比（百分比） */
  primarySize: Ref<number>
  /** 次面板当前占比（百分比）= 100 - primarySize */
  secondarySize: ComputedRef<number>
  /** 是否正在拖拽 */
  isDragging: Ref<boolean>
  /** 绑定到分割线的 mousedown 处理器 */
  onDividerMouseDown: (e: MouseEvent) => void
  /** 绑定到分割线的 touchstart 处理器 */
  onDividerTouchStart: (e: TouchEvent) => void
  /** 重置面板大小为初始值 */
  reset: () => void
}

/**
 * 可拖拽分割面板的核心逻辑 composable
 *
 * 入参均为 Ref 以保证响应式同步；回调为可选的事件钩子。
 */
export function useSplitPane(options: UseSplitPaneOptions): UseSplitPaneReturn {
  const {
    direction,
    initialSize,
    minSize,
    maxSize,
    containerRef,
    onResizeStart,
    onResize,
    onResizeEnd,
  } = options

  const primarySize = ref(initialSize.value)
  const isDragging = ref(false)

  // ---- 工具函数 ----

  /**
   * 根据鼠标/触摸事件计算 primary 面板的新占比
   */
  function calcPercentage(clientX: number, clientY: number): number {
    const el = containerRef.value
    if (!el) return primarySize.value

    const rect = el.getBoundingClientRect()
    const isHorizontal = direction.value === "horizontal"

    const total = isHorizontal ? rect.width : rect.height
    const offset = isHorizontal ? clientX - rect.left : clientY - rect.top

    if (total <= 0) return primarySize.value

    const pct = (offset / total) * 100
    return clamp(pct)
  }

  /** 将百分比钳制在 [minSize, maxSize] 之间 */
  function clamp(value: number): number {
    return Math.max(minSize.value, Math.min(maxSize.value, value))
  }

  /** 清除全局事件监听 */
  function unbindGlobalEvents() {
    document.removeEventListener("mousemove", onMouseMove)
    document.removeEventListener("mouseup", onMouseUp)
    document.removeEventListener("touchmove", onTouchMove)
    document.removeEventListener("touchend", onTouchEnd)
  }

  // ---- 鼠标拖拽 ----

  function onDividerMouseDown(e: MouseEvent) {
    e.preventDefault()
    beginDrag()
    document.addEventListener("mousemove", onMouseMove)
    document.addEventListener("mouseup", onMouseUp)
  }

  function onMouseMove(e: MouseEvent) {
    if (!isDragging.value) return
    primarySize.value = calcPercentage(e.clientX, e.clientY)
    onResize?.(payload())
  }

  function onMouseUp() {
    if (!isDragging.value) return
    endDrag()
  }

  // ---- 触摸拖拽 ----

  function onDividerTouchStart(e: TouchEvent) {
    e.preventDefault()
    beginDrag()
    document.addEventListener("touchmove", onTouchMove, { passive: false })
    document.addEventListener("touchend", onTouchEnd)
    document.addEventListener("touchcancel", onTouchEnd)
  }

  function onTouchMove(e: TouchEvent) {
    if (!isDragging.value) return
    const touch = e.touches[0]
    if (!touch) return
    primarySize.value = calcPercentage(touch.clientX, touch.clientY)
    onResize?.(payload())
  }

  function onTouchEnd() {
    if (!isDragging.value) return
    endDrag()
    document.removeEventListener("touchcancel", onTouchEnd)
  }

  // ---- 拖拽生命周期 ----

  function beginDrag() {
    isDragging.value = true
    onResizeStart?.()
  }

  function endDrag() {
    isDragging.value = false
    unbindGlobalEvents()
    onResizeEnd?.(payload())
  }

  // ---- 辅助 ----

  function payload(): ResizePayload {
    return {
      left: Math.round(primarySize.value * 100) / 100,
      right: Math.round((100 - primarySize.value) * 100) / 100,
    }
  }

  function reset() {
    primarySize.value = initialSize.value
  }

  const secondarySize = computed(() => 100 - primarySize.value)

  // 组件卸载时确保全局事件解绑，防止内存泄漏
  onBeforeUnmount(() => {
    unbindGlobalEvents()
  })

  return {
    primarySize,
    secondarySize,
    isDragging,
    onDividerMouseDown,
    onDividerTouchStart,
    reset,
  }
}
