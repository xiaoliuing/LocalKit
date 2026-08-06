## Context

LocalKit 桌面端已有工具中心、视频播放器与 Agent 会话清理等独立工具。视频工具已实现「多目录源 + 左栏库 + 右栏预览 + localStorage 持久化 + Rust 扫描命令」模式，可作为文件预览工具的结构参考。实现阶段已从早期 `@open-file-viewer/vue` 迁移至 `@file-viewer/vue3` 2.x 生态，并通过 Vite 插件装配 preset。

详见 `proposal.md` 的 Why 与 What Changes。

## Goals / Non-Goals

**Goals:**

- 将文件预览作为工具中心独立工具接入，交互模式与视频工具保持一致（返回工具中心、左栏库、右栏预览）。
- 支持多个预览源扁平管理；每个源可自定义显示名称、路径与扫描层级。
- Rust 侧提供按层级限制的目录扫描，前端以树形展示 folder/file 节点。
- 使用 `@file-viewer/vue3` 作为预览引擎，覆盖 lite / office / engineering 及邮件等常见格式（约 200+ 扩展名）。
- 通过 composable + localStorage 持久化源配置、展开状态、当前文件与预览 view state。
- legacy 纯文本（如 GBK 编码 `.txt`）经 Rust 解码后以 `File` 对象交给 viewer。
- PDF / Word 等文档在预览区支持文本选中与复制。

**Non-Goals:**

- 不在 Web 端提供该工具。
- 不实现文件编辑、移动、删除或批量操作。
- 不实现跨源全文搜索（可后续迭代）。
- 不替换文档阅读工具对 Markdown 知识库的管理职责。
- 扫描件 PDF 无文字层时无法 magically 提供可选中文本（依赖源文件本身）。

## Decisions

### Decision: 复用视频工具的壳层模式，而非新建布局体系

文件预览工具采用与 `DesktopVideoToolView` 相同的两栏布局：左侧 library（源列表 + 树），右侧 preview（`<file-viewer>`）。`DesktopAppShell` 新增 `primaryView: "file-preview"` 并纳入 `isToolView`。

备选方案是嵌入文档阅读侧栏。该方案会混淆「知识库 Markdown」与「通用文件预览」边界，且破坏工具中心统一入口。

### Decision: 预览源扁平、源内树形

顶层 `sources[]` 扁平存储多个根目录；每个源扫描后生成 `PreviewTreeNode[]` 树。源级字段包括 `id`、`title`、`path`、`maxDepth`。

### Decision: 每源独立 `maxDepth`，默认 3，0 表示不限制

Rust 扫描时使用 `max_depth` 参数控制递归深度（根目录为第 1 层）。UI 提供下拉选择（1–10）及「不限制」。

### Decision: Rust 扫描 + 前端预览

新增 Tauri command `scan_preview_directory(path, max_depth)`，返回嵌套 `children` 树。扫描时跳过 `.git`、`node_modules`、`.DS_Store` 等；防止符号链接导致无限循环。

另增 `read_preview_text_file(path)`，对 legacy 纯文本扩展名尝试 UTF-8 / UTF-16 / GBK 解码，供 viewer 以 `File` 输入渲染。

### Decision: Tauri 文件加载优先使用 `convertFileSrc`

预览输入默认使用 `convertFileSrc(absolutePath)` 生成 URL 传给 `<file-viewer>`。legacy 纯文本走 Rust 解码 + `File` 对象。

### Decision: 添加/编辑对话框复用视频源对话框模式

`DesktopFilePreviewSourceDialog` 字段：显示名称、目录路径、扫描层级。选择目录后若 title 为空则自动填 basename。

### Decision: 预览引擎为 `@file-viewer/vue3` + Vite 插件

```ts
// vite.config.ts
fileViewerRenderers({
  copyAssets: true,
  autoPresets: ['lite', 'office', 'engineering'],
  formats: ['eml', 'msg', 'mbox'],
})

// main.ts
import FileViewer from '@file-viewer/vue3'
import '@file-viewer/vue3/dist/file-viewer3.css'
createApp(App).use(FileViewer)
```

Viewer options：

- `locale: 'zh-CN'`
- `theme: 'system'`
- `styleIsolation: 'none'`（允许宿主 CSS 修复 PDF 文本层选中等问题）
- `initialViewState` 从 localStorage 恢复

早期 MVP 仅 image/pdf/text/fallback 的方案已废弃，改为 preset 装配。

### Decision: 预览位置记忆

composable 维护 `viewStateMemory: Record<fileId, FileViewerViewState>`，在 `@view-state-change` 防抖写入 localStorage；加载完成后 `applyViewState` 恢复页码/滚动。

### Decision: 文本选中修复

PDF 可能因 `enablePermissions` 与 canvas 遮挡导致无法选中；Word 需确保 docx 正文可选中。通过全局 CSS（`filePreviewSelection.css`）、shadow-root 注入与 `MutationObserver` 持续修复（`filePreviewTextSelection.ts`）。

### Decision: 持久化键

使用 `docs-atlas.desktop.file-preview.v1`，存储：

- `sources: { id, title, path, maxDepth }[]`
- `currentFileId`
- `expandedSourceIds` / `expandedFolderIds`
- `isLibraryCollapsed`
- `viewStateMemory`

### Decision: 格式说明

`DesktopFilePreviewFormatDialog` + `filePreviewFormats.ts` 维护支持/不支持扩展名列表，左栏提供「格式」入口。

## Risks / Trade-offs

- [大目录扫描慢] → 默认 depth=3；扫描中展示 loading。
- [convertFileSrc 对部分格式不稳定] → fallback 插件兜底；legacy 文本走 Rust 解码。
- [viewer bundle 体积] → preset 按需 lazy chunk；Vite 插件拷贝离线资源。
- [符号链接导致无限递归] → Rust 遍历防环。
- [PDF 扫描件无可选文字] → 用户只能看到图像，无法选中（预期行为）。
- [与视频工具能力重叠（mp4）] → 接受重叠；视频工具保留播放记忆，文件预览只做快速查看。

## Migration Plan

1. 新增工具卡片与视图，不修改现有工具行为。
2. 新增 Rust command 与前端 API，不影响现有 command 签名。
3. 依赖 `@file-viewer/*` 2.2.5；desktop build 通过 vue-tsc + vite build。
4. 若扫描或预览存在严重问题，可通过工具中心暂时隐藏卡片回滚（无需数据迁移）。

## Open Questions

（均已按实现结论关闭）

- 编辑源时改 title 不扫描，改 path/maxDepth 才扫描 — **已采用**。
- 单源扫描结果硬上限 — **MVP 未设硬上限，后续可按性能再加**。
