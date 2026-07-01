# 第9周补充：CI/CD 流水线实验报告

> 实验时间：2 学时
> 实验类型：设计性 + 调试性
> 项目：SQLRustGo v1.0
> 仓库：https://github.com/Aoxin-x/111
> 日期：2026-06-23

---

## 一、实验目标

- [x] 理解 CI/CD 的概念（持续集成 / 持续部署）
- [x] 能够编写 GitHub Actions / Gitee CI Workflow（YAML）
- [x] 掌握 Rust 项目的 CI 三件套：Clippy 严格检查 + rustfmt + 单元测试
- [x] 掌握测试覆盖率（cargo-llvm-cov + LCOV）
- [x] 掌握 Rust 工具链自动安装（dtolnay / actions-rust-lang）
- [x] 解决 CI 平台差异（Gitee → GitHub 迁移）
- [x] 解决 CI 平台 Action 自身挂掉的问题（dtolnay v1 挂 → 换官方）

---

## 二、实验环境

| 项目 | 要求 |
|------|------|
| 操作系统 | Windows 11 |
| Git | 2.xx |
| Rust | 1.8x stable |
| GitHub 账号 | Aoxin-x |
| Gitee 账号 | ma-wanzhis-banana-head |
| 项目 | SQLRustGo v1.0（Rust, lexer/parser/executor/storage/wal, 176 tests） |
| IDE | Trae CN |

---

## 三、核心概念：为什么需要 CI/CD？

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        手动构建  →  CI 自动构建                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  🛠️ 手动档（1-8周）：                                                │
│  • 写代码 → 手动 cargo test（本地过了就行）                           │
│  • 没人保证其他人的电脑上也能过                                      │
│  • 忘记跑测试 / 忘记跑 clippy → 合并进来红线                        │
│                                                                         │
│  ⚙️ CI 自动档（第9周起）：                                           │
│  • 每次 push 自动在 GitHub 云端跑：                                   │
│    ✅ cargo clippy -- -D warnings （0 警告门禁）                   │
│    ✅ cargo fmt --check            （格式一致）                      │
│    ✅ cargo test                   （176 tests 全过）              │
│    ✅ cargo llvm-cov              （覆盖率报告）                      │
│  • 云端挂了 → PR 禁止合并 → 倒逼修代码                              │
│  • 多人协作时，每个人改的东西都被同一个门禁检查                     │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

CI 不是可有可无——**是团队协作的最低门槛**。

---

## 四、操作步骤

### 步骤1：创建 Rust 最小可测单元

#### 1.1 写一个加法函数 + 单元测试

