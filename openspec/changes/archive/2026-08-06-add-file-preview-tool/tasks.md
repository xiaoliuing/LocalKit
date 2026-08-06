## 1. 类型与 API 基础

- [x] 1.1 定义预览源、树节点、扫描结果与持久化状态的 TypeScript 类型（对齐 spec 中的源扁平 + 树形节点语义）。
- [x] 1.2 新增 `api/filePreview.ts`，封装 `scanPreviewDirectory`、`readPreviewTextFile`、资源 URL 转换与路径校验的 Tauri 调用。
- [x] 1.3 在 Rust 侧定义 serde payload（`PreviewTreeNode`、`PreviewDirectoryScanPayload`）并实现 `scan_preview_directory(path, max_depth)` 命令，支持层级限制与忽略目录（`.git`、`node_modules` 等）。
- [x] 1.4 实现 `read_preview_text_file(path)`，对 legacy 纯文本尝试 UTF-8 / UTF-16 / GBK 解码。

## 2. Rust 扫描逻辑

- [x] 2.1 实现按 `max_depth` 递归遍历；`max_depth = 0` 表示不限制，并防止符号链接导致无限循环。
- [x] 2.2 扫描结果包含 folder/file 节点、相对路径、大小、修改时间；文件不过滤扩展名（不可预览文件也进入树）。
- [x] 2.3 为无效路径、非目录、空目录返回可读 message，供前端展示源级状态。
- [x] 2.4 为扫描逻辑添加聚焦 Rust 测试（层级截断、忽略目录、空目录、无效路径）。

## 3. 状态管理与 composable

- [x] 3.1 新增 `useDesktopFilePreviewLibrary`，管理源列表、扫描状态、当前文件、展开状态与 localStorage 持久化（`docs-atlas.desktop.file-preview.v1`）。
- [x] 3.2 实现添加源、编辑源（title/path/maxDepth）、删除源、重新扫描；改 title 不触发扫描，改 path 或 maxDepth 触发重新扫描。
- [x] 3.3 实现源展开/折叠、文件夹节点展开/折叠、文件选择与恢复上次浏览状态。
- [x] 3.4 复用或封装 `pickFolderPath` 选择目录流程（与 workspace/video 一致）。
- [x] 3.5 实现预览 view state 记忆（`viewStateMemory`、`rememberViewState`、`currentViewStateMemory`）。

## 4. UI 组件

- [x] 4.1 新增 `DesktopFilePreviewSourceDialog`（显示名称、目录路径、扫描层级），默认 title 为目录 basename。
- [x] 4.2 新增 `DesktopFilePreviewSourceGroup`，扁平展示源 + 内嵌文件树，支持展开/折叠与重新扫描。
- [x] 4.3 新增 `DesktopFilePreviewTreeNode`，渲染 folder/file 节点并区分点击行为（文件夹切换展开，文件触发预览）。
- [x] 4.4 新增 `DesktopFilePreviewToolView`：左栏 library + 右栏 `<file-viewer>`，布局与交互参考 `DesktopVideoToolView`（含返回工具中心、空状态、扫描中状态）。
- [x] 4.5 新增 `DesktopFilePreviewFormatDialog` 与 `filePreviewFormats.ts`，展示支持/不支持格式说明。

## 5. File Viewer 集成

- [x] 5.1 引入 `@file-viewer/vue3` 与 `@file-viewer/vue3/dist/file-viewer3.css`，在 `main.ts` 注册插件。
- [x] 5.2 配置 `@file-viewer/vite-plugin`：`autoPresets: ['lite', 'office', 'engineering']`，`formats: ['eml', 'msg', 'mbox']`，`copyAssets: true`。
- [x] 5.3 封装 `getDesktopFilePreviewOptions()`：`locale: 'zh-CN'`、`theme: 'system'`、`styleIsolation: 'none'`、工具栏选项。
- [x] 5.4 使用 `convertFileSrc` 将选中文件绝对路径传给 `<file-viewer>`；legacy 纯文本经 Rust 解码后以 `File` 传入。
- [x] 5.5 处理加载完成、`view-state-change` 与 `applyViewState` 恢复预览位置。
- [x] 5.6 确认预览区样式与 LocalKit 浅色/暗色主题兼容。

## 6. 文本选中与编码体验

- [x] 6.1 新增 `filePreviewSelection.css` 与 `filePreviewTextSelection.ts`，修复 PDF / Word 预览区无法选中文本。
- [x] 6.2 在 load-complete 与文件切换时绑定 `MutationObserver` 持续修复 PDF `enablePermissions` 与 canvas 遮挡。

## 7. 工具中心接入

- [x] 7.1 在 `DesktopToolHubView` 新增「文件预览」卡片（id: `file-preview`）并更新首页摘要文案。
- [x] 7.2 在 `DesktopAppShell` 扩展 `primaryView: "file-preview"`、`handleOpenTool` 路由与 `isToolView` 判断。
- [x] 7.3 挂载 `DesktopFilePreviewToolView` 并接入 `backToTools` 事件。

## 8. 验证

- [x] 8.1 运行桌面端类型检查：`cd apps/desktop && node ./node_modules/vue-tsc/bin/vue-tsc.js --noEmit`。
- [x] 8.2 运行桌面端 Web shell 构建：`cd apps/desktop && node ./node_modules/vite/bin/vite.js build`。
- [x] 8.3 手动验证：添加多源、自定义名称、修改扫描层级、树形展开、PDF/Office/图片/文本预览、unsupported fallback、预览位置恢复、文本选中、持久化恢复、返回工具中心。
