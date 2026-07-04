import { FolderOpenOutline } from "@vicons/ionicons5"
import { useModuleRegistry } from "../registry"
import FilesModule from "./FilesModule.vue"

const { register } = useModuleRegistry()
register({
  key: "files",
  label: "文件浏览",
  icon: FolderOpenOutline,
  component: FilesModule,
})
