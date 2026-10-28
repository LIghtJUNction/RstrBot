# CLI、配置与退出

rstrbot默认等同serve。成功维护操作退出0，错误非0并给明确信息；参数错误由CLI解析器拒绝。精确错误文字可本地化，但不得泄露密钥或把失败当成功。

| 入口 | 输入/环境 | 可观察结果 |
| --- | --- | --- |
| serve / 无子命令 | 全局选项 | 配置数据目录/实例锁，完成恢复后监听127.0.0.1:8787并提供同源Web/API；Ctrl-C请求正常退出 |
| check | 数据库和数据根 | 验证数据库可访问、初始化和恢复状态；明确不测供应商连通性 |
| backup | --output 新文件 | 需要初始化且不与运行核心共用数据根；一致数据库快照和摘要，现有文件不覆盖；提醒另存密钥/外部状态 |
| restore | --input 快照 | 输入最多300MiB，校验后导入空数据库，进入恢复暂停；非空目标拒绝 |
| recover-password | --username，RSTRBOT_RECOVERY_PASSWORD | 本机操作者独立恢复密码，撤销旧会话，不把密码输出 |
| plugin-install | --manifest，显式--replace | 验证可信本机程序清单、权限披露/校验后登记；登记不启动进程 |

| 全局选项 | 环境变量 | 合同 |
| --- | --- | --- |
| --data-dir | RSTRBOT_DATA_DIR | 显式便携目录；缺省系统application data目录，不依赖cwd |
| --database-url | RSTRBOT_DATABASE_URL | SQLite/PG URL，配置敏感，默认SQLite在数据根 |
| --listen | RSTRBOT_LISTEN | 默认127.0.0.1:8787；非loopback需明确安全部署配置 |
| --valkey-url | RSTRBOT_VALKEY_URL | 可选，缺失不阻断核心功能 |
| --secure-cookie | RSTRBOT_SECURE_COOKIE | 保存正向true，不提供TLS；关闭需显式维护配置 |
| --allow-private-network | RSTRBOT_ALLOW_PRIVATE_NETWORK | 实例政策与具体目标授权共同约束 |
| --web-dir | RSTRBOT_WEB_DIR | 已构建Web静态目录 |

Unix数据目录0700、凭据/配置0600，维护命令与核心争用同一实例锁。bootstrap.token仅本机读取，15分钟有效，未初始化重启重新生成；URL与日志不携带其内容。配置schema升级、恢复旧版本及反向代理/Host/Origin的细节见OPERATIONS_REFERENCE.md；认证邮件/Google/Passkey见AUTHENTICATION_REFERENCE.md。
