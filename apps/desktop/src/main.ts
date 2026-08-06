import { createApp } from 'vue'
import FileViewer from '@file-viewer/vue3'
import '@file-viewer/vue3/dist/file-viewer3.css'
import App from './App.vue'
import '../../web/src/assets/main.css'
import './assets/main.css'
import './assets/filePreviewSelection.css'
import '@milkdown/crepe/theme/common/style.css'
import '@milkdown/crepe/theme/frame.css'

createApp(App).use(FileViewer).mount('#app')
