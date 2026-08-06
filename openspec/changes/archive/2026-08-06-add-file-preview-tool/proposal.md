## Why

LocalKit 已定位为本地资源工具集，但当前缺少一个通用的本机文件预览入口。用户在本机积累了 PDF、Office、图片、代码、压缩包等多种格式，往往需要在 Finder 与多个应用之间切换。桌面端已接入 `@file-viewer/vue3` 与对应 preset / Vite 插件，适合在工具中心新增「文件预览」工具，让用户添加多个本地文件夹、按树形浏览并按层级扫描后直接预览支持的文件。

## What Changes

- 在桌面端工具中心新增「文件预览」工具入口，并从 `DesktopAppShell` 接入独立工具视图。
- 支持添加多个本地文件夹作为预览源；源之间扁平并列展示。
- 每个源支持自定义显示名称，默认使用文件夹 basename；支持编辑名称、路径与扫描层级后重新扫描。
- 展开某个源后，以树形结构展示该目录下扫描到的文件夹与文件；点击文件在右侧使用 `@file-viewer/vue3` 预览。
- 每个源支持配置最大扫描层级（含「不限制」选项）；超出层级的子目录不参与扫描。
- 新增 Rust 侧目录扫描命令，返回与前端对齐的树形节点结构（含路径、大小、修改时间、节点类型）。
- 新增 Rust 侧 legacy 纯文本解码命令（UTF-8 / UTF-16 / GBK），解决部分 `.txt` 等文件乱码。
- 新增 composable 与 localStorage 持久化，记住已添加源、展开状态、上次预览文件与预览位置（页码/滚动）。
- 集成 `@file-viewer/vue3`，通过 Vite 插件启用 `lite`、`office`、`engineering` preset 及邮件格式；locale 使用 `zh-CN`，`styleIsolation: 'none'` 以便宿主样式覆盖。
- 提供格式说明对话框，列出已支持扩展名与暂不支持分组。
- 修复 PDF / Word 等文档在预览区无法选中文本的问题（CSS + DOM 修复）。
- 树中展示目录下所有扫描到的文件；不可预览格式在预览区走 fallback，而非从树中隐藏。

## Capabilities

### New Capabilities

- `file-preview-tool`: 管理 LocalKit 桌面端文件预览工具的目录源、分层扫描、树形浏览、多格式文件预览、预览位置记忆与状态持久化行为。

### Modified Capabilities

- 无。

## Impact

- 影响桌面端工具中心卡片与摘要文案、`DesktopAppShell` 路由与工具视图编排。
- 新增 `DesktopFilePreviewToolView`、`DesktopFilePreviewFormatDialog`、相关 composable、API 封装与树形节点组件；布局参考视频工具（左栏库 + 右栏预览）。
- 新增 Tauri command：`scan_preview_directory`、`read_preview_text_file`。
- 依赖 `@file-viewer/vue3`、`@file-viewer/preset-*`、`@file-viewer/vite-plugin`；Vite 配置 `fileViewerRenderers` 并拷贝离线 Worker / WASM 资源。
- 不影响 Web 端、文档阅读工具、视频工具与 Agent 会话清理工具的现有行为。
- 不引入 Pinia；复杂状态放在 composable 中。
