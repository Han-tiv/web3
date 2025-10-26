# ⚙️ Config Directory - 配置目录

**项目配置文件统一管理**

---

## 📂 目录结构

```
config/
├── docker/                 # Docker 相关配置
│   ├── docker-compose.yml      # 生产环境
│   └── docker-compose.dev.yml  # 开发环境
│
├── turbo.json              # Turborepo 配置
└── mise.toml               # Mise 工具配置
```

---

## 📋 配置文件说明

### 1️⃣ Docker 配置

#### docker/docker-compose.yml
**生产环境配置**

```yaml
# 用途: 生产环境容器编排
# 包含: 所有服务的生产配置
```

**使用方法**:
```bash
# 启动生产环境
docker-compose -f config/docker/docker-compose.yml up -d

# 停止
docker-compose -f config/docker/docker-compose.yml down

# 查看日志
docker-compose -f config/docker/docker-compose.yml logs -f
```

#### docker/docker-compose.dev.yml
**开发环境配置**

```yaml
# 用途: 开发环境容器编排
# 特点: 包含开发工具、热重载等
```

**使用方法**:
```bash
# 启动开发环境
docker-compose -f config/docker/docker-compose.dev.yml up

# 后台运行
docker-compose -f config/docker/docker-compose.dev.yml up -d
```

---

### 2️⃣ 构建工具配置

#### turbo.json
**Turborepo 配置**

```json
{
  "pipeline": {
    "build": {},
    "dev": {},
    "lint": {}
  }
}
```

**说明**:
- Monorepo 构建工具配置
- 定义任务流水线
- 优化构建缓存

**使用方法**:
```bash
# 构建所有项目
npm run build

# 开发模式
npm run dev
```

---

### 3️⃣ 开发工具配置

#### mise.toml
**Mise 工具配置**

```toml
[tools]
# 开发工具版本管理
```

**说明**:
- 统一开发环境
- 版本管理
- 工具安装

---

## 🔧 配置管理

### 环境变量

**位置**: 根目录 `.env`

**模板**: 根目录 `.env.example`

```bash
# 复制模板
cp .env.example .env

# 编辑配置
nano .env
```

**文档**: [环境配置指南](../docs/guides/ENV_CONFIGURATION_GUIDE.md)

---

### Docker 配置说明

#### 环境区分

| 环境 | 配置文件 | 特点 |
|------|---------|------|
| **开发** | `docker-compose.dev.yml` | 热重载、调试工具 |
| **生产** | `docker-compose.yml` | 优化、安全加固 |

#### 常用服务

```yaml
services:
  # 示例服务配置
  nitter:
    image: zedeus/nitter:latest
    ports:
      - "8080:8080"
    volumes:
      - ./data:/data
    
  redis:
    image: redis:alpine
    ports:
      - "6379:6379"
```

---

## 📖 使用场景

### 场景 1: 首次部署

```bash
# 1. 准备配置
cp .env.example .env
nano .env

# 2. 启动容器
docker-compose -f config/docker/docker-compose.yml up -d

# 3. 查看状态
docker-compose -f config/docker/docker-compose.yml ps
```

### 场景 2: 开发调试

```bash
# 使用开发配置
docker-compose -f config/docker/docker-compose.dev.yml up

# 查看实时日志
docker-compose -f config/docker/docker-compose.dev.yml logs -f
```

### 场景 3: 配置更新

```bash
# 1. 停止服务
docker-compose -f config/docker/docker-compose.yml down

# 2. 修改配置
nano config/docker/docker-compose.yml

# 3. 重新启动
docker-compose -f config/docker/docker-compose.yml up -d
```

---

## 🔒 安全注意事项

### 敏感信息

**不要在配置文件中硬编码敏感信息！**

✅ **正确做法**:
```yaml
environment:
  - API_KEY=${API_KEY}        # 从环境变量读取
  - DATABASE_URL=${DB_URL}
```

❌ **错误做法**:
```yaml
environment:
  - API_KEY=sk-xxxxx          # 不要硬编码
  - DATABASE_URL=postgres://xxx
```

### 文件权限

```bash
# 配置文件权限
chmod 644 config/docker/*.yml
chmod 644 config/*.json

# 敏感配置（如果有）
chmod 600 .env
```

---

## 📝 配置模板

### Docker Compose 模板

```yaml
version: '3.8'

services:
  app:
    build: .
    container_name: my-app
    restart: unless-stopped
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
      - PORT=3000
    volumes:
      - ./data:/app/data
    networks:
      - app-network

networks:
  app-network:
    driver: bridge

volumes:
  data:
    driver: local
```

---

## 🔍 故障排除

### 常见问题

#### Q1: Docker 容器无法启动

```bash
# 查看日志
docker-compose -f config/docker/docker-compose.yml logs

# 检查配置语法
docker-compose -f config/docker/docker-compose.yml config
```

#### Q2: 端口冲突

```bash
# 查看端口占用
netstat -tlnp | grep :8080

# 修改端口
nano config/docker/docker-compose.yml
```

#### Q3: 环境变量未生效

```bash
# 检查 .env 文件
cat .env

# 重新加载
docker-compose -f config/docker/docker-compose.yml up -d --force-recreate
```

---

## 📊 配置统计

```
配置文件总数: 4 个
├── Docker:      2 个
├── Turbo:       1 个
└── Mise:        1 个
```

---

## 🔗 相关文档

- [项目结构说明](../PROJECT_STRUCTURE.md)
- [环境配置指南](../docs/guides/ENV_CONFIGURATION_GUIDE.md)
- [部署指南](../docs/deployment/DEPLOYMENT_GUIDE.md)

---

## 📋 配置清单

### 部署前检查

- [ ] 复制 `.env.example` 到 `.env`
- [ ] 填写所有必需的环境变量
- [ ] 检查 Docker 配置语法
- [ ] 验证端口没有冲突
- [ ] 测试配置是否正确

### 维护检查

- [ ] 定期审查配置安全性
- [ ] 更新依赖版本
- [ ] 备份重要配置
- [ ] 清理无用配置

---

**⚙️ 配置统一管理，部署更轻松！**

_最后更新: 2025-10-26_
