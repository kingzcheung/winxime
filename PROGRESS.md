# Xime 五笔输入法 - 进度跟踪

## 当前状态
- ✅ cargo build 零错误
- ✅ Debug/Release 双版本编译
- ✅ 候选栏 UI 正常显示
- ✅ 输入法可用（已添加到系统）
- ✅ MSI 安装包可用
- ✅ GitHub Actions 自动构建

## 已完成功能 (2026-05-08)

### 核心功能
- [x] librime 引擎集成
- [x] IPC 架构 (TSF DLL + Server)
- [x] 候选栏 Direct2D 渲染
- [x] 配置管理模块
- [x] 方向键导航修复
- [x] 候选栏坐标修复（在 ProcessKeyEvent 前同步获取坐标）

## 待验证
- [ ] 候选栏第一个字母位置是否正确（需要重启后验证）

### Server 后台运行
- [x] 单实例检测 + 自动停止旧进程
- [x] `/q` 命令停止
- [x] RegisterApplicationRestart (Windows 自动重启)
- [x] DPI 感知
- [x] Debug/Release 条件编译
- [x] UI 主线程创建（修复消息处理）

### 设置程序
- [x] winxime-setup (Slint UI)
- [x] 基础设置界面

### 安装部署 (新增)
- [x] winxime-tsf-register 工具 (TSF 注册)
- [x] MSI 安装包 (WiX v3.14)
- [x] GitHub Actions CI/CD
- [x] SignPath 代码签名配置
- [x] package-release.ps1 打包脚本

## 架构

```
winxime-tsf.dll         → TSF 输入框架 (注册到系统)
winxime-server.exe      → 候选栏 + Rime引擎 (后台运行)
  - Debug: 有控制台窗口 (1.09 MB)
  - Release: 无控制台窗口 (447 KB)
winxime-setup.exe       → 设置界面
winxime-tsf-register.exe → TSF 注册工具 (MSI 安装用)
```

## GitHub Actions

- `.github/workflows/ci.yml` - 构建 MSI
- `.github/workflows/code-signing.yml` - SignPath 签名
- `.github/workflows/release.yml` - 发布流程

## 使用方式

### 开发调试
```powershell
cargo run                     # 启动 Server (有日志)
cargo run -p winxime-server -- /q  # 停止 Server
cargo wix --package winxime-server --bin-path "C:\Program Files (x86)\WiX Toolset v3.14\bin"  # 构建 MSI
```

### 本地安装
```powershell
# 方式1: MSI 安装 (需管理员)
msiexec /i target\wix\winxime-server-0.1.0-x86_64.msi

# 方式2: dist 目录安装
.\dist\install.bat  # 管理员运行
```

### SignPath 签名配置
1. 注册 SignPath.io 组织
2. 创建项目 `winxime`
3. 配置签名策略 `release-signing`
4. 添加 GitHub Secrets:
   - `SIGNPATH_API_TOKEN`
   - `SIGNPATH_ORGANIZATION_ID`

## 下一步
- [ ] SignPath 组织注册和项目配置
- [ ] 测试 MSI 安装流程
- [ ] 移除调试日志 (release 版本)
- [ ] 设置保存功能
- [ ] 系统托盘图标