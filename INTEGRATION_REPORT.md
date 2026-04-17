# NRCS 交付物整合与依赖修复报告

**日期**: 2026-04-06 (WSL2 环境)
**任务**: 整合5个子代理交付物并修复编译阻塞
**状态**: ⚠️ 部分完成 - 编译 ICE 阻塞

---

## 1. 整合模块清单

| 模块 | 来源位置 | 目标位置 | 状态 | 备注 |
|------|----------|----------|------|------|
| **Crypto** (国密算法) | `D:/workspace/clawd/zeroclaw/crates/crypto` | `crates/crypto` | ✅ 已覆盖 | SM2/SM3/SM4 + AES-CBC 可插拔 |
| **P2P** (网络协议) | `D:/workspace/clawd/crates/p2p` | `crates/p2p` | ✅ 已复制 | WebSocket + 11 RPC handlers |
| **ORM** (数据层) | `D:/workspace/git/rust-nrcs/crates/orm` (已存在) | `crates/orm` | ✅ 无需改动 | 66表全覆盖，已修复 lib.rs 测试结构体 |
| **API 文档** | `D:/workspace/clawd/docs/` | `docs/` | ✅ 已复制 | Java API 清单、HTTP↔P2P 映射、实现报告 |
| **开发环境文档** | WSL2 手动创建 | `docs/quickstart_wsl.md` | ✅ 新增 | WSL2 环境完整指南 |

### 复制文件清单 (Docs)

```
docs/
├── api-inventory-report.md          # Java API 清单 (308 端点)
├── java-api-spec.json
├── java-api-spec.yaml
├── http-p2p-mapping-report.md      # HTTP↔P2P 映射分析
├── http-p2p-mapping.json
├── crypto-gm-implementation-report.md  # 国密算法实现报告
├── p2p-protocol-implementation-report.md  # P2P 协议报告
├── CRYPTO_CRATE_MODIFICATION_SUMMARY.md
├── quickstart_wsl.md                # WSL2 环境指南 (新)
└── ... (原有文档保留)
```

---

## 2. 依赖修复状态

### 问题诊断

- **ICE 触发位置**: `sqlx-core v0.6.3` 编译过程
- **编译器**: `rustc 1.96.0-nightly (e0e95a718 2026-04-04)`
- **错误类型**: Internal Compiler Error (thread 'rustc' panicked)
- **错误摘要**: `annotate_snippets::renderer::styled_buffer::StyledBuffer replace` - slice index out of bounds

### 已采取的临时措施

1. ✅ 子代理已移除 `block-cipher-trait` 依赖
2. ✅ 禁用 AES-CBC feature（代码中保留，但未启用）
3. ✅ 修复 orm `lib.rs` 中的 `#[cfg(test)]` 杂项解析错误

### 当前依赖树风险点

| Crate | 版本 | 风险说明 |
|-------|------|----------|
| `sqlx` | `0.6.3` | 已知 ICE 触发，尤其在 MSVC 下；WSL GNU 仍可能发生 |
| `aes` | `0.8` | 若启用 `cbc` feature 会引入 `block-cipher-trait` |
| `aes-gcm` | `0.10` (crypto/Cargo.toml) | 可能与其他 cipher 实现冲突 |
| `sm4-gcm` | `0.1.2` | 认证加密，但不依赖 block-cipher-trait |

**注意**: 即使 WSL/GNU 工具链，该 nightly 版本仍出现 ICE，说明是 Rust 编译器 bug。

---

## 3. 编译验证结果

### 最后一次检查

```bash
# WSL2 环境
cd /mnt/d/workspace/git/rust-nrcs
cargo check --all-targets
```

**结果**: ❌ 失败 (exit code 101)

**关键日志**:
```
error: the compiler unexpectedly panicked. This is a bug
query stack during panic:
[ICE stack trace omitted]

error: could not compile `sqlx-core` (lib)
```

**ICE 日志文件**:
```
/root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sqlx-core-0.6.3/rustc-ice-2026-04-06T07_12_19-387105.txt
```

---

## 4. 问题分析与建议

### 根因推测

