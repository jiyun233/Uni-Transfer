import { ref, computed, type Component, type Ref, type ComputedRef } from "vue"

export interface ModuleDefinition {
  key: string
  label: string
  icon: Component
  component: Component
}

const modules = ref<ModuleDefinition[]>([])
const activeKey = ref<string>("")

export function useModuleRegistry(): {
  modules: Ref<ModuleDefinition[]>
  activeKey: Ref<string>
  activeModule: ComputedRef<ModuleDefinition | null>
  register: (def: ModuleDefinition) => void
  switchTo: (key: string) => void
} {
  function register(def: ModuleDefinition) {
    if (modules.value.some((m) => m.key === def.key)) {
      console.warn(`[ModuleRegistry] 模块 key="${def.key}" 已存在，跳过重复注册`)
      return
    }
    modules.value.push(def)
    console.log(`[ModuleRegistry] 模块 key="${def.key}" 注册成功`)
    if (!activeKey.value) {
      activeKey.value = def.key
    }
  }

  function switchTo(key: string) {
    if (modules.value.some((m) => m.key === key)) {
      activeKey.value = key
    }
  }

  const activeModule = computed<ModuleDefinition | null>(
    () => modules.value.find((m) => m.key === activeKey.value) ?? null,
  )

  return {
    modules,
    activeKey,
    activeModule,
    register,
    switchTo,
  }
}
