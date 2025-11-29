# 🔧 Windsurf IDE - Mermaid预览插件安装指南

**目标**: 在Windsurf中安装Mermaid插件以查看流程图  
**难度**: ⭐ 简单  
**时间**: 2分钟

---

## 📦 方法1: 通过扩展市场安装（推荐）

### 步骤1: 打开扩展面板
```
快捷键: Ctrl + Shift + X (Linux/Windows)
       Command + Shift + X (Mac)

或者: 点击左侧边栏的 "扩展" 图标 (四个方块)
```

### 步骤2: 搜索Mermaid插件
在搜索框输入：
```
Mermaid Preview
```

### 步骤3: 选择推荐插件
推荐安装以下任一插件：

#### 选项A: Markdown Preview Mermaid Support ⭐推荐
- **插件名**: Markdown Preview Mermaid Support
- **作者**: Matt Bierner
- **ID**: `bierner.markdown-mermaid`
- **特点**: 
  - ✅ 无需额外配置
  - ✅ 内置在Markdown预览中
  - ✅ 轻量级
  - ✅ 自动渲染

**安装命令**:
```bash
code --install-extension bierner.markdown-mermaid
```

#### 选项B: Mermaid Chart
- **插件名**: Mermaid Chart
- **作者**: Mermaid Chart
- **ID**: `MermaidChart.vscode-mermaid-chart`
- **特点**:
  - ✅ 官方插件
  - ✅ 支持导出
  - ✅ 实时预览

**安装命令**:
```bash
code --install-extension MermaidChart.vscode-mermaid-chart
```

### 步骤4: 重新加载窗口
安装完成后，可能需要重新加载窗口：
```
快捷键: Ctrl + Shift + P → 输入 "Reload Window"
```

---

## 🎯 方法2: 使用命令行安装

### 步骤1: 打开集成终端
```
快捷键: Ctrl + ` (反引号)
```

### 步骤2: 执行安装命令
```bash
# 推荐: 安装 Markdown Preview Mermaid Support
code --install-extension bierner.markdown-mermaid

# 或者: 安装官方 Mermaid Chart
code --install-extension MermaidChart.vscode-mermaid-chart
```

### 步骤3: 等待安装完成
看到成功提示：
```
Extension 'bierner.markdown-mermaid' v1.x.x was successfully installed.
```

---

## 📖 如何使用

### 查看流程图文档

#### 方法1: Markdown预览（推荐）
1. 打开 `PROGRAM_FLOW_DIAGRAMS.md`
2. 按快捷键打开预览：
   ```
   Ctrl + Shift + V  (Linux/Windows)
   Command + Shift + V  (Mac)
   ```
3. 或右键点击 → "打开预览" / "Open Preview"
4. Mermaid图表会自动渲染！

#### 方法2: 分屏预览
1. 打开 `PROGRAM_FLOW_DIAGRAMS.md`
2. 按快捷键：
   ```
   Ctrl + K, V  (分屏预览)
   ```
3. 左侧编辑，右侧实时预览

#### 方法3: 导出为图片（如果安装了Mermaid Chart）
1. 将光标放在Mermaid代码块中
2. 按 `Ctrl + Shift + P`
3. 输入 "Mermaid: Export Diagram"
4. 选择格式（PNG/SVG/PDF）

---

## ✅ 验证安装

### 测试步骤

1. **创建测试文件**: `test_mermaid.md`

2. **输入测试代码**:
```markdown
# Mermaid测试

\`\`\`mermaid
graph LR
    A[开始] --> B[测试]
    B --> C[成功]
    C --> D[结束]
\`\`\`
```

3. **打开预览**: `Ctrl + Shift + V`

4. **查看结果**:
   - ✅ 成功: 看到带箭头的流程图
   - ❌ 失败: 看到代码文本（需要安装插件）

---

## 🔧 配置选项（可选）

### 在 settings.json 中添加配置

打开设置：`Ctrl + ,` 或 `File → Preferences → Settings`

搜索 "mermaid" 并配置：

