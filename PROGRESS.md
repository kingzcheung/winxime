# Xime 五笔输入法 - 进度跟踪

## 当前状态
- [x] cargo build / cargo test 零错误零警告
- [x] librime 引擎编译 (build.rs 自动化)
- [x] 配置管理模块 (winxime-config)
- [x] IPC 架构重构完成
- [x] Server 正常启动并监听 Named Pipe
- [x] Rime engine 初始化成功，加载 wubi86_jidian schema
- [x] 方向键导航候选词修复 (2026-05-08)

## Bug 修复 (2026-05-08)
- **问题**: 方向键 Left/Right 无法正确导航候选词
- **原因**: `vk_to_xk()` 函数缺少方向键映射，Windows VK 直接传给 Rime，但 Rime 需要 X11 keysym
- **修复**: 在 `librime-sys/src/lib.rs` 添加:
  - XK_Left (65361), XK_Up (65362), XK_Right (65363), XK_Down (65364) 常量
  - VK_LEFT/UP/RIGHT/DOWN -> XK_Left/Up/Right/Down 映射
  - VK_PRIOR/NEXT -> XK_Prior/Next 映射（翻页键）

## 架构重构 (2026-05-07)

### 新架构 (参考 weasel/chewing-tsf)
```
┌─────────────────────────────────────────────────────────────┐
│                      用户桌面                                │
│                                                              │
│  ┌──────────────────┐    Named Pipe   ┌───────────────────┐ │
│  │ 应用程序进程 A    │  ═════════════  │ winxime-server.exe│ │
│  │  └── winxime-tsf │◄──────────────►│ - Rime Engine     │ │
│  │      (DLL)       │                 │ - IPC Server      │ │
│  │      IPC Client  │                 │ - UI 窗口         │ │
│  └──────────────────┘                 └───────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Crate 结构
| Crate | 作用 | 输出类型 |
|-------|------|----------|
| winxime-ipc | IPC 协议定义、Named Pipe (interprocess) | lib |
| winxime-server | Server + UI + Rime 引擎 | exe |
| winxime-tsf | TSF DLL, IPC Client | dll |
| winxime-rime | Rime 引擎封装 | lib |
| winxime-core | 共享数据结构 | lib |
| winxime-config | 配置管理、基础配置自动加载 | lib |
| librime-sys | librime FFI | lib |

## 实现情况

### 1. winxime-ipc - 完成
- [x] IPC 消息定义 (IpcRequest, IpcResponse)
- [x] Context, Status, CandidateInfo 数据结构
- [x] IpcClient (使用 interprocess crate)
- [x] 命令类型定义

### 2. winxime-server - 基础完成
- [x] IPC Server 监听 (interprocess PipeListener)
- [x] Request 处理逻辑
- [x] Rime Engine 集成
- [x] UI 候选窗口 (基础 GDI 渲染)
- [ ] 候选词动态绘制 (从 Context 读取)
- [ ] 系统托盘图标

### 3. winxime-tsf - 已更新为 IPC Client
- [x] 修改为 IPC Client (IpcClientHandle)
- [x] 替换直接 RimeEngine 调用
- [x] KeyEventSink 通过 IPC 发送按键请求

### 4. librime-sys - 完成
- [x] FFI 绑定
- [x] 数据文件复制

### 5. winxime-rime - 完成
- [x] Rime API 封装

### 6. winxime-config - 完成 (2026-05-07)
- [x] 配置结构定义 (XimeConfig)
- [x] 双目录管理: shared_data_dir + user_data_dir
- [x] 运行时检查必需配置
- [x] 自动下载默认配置 (https://github.com/kingzcheang/rime-wubi)
- [x] 开发模式 vs 生产模式自动切换

## 运行方式

### 开发测试
```powershell
# 1. 构建
cargo build

# 2. 启动 Server
cargo run

# 3. 注册 DLL (管理员 PowerShell)
regsvr32 target\debug\winxime_tsf.dll

# 或使用 xtask 查看完整流程
cargo run -p xtask -- run-dev
```

### 用户安装（未来）
将提供 MSI 安装包，包含：
- 自动注册 TSF DLL
- 自动启动 Server（开机启动）
- 一键安装/卸载

## 下一步
1. ~~更新 winxime-tsf 使用 IPC Client 连接 Server~~ (已完成)
2. ~~实现 winxime-server IPC Server~~ (已完成)
3. ~~添加 winxime-config 配置模块~~ (已完成 2026-05-07)
4. ~~测试配置加载~~ (Server 成功启动，wubi86_jidian schema 已加载)
5. ~~修复方向键导航~~ (已完成 2026-05-08 - VK 到 X11 keysym 映射)
6. 测试端到端功能（注册 DLL，实际输入测试）
7. 完善候选词窗口动态渲染（Direct2D Fluent Design）
8. 系统托盘图标