import { SettingsOutline } from "@vicons/ionicons5"
import { useModuleRegistry } from "../registry"
import SettingsModule from "./SettingsModule.vue"

const { register } = useModuleRegistry()
register({
  key: "settings",
  label: "设置",
  icon: SettingsOutline,
  component: SettingsModule,
})
