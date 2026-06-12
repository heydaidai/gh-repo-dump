## 目标
构建一个 Rust CLI 工具 `gh-repo-dump`，将 GitHub 仓库的详细信息提取为结构化 JSON。
支持通过 GitHub API 并行拉取 repo 元数据、releases、tags、languages、contributors、branches、commits、readme、license 等信息。

## 技术栈/依赖
- Rust 2021 edition
- clap 4（CLI 参数解析，支持 env 绑定）
- reqwest 0.13（HTTP 客户端，rustls）
- tokio（异步运行时 + 并发请求）
- serde / serde_json（JSON 序列化）
- anyhow（错误处理）
- Nix flake（开发环境：cargo/rustc/rustfmt/clippy/openssl）

## 编码风格
- 标准 Rust 风格，cargo fmt / cargo clippy
- 使用 anyhow 统一错误处理
- 并发请求用 tokio::spawn 并行化，跨 9 个 API 端点
- 分页请求最多 3 页、每页 30 条
- 超时：每个端点 30s，整体 timeout * 3

## 架构决策
- 单文件 src/main.rs（工具规模小，无需拆模块）
- 不使用 GitHub SDK（octocrab），直接调 REST API 保持依赖精简
- Bearer token 认证，支持 GITHUB_TOKEN 环境变量
- 输出仅 JSON 格式，支持 stdout 或文件写入
- pagination 上限 3 页，覆盖绝大多数仓库（>90 commits/releases 不常见）

## 进度与下一步
- [x] CLI 参数解析
- [x] GitHub API 客户端
- [x] 并行端点请求
- [x] JSON 输出
- [x] Nix 开发环境
- [x] 编译验证通过
- [x] README.md
- [ ] 单元测试
- [ ] 集成测试（mock HTTP）
- [ ] CI/CD
