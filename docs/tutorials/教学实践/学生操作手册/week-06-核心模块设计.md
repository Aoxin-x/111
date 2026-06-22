# Week 6 - 核心模块设计 实验指南

## 实验目标

完成 SQLRustGo 1.0 核心模块的设计文档，包括 Parser、Optimizer、Executor、Storage 四大模块的 OOA 分析、OOD 设计和详细设计。

## 实验内容

### 步骤1：OOA分析

1. 用例图（Mermaid）
2. 概念类图（Mermaid classDiagram）
3. 活动图（Mermaid flowchart）

### 步骤2：OOD设计

1. 设计类图（Mermaid classDiagram）
2. 顺序图（Mermaid sequenceDiagram）
3. 状态图（Mermaid stateDiagram-v2）
4. 组件图（Mermaid graph TB）

### 步骤3：详细设计

1. 模块概述
2. 核心功能表
3. 类与接口设计（Rust代码）
4. 执行流程（Mermaid）
5. 性能考虑
6. 1.0实现清单

## 模块清单

| 模块 | 文件 | 核心职责 |
|------|------|----------|
| Parser | parser_module_design.md | SQL字符串 → AST |
| Optimizer | optimizer_module_design.md | LogicalPlan → PhysicalPlan |
| Executor | executor_module_design.md | PhysicalPlan → 结果集 |
| Storage | storage_module_design.md | 数据持久化 + 索引 |
| TestPlan | test_plan.md | 单元/集成/端到端测试 |

## 提交步骤

```bash
git checkout -b docs/module-design-week6
git add docs/design/parser_module_design.md
git add docs/design/optimizer_module_design.md
git add docs/design/executor_module_design.md
git add docs/design/storage_module_design.md
git add docs/design/test_plan.md
git add docs/tutorials/教学实践/学生操作手册/week-06-核心模块设计.md
git commit -m "docs: add module design for week 6"
git push origin docs/module-design-week6
```

## 验收标准

- [ ] 每个模块包含完整的 OOA + OOD + 详细设计
- [ ] 所有图表使用 Mermaid 格式
- [ ] 关键接口提供 Rust 代码实现
- [ ] 1.0实现清单包含优先级（P0/P1/P2）
- [ ] 提交到 docs/module-design-week6 分支