```json
{
  // Mermaid预览主题
  "markdown.mermaid.theme": "default",
  
  // 可选主题: default, forest, dark, neutral, base
  
  // 自动刷新预览
  "markdown.preview.scrollPreviewWithEditor": true,
  
  // 同步滚动
  "markdown.preview.scrollEditorWithPreview": true
}
```

---

## 🎨 Mermaid主题切换

### 可用主题

```markdown
\`\`\`mermaid
%%{init: {'theme':'default'}}%%
graph LR
    A --> B
\`\`\`

\`\`\`mermaid
%%{init: {'theme':'dark'}}%%
graph LR
    A --> B
\`\`\`

\`\`\`mermaid
%%{init: {'theme':'forest'}}%%
graph LR
    A --> B
\`\`\`
```

### 主题对比

| 主题 | 适用场景 |
|------|----------|
| `default` | 默认，适合文档 |
| `dark` | 深色背景 |
| `forest` | 绿色系，柔和 |
| `neutral` | 中性色，专业 |
| `base` | 简洁，高对比 |

---

## 🚨 常见问题

### 问题1: 插件安装后不显示图表

**解决方案**:
1. 重新加载窗口: `Ctrl + Shift + P` → "Reload Window"
2. 检查Mermaid代码块格式: 必须是 \`\`\`mermaid
3. 确认插件已启用: 扩展面板中查看状态

### 问题2: 语法错误导致不渲染

**解决方案**:
1. 检查Mermaid语法: 使用 https://mermaid.live/ 验证
2. 查看开发者控制台: `Help → Toggle Developer Tools`
3. 查看错误信息

### 问题3: 预览窗口不更新

**解决方案**:
1. 关闭并重新打开预览
2. 保存文件后刷新预览
3. 检查文件编码是否为UTF-8

### 问题4: 中文乱码

**解决方案**:
1. 确保文件编码为UTF-8
2. 在Mermaid配置中添加:
   ```json
   "markdown.mermaid.useBaseUrl": false
   ```

---

## 📚 快速查看你的流程图

### 现在就可以查看了！

```bash
# 1. 确认插件已安装
code --list-extensions | grep -i mermaid

# 2. 打开流程图文档（在Windsurf中）
# 文件: PROGRAM_FLOW_DIAGRAMS.md

# 3. 按 Ctrl + Shift + V 打开预览

# 4. 享受10个精美流程图！
```

---

## 🎯 推荐工作流

### 日常使用

```
1. 打开 PROGRAM_FLOW_DIAGRAMS.md
2. 按 Ctrl + K, V (分屏预览)
3. 左侧查看代码，右侧看图
4. 需要时按 Ctrl + Shift + V 全屏预览
```

### 演示/分享

```
1. 使用 Mermaid Chart 插件
2. 导出为 PNG/SVG
3. 或使用在线工具 https://mermaid.live/
4. 生成高质量图片分享
```

---

## 🌐 在线备选方案

如果不想安装插件，可以使用在线工具：

### Mermaid Live Editor
- **网址**: https://mermaid.live/
- **使用**: 复制代码块 → 粘贴 → 实时预览
- **优点**: 无需安装，立即使用
- **缺点**: 需要手动复制粘贴

### GitHub
- **使用**: 将文件push到GitHub
- **优点**: 自动渲染，美观
- **缺点**: 需要上传代码

---

## ✨ 总结

### 最快安装方法

```bash
# 一行命令搞定
code --install-extension bierner.markdown-mermaid

# 重新加载Windsurf
# 打开 PROGRAM_FLOW_DIAGRAMS.md
# 按 Ctrl + Shift + V
# 完成！
```

### 推荐配置

```json
{
  "markdown.mermaid.theme": "default",
  "markdown.preview.scrollPreviewWithEditor": true,
  "markdown.preview.scrollEditorWithPreview": true
}
```

---

<div align="center">

# 🎉 安装完成！

**现在可以查看流程图了**

```
PROGRAM_FLOW_DIAGRAMS.md
    ↓
Ctrl + Shift + V
    ↓
10个精美流程图 ✨
```

**享受可视化的系统架构！** 🎨

</div>
