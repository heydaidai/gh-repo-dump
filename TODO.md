# TODO — gh-repo-dump

## 已完成
- [x] CLI 参数解析 (clap 4)
- [x] GitHub API 并行请求 (9 端点)
- [x] JSON 输出 (stdout / file)
- [x] Nix flake 开发环境
- [x] 编译 + fmt + clippy 通过
- [x] README.md + .gitignore
- [x] 推送到 GitHub (heydaidai/gh-repo-dump)
- [x] 清理额外贡献者，squash 为单一干净 commit

## 未完成 / 待优化
- [ ] 单元测试
- [ ] CI/CD (GitHub Actions)
- [ ] 考虑支持 GitHub Enterprise (自定义 API base URL)
- [ ] 考虑支持 org 级别批量 dump

## 已知限制
- 无 token 时 API 限速 60 req/h
- 分页上限 3 页 × 30 条，超大仓库可能不完整
- 不支持 GraphQL API

## 下一步建议
- 添加基本单元测试覆盖 API 解析逻辑
- 设置 GitHub Actions 做 CI (build + test + clippy + fmt)
