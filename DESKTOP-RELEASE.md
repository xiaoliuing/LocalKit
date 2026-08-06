# LocalKit Desktop Release

这份文档只说明桌面端发布。

Web 示例站继续使用 GitHub Pages。

桌面端发布已经拆成独立 GitHub Action：

- 工作流文件：`.github/workflows/release-desktop.yml`
- 触发方式：
  - 推送 tag：`desktop-v0.31.10`
  - GitHub Actions 页面手动执行 `Release Docs Atlas Desktop`

## 发布前需要做什么

1. 确认桌面端版本号已经更新

- `apps/desktop/package.json`
- `apps/desktop/src-tauri/Cargo.toml`
- `apps/desktop/src-tauri/tauri.conf.json`

2. 确认本地能通过基本检查

```bash
pnpm --filter @docs-atlas/desktop build:web-shell
cd apps/desktop/src-tauri && cargo check
```

3. 当前桌面端发布不再接入 Apple 签名和 notarization。

说明：

- GitHub Action 会直接构建未签名的 macOS 安装包
- 不再依赖任何 `APPLE_*` secrets
- 也不再导入 macOS keychain 证书
- macOS 用户下载后仍可能看到“已损坏，无法打开”或需要手动放行

## 推荐发布方式

推荐用 tag 触发正式发布：

```bash
git tag desktop-v0.31.10
git push origin desktop-v0.31.10
```

工作流会：

- 在 macOS / Windows / Ubuntu 上分别构建桌面端安装包
- 自动创建或更新 GitHub Release
- 把构建产物上传到对应 Release

## 手动发布

如果只是想先验一次构建链路，可以在 GitHub Actions 页面手动执行：

- `draft = true`
- `prerelease = true` 或 `false`

这更适合内部验证，不会影响 Pages 示例站。

## 和 Web 发布的关系

- `deploy-pages.yml` 只负责 Web 示例站
- `release-desktop.yml` 只负责桌面端安装包
- 以后桌面端代码提交不会因为 README 或根版本号变动自动触发 Pages 部署
