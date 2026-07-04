import { PhonePortraitOutline } from "@vicons/ionicons5"
import { useModuleRegistry } from "../registry"
import DeviceModule from "./DeviceModule.vue"

const { register } = useModuleRegistry()
register({
  key: "device",
  label: "设备管理",
  icon: PhonePortraitOutline,
  component: DeviceModule,
})
