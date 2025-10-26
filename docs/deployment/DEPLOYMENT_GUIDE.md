# 🚀 Web3项目新服务器部署完整指南

## 📋 部署概况
- **目标服务器**: `47.79.146.30`
- **用户名**: `hanins`
- **密码**: `hanzhikun`
- **项目路径**: `~/code/web3`

## ✅ 已准备的文件
- `web3_deploy_20250929_070240.tar.gz` (216MB) - 完整项目包
- `transfer_sensitive_files.sh` - 敏感文件传输脚本
- `deploy_manual.sh` - 手动部署指导脚本

---

## 🎯 推荐部署方法：Git克隆 + 敏感文件传输

### 第一步：连接服务器并克隆仓库
```bash
# 连接到服务器
ssh hanins@47.79.146.30
# 输入密码: hanzhikun

# 安装必要工具
sudo apt update && sudo apt install -y git curl

# 创建项目目录
mkdir -p ~/code && cd ~/code

# 克隆GitHub仓库
git clone https://github.com/Han-tiv/web3.git

# 进入项目目录
cd web3
```

### 第二步：传输敏感文件 (在本地执行)
```bash
# 在你的本地 Web3 目录执行
./transfer_sensitive_files.sh
# 按提示输入密码: hanzhikun
```

### 第三步：服务器环境配置
```bash
# 在服务器上继续执行

# 安装 Node.js 18+
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt-get install -y nodejs

# 安装 Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
node --version
docker --version
rustc --version
```

### 第四步：安装项目依赖
```bash
# 在服务器的 ~/code/web3 目录

# 安装主项目依赖
npm install

# 安装子项目依赖
cd apps/social-monitor/services/nitter && npm install && cd ../../../..
cd apps/rust-trading-bot && npm install && cd ../..

# 构建 Rust 项目
cd apps/rust-trading-bot
cargo build --release
cd ../..
```

### 第五步：配置环境变量
```bash
# 创建主环境变量文件
cp .env.example .env
nano .env  # 编辑填入你的配置

# 检查敏感文件是否正确传输
ls -la apps/kronos-defi/packages/trading-engine/.env
ls -la apps/social-monitor/services/nitter/sessions.jsonl
```

### 第六步：启动服务
```bash
# 给启动脚本执行权限
chmod +x start.sh

# 启动服务
./start.sh
```

---

## 🔄 备选方法：压缩包部署

如果Git方法有问题，可以使用压缩包：

```bash
# 在本地执行
scp web3_deploy_20250929_070240.tar.gz hanins@47.79.146.30:~/

# 连接服务器
ssh hanins@47.79.146.30

# 解压项目
mkdir -p ~/code
cd ~/code
tar -xzf ~/web3_deploy_20250929_070240.tar.gz
mv web3 web3_old 2>/dev/null || true
mkdir web3
cd ~/
tar -xzf web3_deploy_20250929_070240.tar.gz -C ~/code/web3/
rm web3_deploy_20250929_070240.tar.gz

# 然后传输敏感文件 (在本地执行)
./transfer_sensitive_files.sh
```

---

## 🔍 验证部署

### 检查项目结构
```bash
# 在服务器上
cd ~/code/web3
ls -la  # 应该看到完整项目结构

# 检查重要文件
ls -la start.sh
ls -la .env.example
ls -la apps/*/
```

### 检查敏感文件
```bash
# 验证敏感配置文件
ls -la .env
ls -la apps/kronos-defi/packages/trading-engine/.env
ls -la apps/social-monitor/services/nitter/sessions.jsonl
```

### 测试启动
```bash
# 测试统一启动脚本
./start.sh
# 选择选项查看服务状态
```

---

## 🔧 常用服务管理命令

```bash
# 查看服务状态
./start.sh  # 选择选项 6

# 停止所有服务
./start.sh  # 选择选项 7

# 重启特定服务
cd apps/social-monitor && docker-compose restart

# 查看日志
docker logs container_name

# 查看端口占用
netstat -tlnp | grep :3001
```

---

## 🔒 安全建议

1. **防火墙设置**
```bash
sudo ufw enable
sudo ufw allow ssh
sudo ufw allow 3001  # Nitter
sudo ufw allow 3002  # 监控面板
```

2. **SSL证书** (生产环境)
```bash
sudo apt install certbot
sudo certbot --nginx -d yourdomain.com
```

3. **定期备份**
```bash
# 创建备份脚本
crontab -e
# 添加: 0 2 * * * tar -czf ~/backup/web3_$(date +\%Y\%m\%d).tar.gz ~/code/web3
```

---

## 📞 故障排除

### 常见问题

1. **权限问题**
```bash
chmod +x start.sh
chmod 600 .env
```

2. **端口占用**
```bash
sudo fuser -k 3001/tcp
sudo fuser -k 3002/tcp
```

3. **Docker权限**
```bash
sudo usermod -aG docker $USER
# 注销后重新登录
```

4. **Node.js版本**
```bash
node --version  # 应该是 18+
npm --version
```

---

## ✅ 部署完成检查清单

- [ ] 服务器连接正常
- [ ] 项目文件完整复制
- [ ] 敏感文件安全传输
- [ ] 环境工具安装完成 (Node.js, Docker, Rust)
- [ ] 项目依赖安装成功
- [ ] 环境变量配置正确
- [ ] 服务启动正常
- [ ] 端口访问正常

---

**弟弟，现在你的Web3项目已经准备好部署到新服务器了！按照上面的步骤执行就可以了** 🚀