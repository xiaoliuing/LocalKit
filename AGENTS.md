# Docs Atlas Agent Guide

## Product Context

- `Docs Atlas` 是本地 Markdown 聚合阅读与知识库管理工具。
- 当前以桌面端为主，Web 端用于静态预览、分享和 GitHub Pages 部署。
- 桌面端基于 `Tauri 2 + Vue 3 + TypeScript + Vite 8`，Web 端基于 `Vue 3 + TypeScript + Vite 8 + vite-ssg`。
- 项目是 monorepo，根目录是 `docs-cms/`。

## Directory Map

- `apps/desktop/`：桌面端主应用。
- `apps/web/`：Web 静态文档站。
- `packages/shared-types/`：共享类型。
- `packages/workspace-db/`：桌面端本地数据层。
- `docs/`：项目自带示例文档，也是默认文档源。
- `open-docs/`：产品规划、发布、设计方案等开放文档。
- `scripts/release/`：桌面端发布脚本和 updater manifest 生成脚本。

## Working Rules

- 使用 Vue 3 Composition API 和 `<script setup lang="ts">`。
- 复杂业务逻辑优先放到 `composables/`，页面和组件保持薄。
- 不引入 Pinia，除非出现明确的跨域复杂状态需求。
- 不手改 `dist/`、`node_modules/`、`.pnpm-store/`。
- 不回退用户已有改动；改动前先用 `git status --short` 看工作区。
- UI 修改要遵守现有桌面端风格，不做大面积无关重构。

## Desktop App

- 桌面端是主产品形态，功能包括文档仓库、文档源、阅读、搜索、编辑、主题、视频工具和应用设置。
- 桌面端不依赖 `config.yaml`，文档仓库和文档源由本地 SQLite 数据维护。
- 首次启动应有默认文档仓库，并指向打包资源中的示例 `docs/`。
- 文档内容支持直接编辑并保存回本地 Markdown 文件。
- 主题支持浅色、暗色、跟随系统、多预设主题色和自定义主题色。
- 自定义主题色使用 `@simonwep/pickr` 的 `Nano` 主题，入口在外观设置页。
- 视频工具使用 `artplayer`，需要记住视频目录、展开状态、上次播放视频、播放位置和播放速度。

## Web App

- Web 端是只读静态文档站，默认读取 `./docs`。
- Web 端可通过 `config.yaml` 聚合多个文档源和嵌套分组。
- Web 输出为 SSG，文档正文和搜索索引应拆成静态 JSON，不要整体打入主 JS。
- 路由保持：
  - `/`
  - `/section/:section`
  - `/docs/:slug(.*)`

## Content Rules

- 一级目录视为专题。
- `README.md` 是专题入口。
- 根目录下直接放置的 Markdown 作为独立文档显示。
- 目录排序采用目录优先、`README.md` 优先、自然数字排序。
- 相对图片按文档所在目录解析。
- Web 端多文档源必须使用 source namespace，避免静态资源和 slug 冲突。

## Markdown Rules

- Web 端 Markdown 索引入口在 `scripts/docsData.ts`。
- Markdown 渲染必须支持标题、表格、代码块、图片、相对链接、mermaid。
- 相对 `.md` 链接改写为站内路由。
- `README.md` 链接改写到对应专题入口。
- `h2 / h3` 进入右侧大纲。
- 首个 `h1` 不应在正文里重复展示，避免和文章标题重复。

## Styling Rules

- 颜色、背景、边框、滚动条优先使用主题变量。
- 不写死主题色；需要新增颜色时，优先扩展主题变量或偏好状态。
- 桌面端 title bar、应用背景、侧栏、文章区、设置页都要兼容浅色和暗色主题。
- Markdown 内容样式要以阅读舒适为优先，代码块、表格、blockquote、图片要单独检查。

## Verification

- 桌面端前端检查：
  - `cd apps/desktop && node ./node_modules/vue-tsc/bin/vue-tsc.js --noEmit`
  - `cd apps/desktop && node ./node_modules/vite/bin/vite.js build`
- Web 端检查：
  - `pnpm build:web`
- 共享包检查：
  - `pnpm check:shared-types`
  - `pnpm check:workspace-db`
- 如果 `pnpm` 因版本签名或网络问题失败，可优先使用项目本地 `node_modules` 中的脚本做前端验证。

## Release

- 桌面端发布使用根目录脚本：
  - `pnpm release:desktop patch --message "release(desktop): desktop-vX.Y.Z"`
  - `pnpm release:desktop minor --message "release(desktop): desktop-vX.Y.Z"`
  - `pnpm release:desktop major --message "release(desktop): desktop-vX.Y.Z"`
- 发版脚本会同步更新 `apps/desktop/package.json`、`tauri.conf.json`、`Cargo.toml`、`Cargo.lock`，并提交、打 tag、推送。
- 版本规则：
  - `patch`：修复、样式、文档、小功能调整。
  - `minor`：新增功能且不破坏兼容。
  - `major`：破坏性变更、配置或数据结构不兼容。
