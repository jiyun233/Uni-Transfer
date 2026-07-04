import { SwapHorizontalOutline } from "@vicons/ionicons5"
import { useModuleRegistry } from "../registry"
import TransferModule from "./TransferModule.vue"

const { register } = useModuleRegistry()
register({
  key: "transfer",
  label: "传输任务",
  icon: SwapHorizontalOutline,
  component: TransferModule,
})
