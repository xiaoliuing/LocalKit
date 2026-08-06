# LLM Prompt Template

这份模板用于让 AI 生成一组可以直接放进 `LocalKit` 的 Markdown 文档。

## Prompt

```text
请为主题 [主题名] 生成一组适合 LocalKit 的 Markdown 文档。

背景：
- 这些文档会被放进 LocalKit 的「文档阅读」工具里统一展示
- LocalKit 是本地资源工具集，文档阅读是其中的核心工具之一
- 读者主要是开发者
- 内容重点是设计说明、接入文档、教程文档或最佳实践

请遵守下面规则：

1. 输出一个目录，例如 `docs/[topic]/`
2. 目录下必须有一个 `README.md`，作为该主题的入口说明
3. 再生成若干正文文档，例如：
   - `01-xxx.md`
   - `02-xxx.md`
   - `03-xxx.md`
4. 每篇文档必须以 `# 一级标题` 开头
5. 正文主要使用 `##` 和 `###`
6. 代码块必须带语言标识
7. 如果文档之间有引用，使用相对 Markdown 链接
8. 不要只给提纲，要给完整正文
9. 风格保持直接、工程化、少空话

输出格式：

目录树：
docs/
└── [topic]/
    ├── README.md
    ├── 01-xxx.md
    ├── 02-xxx.md
    └── 03-xxx.md

文件内容：
=== docs/[topic]/README.md ===
[完整 Markdown]

=== docs/[topic]/01-xxx.md ===
[完整 Markdown]

=== docs/[topic]/02-xxx.md ===
[完整 Markdown]
```

## Minimal Example

```text
请为主题“FastAPI 项目设计文档”生成一组适合 LocalKit 的 Markdown 文档。

目标：
- 面向后端开发者
- 用于统一查看项目设计、接口分层、部署方式和常见问题
- 输出 1 个 README.md + 5 篇正文文档
- 语言为中文
```
