# Xime 五笔输入法 (Windows)

基于 RIME 引擎的 Windows 五笔输入法，使用 Rust + TSF 构建。

## 安装

下载 MSI 安装包并运行：

```powershell
# 安装 MSI
msiexec /i winxime-server-0.1.0-x86_64.msi

# 卸载
msiexec /x winxime-server-0.1.0-x86_64.msi
```

安装后：
1. 按 `Win+Space` 切换到 Xime 输入法
2. 运行 `winxime-setup.exe` 打开设置界面

## 开发构建

### 前置要求

- Rust 工具链（stable）
- Visual Studio 构建工具（C++ 支持）
- CMake

### 快速开始

```powershell
# 开发循环：重新构建并测试
.\rebuild.ps1
```

手动步骤：

```powershell
# 1. 构建
cargo build --quiet

# 2. 注册 TSF DLL（首次或重新构建 DLL 后，必须提权）
powershell -Command "Start-Process -Verb RunAs -Wait -FilePath 'regsvr32.exe' -ArgumentList '/s target\debug\winxime_tsf.dll'"

# 3. 启动服务器
target\debug\winxime-server.exe

# 4. 按 Win+Space 切换到 Xime 输入法
```

## 构建 MSI 安装包

```powershell
.\msi-build.ps1
```

输出：`target\wix\winxime-server-0.1.0-x86_64.msi`

## 项目结构

- `crates/winxime-server/` — IPC 服务器（与 RIME 引擎通信）
- `crates/winxime-setup/` — 设置界面（Slint UI）
- `crates/winxime-tsf/` — TSF 输入法 DLL
- `crates/winxime-ipc/` — 命名管道 IPC
- `crates/winxime-core/` — 共享数据结构
- `crates/winxime-rime/` — RIME 引擎绑定
- `crates/librime-sys/` — librime 原生绑定
- `librime/` — RIME 引擎子模块
- `rime-data/` — 五笔方案和数据文件

## 许可证

MIT