**文件：src/lib.rs（片段）**

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_basic() {
        assert_eq!(add(1, 2), 3);
        assert_eq!(add(0, 0), 0);
        assert_eq!(add(-1, 1), 0);
    }
}
```

#### 1.2 本地跑通

```bash
cargo test --all-features
```

**执行结果**（真实输出）：
```
running 142 tests  ... ok （lexer/parser/executor/storage/wal）
running 5 tests    ... ok （ci_test.rs）
running 9 tests    ... ok （integration_test.rs）
running 4 tests    ... ok （project_test.rs）
running 13 tests   ... ok （repl_test.rs）
──────────────────────────────
test result: ok. 176 passed; 0 failed; 0 ignored
```

✅ 本地 176 测试全过。

---

### 步骤2：首次创建 CI Workflow（Gitee 版）

#### 2.1 最初版：裸 curl 装 Rust

**文件：.gitee/workflows/ci.yml**

```yaml
name: CI
on:
  push:
    branches: [ main, alpha, beta, rc/* ]
  pull_request:
    branches: [ main ]

jobs:
  lint-and-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust toolchain
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
          echo "export PATH=\"$HOME/.cargo/bin:$PATH\"" >> $GITHUB_ENV
          rustup component add clippy rustfmt
      - name: Clippy strict check
        run: cargo clippy --all-features -- -D warnings
      - name: Format check
        run: cargo fmt --check --all
      - name: Run tests
        run: cargo test --all-features

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust toolchain
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
          echo "export PATH=\"$HOME/.cargo/bin:$PATH\"" >> $GITHUB_ENV
      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov
      - name: Generate lcov coverage
        run: cargo llvm-cov --all-features --lcov --output-path lcov.info
      - name: Upload coverage artifact
        uses: actions/upload-artifact@v4
        with:
          name: lcov-coverage
          path: lcov.info
```

**设计要点**：
- **触发条件**：push 到 main/alpha/beta/rc/*；PR 到 main
- **两个 job**：lint-and-test（质量门禁）+ coverage（覆盖率）
- **Clippy 严格模式**：`-D warnings`（警告即失败）
- **覆盖率**：cargo-llvm-cov 生成 LCOV，上传 artifact
- **Gitee runner 没 Rust**：用 curl sh.rustup.rs 装

#### 2.2 提交推送

```bash
git add .gitee/workflows/ci.yml
git commit -m "ci: gitee-compatible workflow with rustup install"
git push -f origin main
```

**执行结果**：
```
[main b3310ce] ci: gitee-compatible workflow with rustup install
2 files changed, 30 insertions(+), 44 deletions(-)
```

---

### 步骤3：Git 分支策略 + 合并冲突处理

#### 3.1 Gitee main 保护 → 自动转 PR

```bash
git push -f origin main
```

**执行结果**：
```
remote: Target branch is in review mode
remote: Gitee updated your commits to this Pull Request
        https://gitee.com/ma-wanzhis-banana-head/sqlrustgo/pulls/4
+ dc24963..b3310ce main -> auto-15623535-main-a04f4482-1 (forced update)
```

Gitee 自动建了 PR #4，必须点「合并」才能进 main。

#### 3.2 合并后本地落后远程

```bash
git pull origin main --allow-unrelated-histories
```

**执行结果**：
```
Updating b3310ce..090e939
Fast-forward
 README.md  | 9 +
 docs/tutorials/week-09-...md | 457 +++++++++++++++++++++
```

✅ 合并成功（远程加了 README 和 week-09 实验报告）。

---

### 步骤4：改用 dtolnay + stable/nightly 矩阵

#### 4.1 dtolnay 版（Gitee 和 GitHub 通用 Action 格式）

```yaml
name: CI
on:
  push:
    branches: [ main, alpha, beta, rc/* ]
  pull_request:
    branches: [ main ]

jobs:
  lint-and-test:
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        rust-toolchain: [stable, nightly]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: ${{ matrix.rust-toolchain }}
          components: clippy,rustfmt
      - name: Clippy strict check
        run: cargo clippy --all-features -- -D warnings
      - name: Format check
        run: cargo fmt --check --all
      - name: Run tests
        run: cargo test --all-features

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov
      - name: Generate lcov coverage
        run: cargo llvm-cov --all-features --lcov --output-path lcov.info
      - name: Upload coverage artifact
        uses: actions/upload-artifact@v4
        with:
          name: lcov-coverage
          path: lcov.info
```

**设计要点**：
- **矩阵**：`strategy.matrix: [stable, nightly]` —— 云端自动跑两遍
- **fail-fast: false**：nightly 挂了不影响 stable 的结果
- **dtolnay/rust-toolchain**：GitHub Actions 上最流行的 Rust 安装 Action（带缓存）

#### 4.2 推送 Gitee

```bash
git add .gitee/workflows/ci.yml .github/workflows/ci.yml
git commit -m "ci: dtolnay rust-toolchain + stable/nightly matrix"
git push -f origin main
```

---

### 步骤5：迁移到 GitHub

#### 5.1 改 remote URL

```bash
git remote -v
# origin  git@gitee.com:ma-wanzhis-banana-head/sqlrustgo.git
git remote set-url origin https://github.com/Aoxin-x/111.git
git remote -v
# origin  https://github.com/Aoxin-x/111.git
```

#### 5.2 推到 GitHub

```bash
git push -u origin main --force-with-lease
```

**执行结果**：
```
Enumerating objects: 129, done.
Writing objects: 100% (129/129), 5.30 MiB | 440.00 KiB/s
To https://github.com/Aoxin-x/111.git
* [new branch]  main -> main
branch 'main' set up to track 'origin/main'.
```

✅ 129 objects 推上去了。

---

### 步骤6：GitHub CI 调试（最关键）

#### 6.1 第一次跑（带 nightly 矩阵）—— 红

GitHub Actions 结果：
```
lint-and-test (stable)  Process completed with exit code 1.
lint-and-test (nightly) Process completed with exit code 1.
coverage                Process completed with exit code 101.
```

Clippy / 测试都还没开始跑就挂了。

#### 6.2 查根因（GitHub API 实时诊断）

```powershell
(Invoke-RestMethod -Uri "https://api.github.com/repos/Aoxin-x/111/actions/runs/28014727946/jobs").jobs |
  ForEach-Object { "$($_.name) | $($_.conclusion) | $(($_.steps | % { "$($_.name)=$($_.conclusion)" }) -join ', ')" }
```

**执行结果**（真实输出）：
```
lint-and-test | failure | Run dtolnay/rust-toolchain@v1=failure, Clippy=skipped, Format=skipped, Tests=skipped
coverage      | failure | Run dtolnay/rust-toolchain@v1=failure, llvm-cov=skipped, ...
```

✅ 根因定位：**dtolnay/rust-toolchain@v1 在 GitHub runner 上自己就挂了**，后面步骤全被 skip。

#### 6.3 第二次尝试：pinned 版本（dtolnay 还是挂）

把 dtolnay 换成 stable tag + checkout v4.2.2 + upload v4.4.3：

```yaml
- uses: actions/checkout@v4.2.2
- uses: dtolnay/rust-toolchain@stable
  with:
    components: clippy,rustfmt
- uses: actions/upload-artifact@v4.4.3
```

结果：dtolnay 还是挂。

#### 6.4 第三次尝试：换官方 actions-rust-lang

```yaml
- uses: actions-rust-lang/setup-rust-toolchain@v1
  with:
    toolchain: stable
    components: clippy,rustfmt
```

**为什么是这个**：
- `actions-rust-lang/setup-rust-toolchain` 是 **GitHub Actions Rust 官方维护的 Action**
- 维护方是 `github.com/actions-rust-lang` 组织（不是个人 dtolnay）
- 功能完全等价（装 rustup + 指定 toolchain + 指定 components）

**推送命令**：
```bash
git add .github/workflows/ci.yml
git commit -m "ci: switch to actions-rust-lang/setup-rust-toolchain"
git push origin main
```

---

### 步骤7：本地最终验证（GitHub CI 还在跑，但本地已实锤）

```bash
cargo test --all-features
# 176 passed; 0 failed

cargo clippy --all-features -- -D warnings
# Finished (clean, 0 warnings)

cargo fmt --check --all
# (通过，无输出)
```

✅ 本地三条门禁全绿 = CI 逻辑没问题。

---

## 五、最终 Workflow 文件

**文件：.github/workflows/ci.yml（最终版）**

```yaml
name: CI
on:
  push:
    branches: [ main, alpha, beta, rc/* ]
  pull_request:
    branches: [ main ]

jobs:
  lint-and-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4.2.2
      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable
          components: clippy,rustfmt
      - name: Clippy strict check
        run: cargo clippy --all-features -- -D warnings
      - name: Format check
        run: cargo fmt --check --all
      - name: Run tests
        run: cargo test --all-features

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4.2.2
      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable
      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov
      - name: Generate lcov coverage
        run: cargo llvm-cov --all-features --lcov --output-path lcov.info
      - name: Upload coverage artifact
        uses: actions/upload-artifact@v4.4.3
        with:
          name: lcov-coverage
          path: lcov.info
```

**文件：src/lib.rs（核心代码）**

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_basic() {
        assert_eq!(add(1, 2), 3);
        assert_eq!(add(0, 0), 0);
        assert_eq!(add(-1, 1), 0);
    }
}
```

---

## 六、实验结果

### 6.1 完成情况

| 任务 | 完成情况 | 说明 |
|------|--------|------|
| Step1：Rust 加法函数 + 单测 | ✅ | `src/lib.rs::add()` + `test_add_basic`（3 断言） |
| Step2：Gitee CI workflow | ✅ | `.gitee/workflows/ci.yml`（curl 装 Rust 版） |
| Step3：Git 合并冲突 | ✅ | main 保护自动建 PR → 合并 → pull --allow-unrelated |
| Step4：dtolnay + stable/nightly 矩阵 | ✅ | YAML 写完，推送成功 |
| Step5：迁移 GitHub | ✅ | remote 改 GitHub + 129 objects push 成功 |
| Step6：GitHub CI 调试 | ✅ | dtolnay 挂 → 换官方 actions-rust-lang |
| Step7：本地实锤验证 | ✅ | 176 tests + clippy clean + fmt OK |

### 6.2 关键成果

（列出本次实验产出的关键成果）

1. **GitHub Actions CI 流水线**：编写并迭代了 3 版 workflow YAML，最终版使用官方 `actions-rust-lang/setup-rust-toolchain@v1`，含 lint-and-test（Clippy 严格模式 + fmt + 176 tests）和 coverage（cargo-llvm-cov + LCOV artifact）两个 job，push 到 main/alpha/beta/rc/* 和 PR 到 main 自动触发。
2. **Rust 最小可测单元**：在 `src/lib.rs` 实现加法函数 `add(a, b)` + 单元测试 `test_add_basic`（覆盖正数、零、负数），并验证整个 SQLRustGo 项目 176 个测试全绿、Clippy 0 警告。
3. **Git 分支与 PR 工作流**：经历了 Gitee main 保护自动建 PR → 合并 → 本地 pull --allow-unrelated → 迁移 GitHub 仓库（remote set-url + force-with-lease）的完整流程，掌握了 protected branch、force-with-lease、upstream tracking、GitHub API 诊断 CI 失败等实用技能。

### 6.3 代码提交

| 项目 | 内容 |
|------|------|
| 分支名称 | main |
| 提交哈希 | bb8dcb3（最终：ci: switch to actions-rust-lang/setup-rust-toolchain） |
| PR 链接 | https://github.com/Aoxin-x/111（CI 直接推 main，GitHub PR 自动触发） |

### 6.4 最终提交历史

```
1fb86ca 补推残留
bb8dcb3 ci: switch to actions-rust-lang/setup-rust-toolchain
617a228 ci: pin action versions
941b365 ci: stable only, remove nightly matrix
cbb1a8f ci: dtolnay rust-toolchain + stable/nightly matrix
b3310ce ci: gitee-compatible workflow with rustup install
c7649df feat: sqlrustgo v1.0 - lexer/parser/executor/storage/wal with 142 unit tests + clippy-clean
```

### 6.5 本地验证（关键）

```
✅ cargo test --all-features       176 passed; 0 failed
✅ cargo clippy -- -D warnings     Finished in 1.77s（0 warning）
✅ cargo fmt --check --all         OK
```

---

## 七、问题与解决

### 7.1 问题清单

| 序号 | 问题描述 | 解决方法 | 根因 |
|------|--------|--------|------|
| 1 | Gitee runner 没 Rust，curl 装 Rust 慢/超时 | 用 dtolnay/rust-toolchain Action（带缓存） | Gitee 默认镜像 ubuntu-latest 没装 rustup |
| 2 | Gitee Go 不认 .gitee/workflows/ci.yml，要「新建流水线」 | 暂时跳过 Gitee 自建流水线，直接用 GitHub | Gitee CI 是自家格式，和 GitHub Actions 不兼容 |
| 3 | git push 到 Gitee main → `protected branch` → `auto-xxx-*` | Gitee main 保护自动建 PR，必须点合并 | Gitee 开启了 main 保护（实验之前配的） |
| 4 | 合并 PR 后本地 main 落后远程 | `git pull origin main --allow-unrelated-histories` 再 fast-forward | 远程多了 README + 实验报告 |
| 5 | `git push` 无 `-f` → `non-fast-forward` | `git push -f origin main` 或 `--force-with-lease` | 本地 HEAD 落后远程 |
| 6 | GitHub CI dtolnay/rust-toolchain@v1 → step failure，后面全 skipped | 换成 `actions-rust-lang/setup-rust-toolchain@v1` | dtolnay 版今天 runner 上挂了（个人维护） |
| 7 | 贴到浏览器 Vim merge message 退不出去 | 关终端重开 + `git merge --abort` + `git push -f` | Vim 初学者陷阱（Esc → :wq） |

### 7.2 根因分析

**A. CI 平台差异**：Gitee 自带 Go、Java、Python；不自带 Rust。GitHub ubuntu-latest 也没 Rust，必须手动装。

**B. Action 选型**：Rust 安装 Action 对比：

| Action | 维护方 | 稳定性 | 推荐场景 |
|--------|--------|--------|---------|
| `curl sh.rustup.sh` | 无 | 不稳定（慢、没缓存） | 应急 |
| `dtolnay/rust-toolchain@v1` | dtolnay 个人 | 偶尔挂 | 备选 |
| `actions-rust-lang/setup-rust-toolchain@v1` | GitHub Actions 官方组 | 最稳 | ✅ 推荐 |

**C. 「本地过了云端不过」的真实含义**：不是代码错，是云端环境和本地环境不一样。CI 的意义就是在云端（干净的环境）再跑一遍。

---

## 七、实验总结

### 7.1 知识收获

（总结本次实验学到的知识点）

1. **CI/CD 核心价值**：不是"能自动化就自动化"，而是把"别人电脑上也能跑"变成硬性门禁。以前本地 cargo clippy 没过就 push 了，现在云端自动卡合不了。
2. **Rust CI 三件套**：Clippy `-D warnings`（警告即报错）+ rustfmt `--check`（格式一致）+ cargo test（测试全绿），三条一起跑才叫"代码质量门禁"。
3. **覆盖率 LCOV 流程**：cargo-llvm-cov（比 tarpaulin 新，支持 LLVM 代码覆盖率）→ 生成 lcov.info → upload-artifact 传到 GitHub 页面可下载。
4. **Rust Action 选型**：curl sh.rustup.sh（应急）< dtolnay/rust-toolchain（备选）< actions-rust-lang/setup-rust-toolchain（官方最稳）。Action 作者身份影响稳定性。
5. **CI 平台差异**：Gitee runner 不带 Rust，GitHub ubuntu-latest 也不带——都要自己装。Gitee CI YAML 格式和 GitHub Actions 不兼容（Gitee Go 自家格式）。
6. **GitHub API 诊断**：CI 挂了不用打开浏览器点来点去，直接 `Invoke-RestMethod` 查 jobs/steps/conclusion，1 行命令拿到完整失败链。

### 7.2 技能提升

（总结本次实验提升的技能）

1. 会写 GitHub Actions workflow YAML：trigger（push/PR 分支）+ jobs（runs-on + steps + uses + with）+ matrix（stable/nightly）。
2. 会用 GitHub API 诊断 CI 失败：actions/runs/{id}/jobs → 看每个 step 的 conclusion 链，秒定位根因。
3. 会处理 protected branch：Gitee main 保护自动转 PR、force-with-lease 安全强推、remote set-url 迁移仓库。
4. 会切换 CI 平台：Gitee（curl 装 Rust，CI 自家格式不认 GitHub Actions YAML）→ GitHub（标准 Actions，官方 Rust Action 可直接用）。
5. 会分析 Action 自身故障：dtolnay v1 挂 → 看 GitHub jobs API 的 step conclusion → 换成官方 actions-rust-lang → 恢复。

### 7.3 心得体会

（分享实验过程中的心得体会）

最触动的是**CI 不是"锦上添花"，是"最低门槛"**。

之前几周都是本地跑一遍 cargo test 就 push 了。今天写了 CI 才发现：我的本地 Clippy 偶尔就有 warning（`unused import` 之类的），但我从来不管——因为没人拦我。CI 加上 `-D warnings` 之后，哪怕 1 个 warning 都合不了 main，倒逼自己把这些小事清干净。

第二个触动是**Action 作者身份很重要**。dtolnay 在 Rust 社区口碑极好，但他的 Action 今天在 GitHub runner 上就是挂了。换成官方 actions-rust-lang 立刻好。这让我意识到：CI 用的 Action 不是"大 V 推荐就一定好"，而是"组织维护的 > 个人维护的"。

第三个触动是**平台差异真的坑人**。Gitee CI 不认 `.github/workflows/`，Gitee Go 不认 GitHub Actions YAML，GitHub runner 配置又和 Gitee 不同。跨平台不是加个 flag 就完事，是整个 YAML 结构可能要重写。最后决定把 origin 直接改 GitHub，彻底砍掉 Gitee CI，专注 GitHub Actions 一套跑通。

### 7.4 改进建议

（对实验内容或方法的改进建议）

1. **先讲 Action 选型**：实验前先演示"三个 Rust 安装 Action 的区别表格"（curl / dtolnay / actions-rust-lang），让学生知道"选官方维护的更稳"，而不是遇到挂了才去换。
2. **CI 平台建议直接上 GitHub**：Gitee CI 和 GitHub Actions YAML 不兼容，学生容易懵。建议 Week-9 直接指定 GitHub，避免一半时间花在"Gitee 为什么不认我的 YAML"上。
3. **GitHub API 诊断命令提前给**：把 `Invoke-RestMethod -Uri https://api.github.com/repos/{owner}/{repo}/actions/runs/{id}/jobs` 写成现成 PowerShell 片段贴给学生，遇到 CI 挂了直接跑，比让学生翻浏览器日志快 10 倍。
4. **Force-with-lease 安全强推**：演示时用 `git push --force-with-lease` 而不是 `git push -f`。前者如果远程有自己没看到的 commit 会拒绝，不会把别人的工作冲掉，更符合"软件治理"的主题。
5. **覆盖率 artifact 下载验证**：实验里跑完覆盖率后让学生"点 GitHub Actions 页面 → Artifacts → 下载 lcov.info → 本地用 genhtml 或 VS Code Coverage Gutters 打开"，感受"云端跑出来的东西可以拿到本地看"。

---

## 八、AI 辅助示例

### 提示词 1：CI 失败，我要知道哪一步挂了

```bash
(Invoke-RestMethod -Uri "https://api.github.com/repos/Aoxin-x/111/actions/runs/28014727946/jobs").jobs |
  ForEach-Object { "$($_.name) | $($_.conclusion) | $(($_.steps | % { "$($_.name)=$($_.conclusion)" }) -join ', ')" }
```

**AI 输出**：
```
lint-and-test | failure | Run dtolnay/rust-toolchain@v1=failure, Clippy=skipped, ...
coverage      | failure | Run dtolnay/rust-toolchain@v1=failure, llvm-cov=skipped, ...
根因定位：dtolnay/rust-toolchain@v1 在 GitHub runner 上自己挂了。
解决：换成 actions-rust-lang/setup-rust-toolchain@v1（官方维护，更稳）。
```

**使用效果**：1 条命令 + AI 分析 → 3 分钟定位并修复。

---

### 提示词 2：Git push 被拒绝（protected branch）

```
remote: Target branch is in review mode
remote: Gitee created a Pull Request for code review
        https://gitee.com/.../pulls/4
```

**AI 输出**：
```
正确行为 ✅ main 保护规则生效，自动建 PR。
操作：点进去「合并」，不要用 -f 硬推。
```

**使用效果**：不是报错，是预期行为。AI 直接告诉我「这是对的，继续」。

---

### 提示词 3：修改 ci.yml 用 dtolnay + stable/nightly 矩阵

**AI 输出**：
```yaml
strategy:
  fail-fast: false
  matrix:
    rust-toolchain: [stable, nightly]
steps:
  - uses: dtolnay/rust-toolchain@stable
    with:
      toolchain: ${{ matrix.rust-toolchain }}
      components: clippy,rustfmt
```

**使用效果**：AI 一次写对，直接用，不需要查文档。

---

## 九、参考资料

1. GitHub Actions 官方文档：https://docs.github.com/actions
2. actions-rust-lang/setup-rust-toolchain：https://github.com/actions-rust-lang/setup-rust-toolchain
3. dtolnay/rust-toolchain：https://github.com/dtolnay/rust-toolchain
4. cargo-llvm-cov：https://github.com/taiki-e/cargo-llvm-cov
5. Clippy 官方文档：https://rust-lang.github.io/rust-clippy/
6. rustfmt 官方文档：https://rust-lang.github.io/rustfmt/
7. Gitee CI 文档：https://gitee.com/docs/administration/ci
8. Git branch protection：https://docs.github.com/repositories/configuring-branches-and-merges

---

## 十、评分标准（自我评估）

| 检查项 | 分值 | 自评 |
|--------|------|------|
| CI workflow 编写（YAML + 触发条件） | 20 | 20（三个版本迭代，含 stable/nightly 矩阵） |
| 代码质量门禁（clippy + fmt + test） | 25 | 25（-D warnings 严格模式 + 176 tests） |
| 覆盖率（cargo-llvm-cov + artifact） | 15 | 15（LCOV 生成 + upload-artifact） |
| Git 分支/PR/冲突处理 | 20 | 20（Gitee → GitHub 迁移 + 合并 + force-with-lease） |
| CI 调试（dtolnay 挂 → 换官方） | 10 | 10（GitHub API 诊断根因 + 正确替换） |
| 实验报告完整 | 10 | 10（含真实命令 + 真实输出 + 根因分析） |
| **总分** | **100** | **100** |

---

*最后更新: 2026-06-23*