1. **Rust nightly 编译器 bug**: ICE 发生在 `sqlx-core` 的代码生成/诊断阶段，与依赖树无关
2. **MSVC vs GNU**: WSL GNU 工具链同样触发，说明不是 `link.exe` 问题，而是纯编译器错误
3. **SQLx 0.6.3 复杂度**: 该版本宏和 proc-macro 代码量大，更容易暴露 ICE

### 短期解决路径

#### 方案 A: 升级 Rust 工具链 (推荐先试)

```bash
# WSL 中执行
rustup update nightly
# 检查是否已有更新版本修复该 ICE
rustup override set nightly  # 确保项目使用最新 nightly
cargo clean
cargo check --all-targets
```

#### 方案 B: 使用 stable/beta

某些项目在 stable 上不会 ICE，仅 nightly 有 regression。

```bash
rustup override set stable
cargo check --all-targets
```

#### 方案 C: 降级/调整 sqlx

如果升级 Rust 无效，考虑：
- 暂时降到 `sqlx = "0.5"` (可能影响 ORM 模型生成)
- 或使用 `sqlx` 的 `--features=macros` 最小化离线模式

#### 方案 D: 分模块隔离检查

定位具体哪个 crate 触发 ICE:

```bash
# 逐个检查
cargo check -p blockchain-types
cargo check -p crypto
cargo check -p orm
cargo check -p http-api
# 逐步增加，找出触发 ICE 的边界
```

### 长期修复

- 向 Rust 提交 ICE 报告 (附上日志和 minimal reproducible example)
- 考虑升级到 `sqlx 0.7` (当上游发布)
- 维持 WSL2 作为官方开发环境，避免 Windows MSVC 链接问题

---

## 5. 项目结构现状

```
rust-nrcs/
├── crates/
│   ├── account/          # ✅ 已有
│   ├── blockchain-types/ # ✅ 已有
│   ├── consensus/        # ✅ 已有
│   ├── contract/         # ✅ 已有
│   ├── crypto/           # ✅ 子代理整合 (覆盖)
│   ├── http-api/         # ✅ 已有
│   ├── orm/              # ✅ 已有 (lib.rs 已修复)
│   ├── p2p/              # ✅ 子代理整合 (复制)
│   └── tx-engine/        # ✅ 已有
├── docs/
│   ├── ... (原有文档)
│   ├── api-inventory-report.md
│   ├── http-p2p-mapping-report.md
│   ├── crypto-gm-implementation-report.md
│   ├── p2p-protocol-implementation-report.md
│   └── quickstart_wsl.md  # 🆕 WSL2 指南
├── Cargo.toml (workspace)
├── rust-toolchain.toml    # nightly 配置
└── config/
    ├── default.toml
    └── local.toml        # 本地配置 (需创建)
```

---

## 6. 下一步行动 (建议优先级)

| 优先级 | 任务 | 预期收益 | 负责 |
|--------|------|----------|------|
| P0 | 升级 Rust nightly & 重试 `cargo check` | 可能立即解决 ICE | 主会话 |
| P0 | 若 ICE 依旧，切换到 `stable` 工具链 | 绕过 nightly regression | 主会话 |
| P1 | 分模块隔离检查，定位触发 ICE 的 crate | 缩小问题范围 | 主会话 |
| P1 | 审核 `crypto/Cargo.toml`，移除未使用的 `aes-gcm` | 简化依赖树 | 主会话 |
| P2 | 提交 ICE 报告至 Rust 仓库 | 帮助上游修复 | 可选 |
| P2 | 验证所有单元测试 `cargo test --all-targets` | 确保功能正确 | 待 P0/P1 完成后 |

---

## 7. 结论

子代理的交付物整合工作 **已基本完成**，文件复制、结构修复均达成。当前唯一阻塞是 **Rust nightly 编译器的 ICE**，这与 block-cipher-trait 无关，属于工具链 bug。

建议立即尝试：
1. `rustup update nightly` → 重试 `cargo check`
2. 若无效 → `rustup override set stable` 再试
3. 仍失败 → 使用分模块隔离检查定位问题 crate

一旦编译通过，即可进入测试和互通验证阶段。

---

**报告生成**: 2026-04-06 15:30 GMT+8
**工作区**: `D:/workspace/git/rust-nrcs`
**Gateway**: OpenClaw (子代理任务中断后主会话接管)
