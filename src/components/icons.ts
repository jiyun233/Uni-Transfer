import { NIcon } from "naive-ui"
import { h, type Component } from "vue"

/**
 * 将 vicon / 任意 Vue 组件转换为 NaiveUI NMenu 所需的 render 函数
 *
 * @example
 * ```ts
 * import { PhonePortraitOutline } from '@vicons/ionicons5'
 * const menuOptions = [{ label: '设备', key: 'device', icon: renderIcon(PhonePortraitOutline) }]
 * ```
 */
export function renderIcon(iconComponent: Component, size = 18) {
  return () => h(NIcon, { size }, { default: () => h(iconComponent) })
}
