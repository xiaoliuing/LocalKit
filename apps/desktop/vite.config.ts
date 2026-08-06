import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { fileViewerRenderers } from '@file-viewer/vite-plugin'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'
import type { ResolvedDocsConfig } from '../web/scripts/docsConfig'
import { createDocsDataPlugin } from '../web/scripts/docsData'

const projectRoot = fileURLToPath(new URL('.', import.meta.url))
const projectDocsRoot = path.resolve(projectRoot, '../../docs')

export default defineConfig(({ mode }) => {
  const docsConfig: ResolvedDocsConfig = {
    sourceGroups: [
      {
        children: [],
        id: 'source:project-docs',
        isSource: true,
        mountPath: 'docs',
        name: '项目文档',
        sourceId: 'source:project-docs',
        sourceLabel: '项目文档',
      },
    ],
    sources: [
      {
        displayName: '项目文档',
        id: 'source:project-docs',
        mountPath: 'docs',
        root: projectDocsRoot,
        sourceLabel: '项目文档',
      },
    ],
  }

  return {
    server: {
      host: '127.0.0.1',
      port: 43174,
      strictPort: true,
    },
    preview: {
      host: '127.0.0.1',
      port: 43174,
      strictPort: true,
    },
    plugins: [
      vue(),
      fileViewerRenderers({
        copyAssets: true,
        autoPresets: ['lite', 'office', 'engineering'],
        formats: ['eml', 'msg', 'mbox'],
      }),
      createDocsDataPlugin({
        appBase: '/',
        docsConfig,
        enableHotReload: false,
      }),
    ],
    resolve: {
      alias: {
        '@': path.resolve(projectRoot, 'src'),
      },
    },
  }
})
