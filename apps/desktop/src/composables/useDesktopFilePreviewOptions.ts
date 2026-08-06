import type { FileViewerOptions } from '@file-viewer/vue3'

export function getDesktopFilePreviewOptions(): FileViewerOptions {
  return {
    locale: 'zh-CN',
    theme: 'system',
    styleIsolation: 'none',
    toolbar: {
      position: 'auto',
      download: true,
      exportHtml: false,
      print: true,
      search: true,
      zoom: true,
    },
  }
}
