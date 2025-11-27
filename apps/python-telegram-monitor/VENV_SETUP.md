# Python虚拟环境使用指南

## ✅ 当前状态

Python监控模块已配置完成虚拟环境:
- 虚拟环境路径: `/home/hanins/code/web3/apps/python-telegram-monitor/venv/`
- 所有依赖已安装: telethon, aiohttp, python-dotenv, colorlog

---

## 📦 虚拟环境已安装的包

```
telethon==1.36.0       # Telegram客户端
aiohttp==3.9.1         # HTTP异步客户端
python-dotenv==1.0.0   # 环境变量管理
colorlog==6.8.0        # 彩色日志输出
```

---

## 🚀 使用方式

### 方式1: 使用启动脚本 (推荐)

启动脚本已自动支持venv:

```bash
# 从根目录一键启动
cd /home/hanins/code/web3
bash start_trading.sh

# 或单独启动Python监控
cd apps/python-telegram-monitor
bash start_monitor.sh
```

### 方式2: 手动激活venv

```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor

# 激活虚拟环境
source venv/bin/activate

# 运行监控程序
python3 telegram_monitor.py

# 退出虚拟环境
deactivate
```

---

## 🔧 虚拟环境管理

### 重新创建虚拟环境

```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor

# 删除旧环境
rm -rf venv

# 创建新环境
python3 -m venv venv

# 激活并安装依赖
source venv/bin/activate
pip install -r requirements.txt
```

### 更新依赖包

```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor
source venv/bin/activate

# 更新单个包
pip install --upgrade telethon

# 更新所有包
pip install --upgrade -r requirements.txt

# 查看已安装的包
pip list
```

### 添加新依赖

```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor
source venv/bin/activate

# 安装新包
pip install requests

# 更新requirements.txt
pip freeze > requirements.txt
```

---

## ⚠️ 注意事项

### 1. Git忽略venv目录

虚拟环境目录已添加到`.gitignore`:
```
venv/
*.pyc
__pycache__/
```

### 2. 不要全局安装依赖

**错误做法**:
```bash
pip3 install telethon  # ❌ 全局安装,可能污染系统
```

**正确做法**:
```bash
source venv/bin/activate
pip install telethon  # ✅ 只安装到venv
```

### 3. 激活venv的判断

检查是否在venv中:
```bash
which python3
# venv中: /home/hanins/code/web3/apps/python-telegram-monitor/venv/bin/python3
# 系统: /usr/bin/python3
```

### 4. IDE配置

如果使用VS Code:
```json
{
  "python.defaultInterpreterPath": "${workspaceFolder}/apps/python-telegram-monitor/venv/bin/python3"
}
```

如果使用PyCharm:
- Settings → Project → Python Interpreter
- 选择: `/home/hanins/code/web3/apps/python-telegram-monitor/venv/bin/python3`

---

## 🐛 常见问题

### Q: 激活venv后提示找不到模块?

```bash
# 确认已安装依赖
source venv/bin/activate
pip list | grep telethon

# 如果未安装
pip install -r requirements.txt
```

### Q: venv损坏无法使用?

```bash
# 删除重建
rm -rf venv
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

### Q: 系统提示没有venv模块?

```bash
# 安装venv
sudo apt install python3-venv

# 或使用完整包名
sudo apt install python3.11-venv
```

### Q: pip版本过旧?

```bash
source venv/bin/activate
python3 -m pip install --upgrade pip
```

---

## 📊 虚拟环境信息

```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor
source venv/bin/activate

# Python版本
python3 --version
# Python 3.11.2

# pip版本
pip --version
# pip 23.0.1

# 已安装包数量
pip list | wc -l
# 15个包 (包括依赖)

# 磁盘占用
du -sh venv/
# 约30MB
```

---

## ✅ 验证安装

运行测试脚本:
```bash
cd /home/hanins/code/web3/apps/python-telegram-monitor
source venv/bin/activate

# 测试配置
python3 -c "from config import validate_config; print('配置OK')"

# 测试依赖
python3 -c "import telethon; print(f'Telethon {telethon.__version__} OK')"
python3 -c "import aiohttp; print(f'aiohttp {aiohttp.__version__} OK')"
```

---

## 🎯 最佳实践

1. **始终使用venv** - 避免污染系统Python环境
2. **保持requirements.txt更新** - 便于在其他机器部署
3. **定期更新依赖** - 修复安全漏洞和bug
4. **文档化依赖原因** - 便于团队理解

---

**最后更新**: 2025-11-21
**维护者**: AI Trading System
