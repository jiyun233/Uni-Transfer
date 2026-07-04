import { PhonePortraitOutline } from "@vicons/ionicons5"
import { useModuleRegistry } from "../registry"
import ConnectModule from "./ConnectModule.vue"

const { register } = useModuleRegistry()
register({
  key: "connect",
  label: "设备连接",
  icon: PhonePortraitOutline,
  component: ConnectModule,
})
