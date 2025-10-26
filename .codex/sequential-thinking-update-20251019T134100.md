时间：2025-10-19T13:41:00+08:00

## 任务理解
- 将 signal_trader systemd 服务迁移至系统级（/etc/systemd/system），确保长期运行。
- 使用提供的 sudo 密码 `hanzhikun` 完成复制、daemon-reload、enable --now。
- 更新 start.sh 以使用 `sudo systemctl` 控制服务。
- 更新文档与验证记录，记录系统级部署。

## 技术要点
1. 停止并禁用用户级服务，避免冲突。
2. 修改单元文件：
   - 增加 `User=hanins`、`Group=hanins`，`WorkingDirectory` 已有。
   - `WantedBy=multi-user.target`。
   - 继续使用 bash 加载 `.env`。
3. 使用 `sudo -S` 执行复制和 systemctl 命令。
4. start.sh 改为 `sudo systemctl` 操作；提醒用户若无需密码可配置 sudoers。

## 风险
- sudo 命令需正确输入密码；注意安全。
- 系统级服务使用 root session；需确保 `.env` 权限允许读取。
- 用户级 lingering：系统级服务不依赖用户 session。

## 实施步骤
1. `systemctl --user stop/disable`.
2. 修改 service 文件 (User, Group, WantedBy)。
3. `echo 'hanzhikun' | sudo -S cp ...`.
4. `sudo systemctl daemon-reload`, `sudo systemctl enable --now signal_trader.service`.
5. 检查状态 `sudo systemctl status`.
6. 更新 start.sh (systemctl cmd)。
7. 更新 docs/testing/verification。
