<script setup lang="ts">
import { computed } from "vue"
import {
  NMenu,
  NButton,
  NIcon,
  NText,
  NTooltip,
} from "naive-ui"
import {
  CubeOutline,
  SunnyOutline,
  MoonOutline,
  CogOutline,
} from "@vicons/ionicons5"
import { useTheme } from "../../composables/useTheme"
import { useModuleRegistry } from "../../modules/registry"
import { renderIcon } from "../icons"
import type { MenuOption } from "naive-ui"

const { theme, toggle: toggleTheme } = useTheme()
const { modules, activeKey, switchTo } = useModuleRegistry()

const menuOptions = computed<MenuOption[]>(() =>
  modules.value.map((m) => ({
    label: m.label,
    key: m.key,
    icon: renderIcon(m.icon),
  })),
)
</script>

<template>
  <div class="top-menu-bar">
    <div class="topbar-left">
      <NIcon :size="22" :component="CubeOutline" color="var(--n-color-target)" />
      <NText strong style="font-size: 16px; letter-spacing: 0.5px">
        Uni-Transfer
      </NText>
    </div>

    <!-- 中间：模块导航 -->
    <div class="topbar-center">
      <NMenu mode="horizontal" :value="activeKey" :options="menuOptions" @update:value="switchTo" />
    </div>

    <!-- 右侧：操作按钮 -->
    <div class="topbar-right">
      <NTooltip trigger="hover">
        <template #trigger>
          <NButton text circle @click="toggleTheme" :aria-label="theme === 'light' ? '切换深色模式' : '切换浅色模式'">
            <template #icon>
              <NIcon :size="18" :component="theme === 'light' ? MoonOutline : SunnyOutline" />
            </template>
          </NButton>
        </template>
        {{ theme === "light" ? "深色模式" : "浅色模式" }}
      </NTooltip>

      <NTooltip trigger="hover">
        <template #trigger>
          <NButton text circle @click="switchTo('settings')" aria-label="设置">
            <template #icon>
              <NIcon :size="18" :component="CogOutline" />
            </template>
          </NButton>
        </template>
        设置
      </NTooltip>
    </div>
  </div>
</template>

<style scoped>
.top-menu-bar {
  display: flex;
  align-items: center;
  height: 100%;
  padding: 0 16px;
  gap: 16px;
}

.topbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  padding-right: 16px;
  border-right: 1px solid var(--n-border-color);
  margin-right: 4px;
}

.topbar-center {
  flex: 1;
  display: flex;
  align-items: center;
  overflow: hidden;
}

.topbar-right {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}
</style>
