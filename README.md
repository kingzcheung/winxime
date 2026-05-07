# Xime 五笔输入法 (Windows)

基于 RIME 引擎的 Windows 五笔输入法，使用 Rust + TSF 构建。

## 前置要求

- Rust 工具链（stable）
- Visual Studio 构建工具（C++ 支持）
- CMake

## 快速开始

```powershell
# 1. 构建所有组件
cargo build --quiet

# 2. 注册 TSF DLL（首次或重新构建 DLL 后执行，必须提权！）
powershell -Command "Start-Process -Verb RunAs -Wait -FilePath 'regsvr32.exe' -ArgumentList '/s target\debug\winxime_tsf.dll'"

# 3. 启动服务器（在新终端中运行，保持窗口打开）
target\debug\winxime-server.exe

# 4. 按 Win+Space 切换到 Xime 输入法
```

> **注意：** 必须使用 DLL 的**绝对路径**，提权后工作目录会变。否则注册不会生效。

## 开发循环

每次修改代码后：

```powershell
# 停止旧服务器
taskkill /F /IM winxime-server.exe

# 重新构建
cargo build --quiet

# 重新注册（必须提权）
powershell -Command "Start-Process -Verb RunAs -Wait -FilePath 'regsvr32.exe' -ArgumentList '/s target\debug\winxime_tsf.dll'"

# 启动新服务器
target\debug\winxime-server.exe
```

## 测试

1. 确保服务器正在运行
2. 按 `Win+Space` 切换到 Xime 输入法
3. 在任意应用中输入五笔编码

## 卸载

```powershell
taskkill /F /IM winxime-server.exe
powershell -Command "Start-Process -Verb RunAs -Wait -FilePath 'regsvr32.exe' -ArgumentList '/s /u target\debug\winxime_tsf.dll'"
```

## 项目结构

- `crates/winxime-server/` — IPC 服务器（与 RIME 引擎通信）
- `crates/winxime-tsf/` — TSF 输入法 DLL
- `crates/winxime-ipc/` — 命名管道 IPC
- `crates/winxime-core/` — 共享数据结构
- `crates/winxime-rime/` — RIME 引擎绑定
- `crates/librime-sys/` — librime 原生绑定
- `librime/` — RIME 引擎子模块

## 许可证

MIT
