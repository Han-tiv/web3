# 🎯 Mermaid.live 正确复制指南

## ❌ 你遇到的错误

```
UnknownDiagramError: No diagram type detected matching given configuration for text: #
```

**原因**: 复制了 Markdown 标题（`#` 开头）或代码块标记（\`\`\`mermaid）

---

## ✅ 正确的复制方法

### 第1步：找到代码块

在 `PROGRAM_FLOW_DIAGRAMS.md` 中，代码块格式如下：

```
## 1️⃣ 系统整体架构图          ← ❌ 这是标题，不要复制

```mermaid                     ← ❌ 这是代码块标记，不要复制
graph TB                       ← ✅ 从这里开始复制
    subgraph "外部输入"
        A[Telegram消息] --> B
    end
    ...                        ← ✅ 复制所有Mermaid代码
    style H fill:#fff3e0       ← ✅ 包括样式定义
```                            ← ❌ 这是代码块结束标记，不要复制
```

### 第2步：精确复制范围

**只复制** \`\`\`mermaid 和 \`\`\` **之间**的内容！

---

## 📋 第一个图表：系统整体架构

### 正确的复制内容（从PROGRAM_FLOW_DIAGRAMS.md 第24-79行）

复制以下完整代码到 https://mermaid.live/ ：

```
graph TB
    subgraph "外部输入"
        A[Telegram消息] --> B[Python监听器]
        B --> C[数据库 telegram_signals]
    end
    
    subgraph "Rust交易系统"
        D[主程序 mod.rs] --> E[配置加载]
        D --> F[数据库初始化]
        D --> G[交易器初始化]
        
        G --> H[IntegratedAITrader<br/>trader.rs]
        
        H --> I[持仓监控线程<br/>monitor_positions]
        H --> J[延迟队列线程<br/>reanalyze_pending_entries]
        H --> K[信号轮询线程<br/>Telegram Polling]
        H --> L[Web服务器<br/>8080端口]
        
        C --> K
        K --> M[信号处理<br/>analyze_and_trade]
        M --> N[AI分析 Gemini]
        N --> O[开仓执行<br/>execute_trial_entry]
        
        I --> P[AI评估 DeepSeek]
        P --> Q[止损/止盈决策]
        
        J --> M
    end
    
    subgraph "外部API"
        N --> R[Gemini API<br/>入场分析]
        P --> S[DeepSeek API<br/>持仓管理]
        O --> T[Binance API<br/>下单交易]
        Q --> T
    end
    
    subgraph "数据存储"
        O --> U[(SQLite Database)]
        Q --> U
        L --> U
        U --> V[positions表<br/>ai_analysis表<br/>orders表]
    end
    
    style D fill:#e1f5ff
    style H fill:#fff3e0
    style I fill:#c8e6c9
    style J fill:#c8e6c9
    style K fill:#c8e6c9
    style L fill:#c8e6c9
    style M fill:#ffe0b2
    style N fill:#f8bbd0
    style O fill:#ffccbc
    style P fill:#f8bbd0
    style Q fill:#ffccbc
```

**复制上面这段代码，不包括最上面和最下面的三个反引号！**

---

## 🎯 逐步操作指南

### 方法1: 手动选择（推荐）

1. **打开** `PROGRAM_FLOW_DIAGRAMS.md`

2. **找到第24行**，看到 \`\`\`mermaid

3. **从第25行开始选择**（`graph TB`）

4. **选到第79行结束**（最后一个 `style P...`）

5. **不要选择** 第23行（标题）和第80行（\`\`\`结束符）

6. **复制** `Ctrl + C`

7. **打开** https://mermaid.live/

8. **粘贴** `Ctrl + V` 到左侧编辑器

9. **查看** 右侧自动渲染的流程图！

### 方法2: 从本文档直接复制

直接从上面的代码块复制（不包括三个反引号）：

```
← 不要复制这行
graph TB
    subgraph "外部输入"
    ...
    style P fill:#f8bbd0
← 不要复制这行
```

---

## 📊 所有10个图表的精确位置

| 图表名称 | 开始行 | 结束行 | 复制内容 |
|----------|--------|--------|----------|
| 1. 系统整体架构 | 25 | 79 | `graph TB` 到 `style Q...` |
| 2. 程序启动流程 | 87 | 138 | `flowchart TD` 到 `style MainLoop...` |
| 3. 并发任务架构 | 146 | 249 | `graph TB` 到 `style T5...` |
| 4. 信号处理流程 | 257 | 316 | `flowchart TD` 到 `style AIAnalysis...` |
| 5. AI分析决策 | 324 | 440 | `flowchart TD` 到 `style End...` |
| 6. 持仓监控流程 | 448 | 568 | `flowchart TD` 到 `style UpdateDB...` |
| 7. 延迟队列流程 | 576 | 665 | `flowchart TD` 到 `style Remove3...` |
| 8. 数据流向图 | 673 | 738 | `graph LR` 到 `style N...` |
| 9. 状态转换图 | 746 | 810 | `stateDiagram-v2` 到最后 |
| 10. 时序图 | - | - | 未包含（如需要可添加） |

**提示**: 
- 开始行 = \`\`\`mermaid 的下一行
- 结束行 = \`\`\` 的上一行
- 不包括代码块标记符本身

---

## 🚀 快速测试

### 简单示例（测试用）

复制这段到 https://mermaid.live/ 试试：

```
graph LR
    A[开始] --> B[处理]
    B --> C{判断}
    C -->|是| D[成功]
    C -->|否| E[失败]
    
    style A fill:#4caf50,color:#fff
    style D fill:#2196f3,color:#fff
    style E fill:#f44336,color:#fff
```

**记住**: 不要复制上下的三个反引号！

---

## ⚠️ 常见错误

### 错误1: 复制了标题
```markdown
## 1️⃣ 系统整体架构图  ← ❌ 这会导致错误
```

### 错误2: 复制了代码块标记
```markdown
```mermaid                  ← ❌ 不要复制
graph TB
```                         ← ❌ 不要复制
```

### 错误3: 复制了注释
```markdown
<!-- 这是注释 -->          ← ❌ 不要复制
```

---

## ✅ 验证方法

粘贴到 mermaid.live 后：

**成功** ✅：
- 右侧显示流程图
- 没有错误信息

**失败** ❌：
- 显示 "UnknownDiagramError"
- 显示 "Syntax error"
- 没有图形显示

如果失败，检查：
1. 是否复制了 `#` 标题？
2. 是否复制了 \`\`\`mermaid 标记？
3. 是否复制了 \`\`\` 结束标记？
4. 代码是否完整？

---

## 🎯 推荐工作流

### 最佳实践

```bash
1. 打开 PROGRAM_FLOW_DIAGRAMS.md
2. 找到想看的图表
3. 看到 ```mermaid 行号（比如第24行）
4. 从下一行开始选择（第25行）
5. 选到 ``` 的上一行（比如第79行）
6. 复制 Ctrl + C
7. 打开 https://mermaid.live/
8. 粘贴 Ctrl + V
9. 查看右侧渲染结果
10. 可选：导出为PNG图片
```

---

<div align="center">

# 🎉 现在应该可以了！

## 关键要点

```
只复制 Mermaid 代码本身
不要复制:
  ❌ Markdown 标题 (#)
  ❌ 代码块标记 (```mermaid)
  ❌ 结束标记 (```)
```

---

**试试复制上面的"系统整体架构"代码**  
**粘贴到 https://mermaid.live/**  
**立即看到效果！** 🚀

</div>
