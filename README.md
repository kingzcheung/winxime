# Xime 五笔输入法 (Windows)

基于 RIME 引擎的 Windows 五笔输入法，使用 Rust + TSF 构建。

## 快速开始

- **开发构建**：`.\rebuild.ps1` (构建+注册+启动服务器)
- **打包 MSI**：`.\msi-build.ps1` (生成安装包)
- **卸载 MSI**：`.\uninstall-msi.ps1` (完整卸载安装)

## 安装

下载 MSI 安装包并运行：

```powershell
# 安装 MSI（需管理员权限）
msiexec /i xime-0.1.0.msi

# 或双击 MSI 文件安装
```

安装后：
1. 按 `Win+Space` 切换到 Xime 输入法
2. 开始菜单 → Xime → Xime 设置（打开设置界面）

## 开发构建

### 前置要求

- Rust 工具链（stable）
- Visual Studio 构建工具（C++ 支持）
- CMake
- WiX Toolset v3.14（用于 MSI 打包）

### 开发循环

```powershell
# 重新构建并测试
.\rebuild.ps1
```

### 手动步骤

```powershell
# 1. 构建
cargo build --quiet

# 2. 注册 TSF DLL（首次或重新构建 DLL 后，必须提权）
powershell -Command "Start-Process -Verb RunAs -Wait -FilePath 'regsvr32.exe' -ArgumentList '/s target\debug\winxime_tsf.dll'"

# 3. 启动服务器
target\debug\winxime-server.exe

# 4. 按 Win+Space 切换到 Xime 输入法
```

## MSI 打包

```powershell
# 构建 MSI
.\msi-build.ps1

# 输出位置
target\wix\xime-0.1.0.msi
```

## 项目结构

- `crates/winxime-server/` — IPC 服务器（候选栏渲染 + RIME 引擎）
- `crates/winxime-setup/` — 设置界面（GPUI）
- `crates/winxime-tsf/` — TSF 输入法 DLL
- `crates/winxime-tsf-register/` — TSF 注册工具
- `crates/winxime-ipc/` — 命名管道 IPC
- `crates/winxime-core/` — 共享数据结构
- `crates/winxime-rime/` — RIME 引擎绑定
- `crates/winxime-config/` — 配置管理
- `crates/librime-sys/` — librime 原生绑定
- `librime/` — RIME 引擎子模块

## 许可证

MIT