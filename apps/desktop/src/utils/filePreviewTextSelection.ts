const PREVIEW_SELECTION_STYLE_ID = 'localkit-preview-selection-style'

const PREVIEW_SELECTION_CSS = `
  .pdfViewer.enablePermissions .textLayer span:not([role='img']) {
    -webkit-user-select: text !important;
    -moz-user-select: text !important;
    user-select: text !important;
    cursor: text !important;
  }

  .textLayerImages {
    pointer-events: none !important;
  }

  .textLayer {
    z-index: 2 !important;
    pointer-events: auto !important;
    -webkit-user-select: text !important;
    user-select: text !important;
  }

  .textLayer :is(span:not([role='img']), br) {
    pointer-events: auto !important;
    -webkit-user-select: text !important;
    user-select: text !important;
  }

  .pdfViewer .page .canvasWrapper,
  .pdfViewer .page canvas {
    pointer-events: none !important;
  }

  .pdfViewer .page .annotationEditorLayer:not(:has(.selectedEditor)) {
    pointer-events: none !important;
  }

  .file-render,
  .file-render-host,
  .content,
  .docx-fit-viewer,
  .docx-wrapper,
  .docx-page-frame,
  .docx-flow-frame,
  section.docx,
  section.docx *:not(img):not(svg):not(canvas) {
    -webkit-user-select: text !important;
    user-select: text !important;
    cursor: text;
  }
`

type PreviewSelectionBinding = {
  disconnect: () => void
}

function collectSelectionRoots(host: HTMLElement) {
  const roots: Array<DocumentFragment | HTMLElement> = [host]
  const stack: Node[] = [host]

  while (stack.length > 0) {
    const node = stack.pop()
    if (!node) {
      continue
    }

    if (node instanceof HTMLElement && node.shadowRoot) {
      roots.push(node.shadowRoot)
      stack.push(node.shadowRoot)
    }

    node.childNodes.forEach((child) => {
      stack.push(child)
    })
  }

  return roots
}

function injectSelectionStyle(root: DocumentFragment | HTMLElement) {
  const owner =
    root instanceof DocumentFragment
      ? root.host?.ownerDocument ?? null
      : root.ownerDocument

  if (!owner) {
    return
  }

  const existing = root.querySelector(`#${PREVIEW_SELECTION_STYLE_ID}`)
  if (existing) {
    return
  }

  const style = owner.createElement('style')
  style.id = PREVIEW_SELECTION_STYLE_ID
  style.textContent = PREVIEW_SELECTION_CSS
  root.appendChild(style)
}

function applyInlineSelectionStyles(root: ParentNode) {
  root.querySelectorAll<HTMLElement>('.pdfViewer.enablePermissions').forEach((viewer) => {
    viewer.classList.remove('enablePermissions')
  })

  root.querySelectorAll<HTMLElement>('.textLayer').forEach((layer) => {
    layer.style.setProperty('pointer-events', 'auto', 'important')
    layer.style.setProperty('user-select', 'text', 'important')
    layer.style.setProperty('-webkit-user-select', 'text', 'important')
    layer.style.setProperty('z-index', '2', 'important')
  })

  root.querySelectorAll<HTMLElement>('.textLayer span:not([role="img"])').forEach((span) => {
    span.style.setProperty('user-select', 'text', 'important')
    span.style.setProperty('-webkit-user-select', 'text', 'important')
    span.style.setProperty('cursor', 'text', 'important')
  })

  root.querySelectorAll<HTMLElement>('.pdfViewer .page canvas, .pdfViewer .page .canvasWrapper').forEach(
    (element) => {
      element.style.setProperty('pointer-events', 'none', 'important')
    },
  )

  root.querySelectorAll<HTMLElement>('.textLayerImages').forEach((element) => {
    element.style.setProperty('pointer-events', 'none', 'important')
  })

  root.querySelectorAll<HTMLElement>('section.docx, section.docx *').forEach((element) => {
    if (element.matches('img, svg, canvas')) {
      return
    }

    element.style.setProperty('user-select', 'text', 'important')
    element.style.setProperty('-webkit-user-select', 'text', 'important')
  })
}

export function enablePreviewTextSelection(host: HTMLElement | null | undefined) {
  if (!host) {
    return
  }

  for (const root of collectSelectionRoots(host)) {
    injectSelectionStyle(root)
    applyInlineSelectionStyles(root)
  }
}

export function scheduleEnablePreviewTextSelection(
  host: HTMLElement | null | undefined,
) {
  if (!host) {
    return
  }

  const run = () => enablePreviewTextSelection(host)
  run()
  requestAnimationFrame(run)
  window.setTimeout(run, 120)
  window.setTimeout(run, 480)
  window.setTimeout(run, 1200)
}

export function bindPreviewTextSelection(
  host: HTMLElement | null | undefined,
): PreviewSelectionBinding | null {
  if (!host) {
    return null
  }

  let pendingFrame = 0
  let pendingTimer: ReturnType<typeof setTimeout> | undefined

  const run = () => {
    enablePreviewTextSelection(host)
  }

  const schedule = () => {
    if (pendingFrame) {
      cancelAnimationFrame(pendingFrame)
    }
    pendingFrame = requestAnimationFrame(() => {
      pendingFrame = 0
      run()
    })

    if (pendingTimer) {
      clearTimeout(pendingTimer)
    }
    pendingTimer = window.setTimeout(() => {
      pendingTimer = undefined
      run()
    }, 80)
  }

  schedule()
  scheduleEnablePreviewTextSelection(host)

  const observer = new MutationObserver(() => {
    schedule()
  })

  observer.observe(host, {
    attributes: true,
    attributeFilter: ['class', 'style'],
    childList: true,
    subtree: true,
  })

  return {
    disconnect: () => {
      observer.disconnect()
      if (pendingFrame) {
        cancelAnimationFrame(pendingFrame)
        pendingFrame = 0
      }
      if (pendingTimer) {
        clearTimeout(pendingTimer)
        pendingTimer = undefined
      }
    },
  }
}
