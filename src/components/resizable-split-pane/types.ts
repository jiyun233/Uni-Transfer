export type SplitDirection = "horizontal" | "vertical"

export interface SplitPaneProps {
  /** 分割方向：horizontal = 左右，vertical = 上下 */
  direction?: SplitDirection
  /** 左侧/上方面板初始占比（百分比 0-100） */
  initialSize?: number
  /** 左侧/上方面板最小占比（百分比 0-100） */
  minSize?: number
  /** 左侧/上方面板最大占比（百分比 0-100） */
  maxSize?: number
  /** 分割线宽度（px） */
  dividerSize?: number
}

export interface ResizePayload {
  left: number
  right: number
}

export interface SplitPaneEmits {
  (e: "resize-start"): void
  (e: "resize", payload: ResizePayload): void
  (e: "resize-end", payload: ResizePayload): void
}

/** 内部使用的常量 */
export const DEF_MIN_SIZE = 15
export const DEF_MAX_SIZE = 85
export const DEF_INITIAL_SIZE = 50
export const DEF_DIVIDER_SIZE = 6
