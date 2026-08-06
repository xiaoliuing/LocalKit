# CLAUDE.md — LocalKit 项目规范

> 本文件是 Claude Code 的项目级规则文件。本项目的完整规范还包含：
> - [AGENTS.md](./AGENTS.md) — 技术范围、内容规则、路由、UI、工程规范

---

## 项目概述

LocalKit 是一个本地资源工具集，桌面端通过工具中心组织文档阅读、视频播放等工具；Web 端为文档阅读的静态发布延伸。使用 Vue 3 + TypeScript + Vite 8 构建。

**技术栈**：Vue 3 / TypeScript / Vite 8 / vite-ssg / markdown-it / highlight.js

---

## 项目结构

```
src/
├── assets/main.css          # 全局样式和主题变量（CSS Custom Properties）
├── app/                     # App 级初始化逻辑
├── components/
│   ├── layout/              # SiteHeader、AppShell 等布局组件
│   └── docs/                # DocContent、DocToc、DocPager、DocImagePreview、
│                            # DocsSidebar、DocsSidebarNode、DocsSearchPanel
├── composables/
│   ├── useTheme.ts          # 主题系统：模式（light/dark/system）+ 主题色
│   ├── useDocsSearch.ts     # 搜索：全文搜索、结果排序、键盘交互
│   ├── useDocsCatalog.ts    # 目录：左侧文档树、元数据
│   ├── useDocsContent.ts    # 正文：按需加载文档详情 JSON
│   ├── useDocRoute.ts       # 路由：解析当前分类和文档
│   └── useActiveHeadings.ts # 目录：滚动命中标题跟踪
├── views/
│   ├── HomeView.vue         # 首页
│   ├── SectionView.vue       # 专题页
│   └── DocView.vue          # 文档详情页
├── main.ts                  # 应用入口
└── types/                   # 类型定义
```

---

## 主题系统规范

### 变量体系（src/assets/main.css）

主题系统基于 **CSS Custom Properties** 实现，通过 `data-theme`（模式）和 `data-accent`（主题色）两层叠加：

```
:root                          # 默认 light 模式基础变量
:root[data-theme="dark"]       # dark 模式变量
:root[data-accent="xxx"]      # 主题色变量（覆盖 accent 相关变量）
:root[data-theme="dark"][data-accent="xxx"]  # dark + 主题色
```

### 变量层级说明

| 变量前缀 | 用途 |
|---------|------|
| `--theme-mist-rgb` | 背景渐变起点色 |
| `--theme-panel-rgb` | 背景渐变终点色 |
| `--theme-shadow-rgb` | 阴影色 |
| `--color-accent` | 主强调色 |
| `--color-accent-rgb` | 主强调色（RGB 格式，用于 alpha 合成） |
| `--color-accent-deep` | 深强调色（用于链接 hover 等） |
| `--color-ink` | 主文本色 |
| `--color-muted` | 次要文本色 |
| `--color-soft` | 柔和文本色 |
| `--color-line` | 边框色 |
| `--surface-*` | 各种表面材质（卡片、代码、引用等） |

### 新增主题色的正确方式

新增一个主题色需要修改两个文件：

**1. `src/composables/useTheme.ts`**：
- `ThemeAccentId` 类型添加新的 id
- `themeAccents` 数组添加新条目 `{ id, label, color }`

**2. `src/assets/main.css`**：
- 添加 `[data-accent="xxx"]` light 模式变量覆盖
- 添加 `[data-theme="dark"][data-accent="xxx"]` dark 模式变量覆盖

**原则**：不修改 base `:root` 和 `:root[data-theme="dark"]` 的变量，只通过 accent 层叠加。

---

## 提交规范

### 格式

```
<type>: <简短描述>

[可选的详细说明]
```

### Type 枚举

| type | 使用场景 |
|------|---------|
| `feat` | 新增功能 |
| `fix` | 修复 bug |
| `style` | 样式调整（不影响功能的视觉变化） |
| `docs` | 文档变更 |
| `refactor` | 重构（既不是 feat 也不是 fix） |
| `chore` | 工具、构建、依赖更新等 |

### 示例（来自真实历史）

```
feat: add pure-white theme accent
style: stabilize sidebar header and scrollbar
fix: clarify source-level doc hierarchy
chore: remove docs from git history and ignore them
```

### 注意事项

- 提交信息用英文写，作者信息用全局 git 配置（xiaoliuing <2303686241@qq.com>）
- 不要在提交信息中包含 `Co-Authored-By: Claude...`（除非确实需要）
- 每次完成改动后默认自动 commit 并 push
- 版本号递增规则见 AGENTS.md

---

## 构建与验证

### 常用命令

```bash
pnpm dev      # 开发启动
pnpm build    # 生产构建
```

### 必须验证构建的场景

修改以下内容后**必须跑一次 `pnpm build`** 确认正常：

- 文档索引逻辑（`scripts/docsData.ts` 相关）
- 路由设计
- 搜索逻辑
- 主题系统
- Header 组件
- Markdown 渲染逻辑

### 关键检查项

- 构建产物中包含各路由对应的静态 HTML
- `__docs/` 目录下有 `search-index.json` 和 `content/*.json`
- 主题切换后 CSS 变量正确应用

---

## Git 历史重写规范

本项目曾使用 `git filter-repo` 从历史中移除 `PRD.md`、`requirements.md` 等文档文件。

**执行历史重写的场景**：必须同时满足以下条件：
1. 文件已经添加到 `.gitignore`
2. 有敏感信息泄露或大文件误传
3. 已经备份仓库

**操作后必须执行**：
1. 重新添加 remote
2. 用 `--force` 推送
3. 确认推送成功

---

## 组件开发规范

### 薄视图、厚 composable 原则

视图组件（`views/`）只负责：
- 布局和样式
- 调用 composable 获取数据
- 事件透传

复杂逻辑全部放到 `composables/`：
- 数据获取和缓存
- 状态管理
- 副作用处理

### 不引入 Pinia

本项目使用 Vue 3 Composition API，状态由 composables 管理，不引入 Pinia。

### 样式规范

- 全局样式在 `src/assets/main.css`
- 不在手写死颜色，优先使用 CSS 变量
- 主题相关颜色必须通过变量引用，不能写死 hex 值
- 响应式断点参考 AGENTS.md

---

## Markdown 文档规则

### 内容来源

- 默认读取 `./docs/`
- 支持 `config.yaml` 配置多文档源和嵌套分组

### 渲染规则

- HTML 必须经过 sanitize-html 清洗
- 相对 `.md` 链接自动改写为站内路由
- `README.md` 链接改写为专题页
- `h2/h3` 进入右侧目录
- 首个 `h1` 在正文中移除（避免和页面标题卡片重复）

---

## 常见任务参考

| 任务 | 参考文档 |
|------|---------|
| 新增主题色 | 本文件「主题系统规范」章节 |
| 添加新组件 | `src/components/docs/` 或 `src/components/layout/` |
| 修改文档索引 | `scripts/docsData.ts` |
| 添加搜索能力 | `src/composables/useDocsSearch.ts` |
| 修改文档路由 | `vite.config.ts` + `src/views/` |
| 样式调整 | `src/assets/main.css` |
