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
- [x] Shift 键切换中/英文
- [x] 系统托盘图标（嵌入 ICO 文件）
- [x] 托盘显示中/EN 状态图标
- [x] 托盘左键点击切换中/英
- [x] 托盘右键菜单（设置、退出）
- [x] 切换输入法时自动显示/隐藏托盘图标
- [x] 任务栏按钮点击切换中/英文
- [x] 状态同步：输入法启动/切换时正确显示当前中/英状态
- [x] **ITfCompartmentEventSink 实现（监听输入法切换，已打开应用立即生效）**

### 2026-05-09 新增
- [x] **修复候选栏背景截断问题**
  - 问题：候选词少于5个时背景右边被截断，无圆角
  - 原因：窗口大小预留空间固定为25像素，但阴影需要16*scale像素
  - 解决：添加 BLUR_RADIUS 常量，窗口大小改为 `(width + blur_radius * 2) * scale`

- [x] **ITfThreadMgrEventSink 实现（修复已打开应用切换输入法不生效问题）**
  - 问题：从其他输入法切换到当前输入法时，已打开的应用不触发 StartSession
  - 原因：缺少 `ITfThreadMgrEventSink::OnSetFocus` 接口实现
  - 解决：添加 `ITfThreadMgrEventSink` 接口，在文档焦点变化时触发 start_session

- [x] **架构重构：XimeTextService 直接实现 ITfKeyEventSink**
  - 参考 windows-chewing-tsf 项目架构
  - 移除独立的 KeyEventSink 结构
  - 在 Activate 时一次性注册，永不重新注册

- [x] **修复按键双重处理 bug (P0)**
  - 问题：OnTestKeyDown 和 OnKeyDown 都调用 process_key，按键被处理两次
  - 修复：OnTestKeyDown 只做 should_handle_key 检查，不调用 process_key

- [x] **修复 OnSetFocus 焦点处理 (P0)**
  - 问题：两个分支做相同事情，没有区分焦点丢失/获得
  - 修复：pdimfocus.is_null() → focus_out + 清除 composition；非 null → focus_in + start_session

- [x] **移除所有 unwrap() 调用**
  - winxime-tsf 和 winxime-core 已零 unwrap/expect
  - 改用 `lock().unwrap_or_else(|e| e.into_inner())` 容忍 mutex 中毒

## 已验证
- [x] 候选栏第一个字母位置正确

### Server 后台运行
- [x] 单实例检测 + 自动停止旧进程
- [x] `/q` 命令停止
- [x] RegisterApplicationRestart (Windows 自动重启)
- [x] DPI 感知
- [x] Debug/Release 条件编译
- [x] UI 主线程创建（修复消息处理）

### 设置程序
- [x] winxime-setup (GPUI UI)
- [x] 基础设置界面
- [x] 状态管理模块 (Entity<SettingsState>)
- [x] 组件回调支持 (Switch/NumberInput/Button)
- [x] 关于页面 (版本、作者、仓库、许可)
- [x] 菜单图标 (SVG)
- [x] 标题栏左侧与侧边栏颜色一致
- [x] 菜单选中背景色改为主色

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

## 设计决策

### winxime-setup 配置交互方案 (2026-05-09)
参考项目分析：
- **weasel (小狼毫)**：`WeaselDeployer.exe` 通过 IPC + librime API 交互
  - `StartMaintenance()` → Server 暂停服务
  - 修改 Rime 配置文件
  - `rime->deploy()` → 重新部署
  - `EndMaintenance()` → 恢复服务
- **windows-chewing-tsf**：注册表 + 自动重载
  - 配置存储在 `HKCU\Software\ChewingTextService`
  - TSF DLL 通过 `reload_if_needed()` 检测变化

**最终方案**：采用 `xime.custom.yaml` 配置文件方式
- 配置路径：`%APPDATA%\Xime\xime.custom.yaml`
- winxime-setup 修改配置文件
- winxime-server 通过 librime API 加载，定期检测变化重载
- 交互方式（待定）：文件监听 或 IPC `ReloadConfig` 命令
- UI 设计要符合 fluent design

## 下一步
 - [x] winxime-setup UI 完善进度
   - [x] 状态管理模块
   - [x] 基础组件回调
   - [x] 关于页面
   - [x] 菜单图标
   - [x] 实现配置持久化 (保存到 xime.custom.yaml)
   - [x] 配置项分组细化
   - [x] 标题栏全局部署按钮
 - [x] 实现 xime.custom.yaml 配置读写
   - [x] librime-sys levers API 绑定
   - [x] RimeConfigManager (UI 配置管理)
   - [x] SchemaManager (输入方案管理)
   - [x] deploy_all() (重新部署功能)
   - [x] 自动创建用户配置文件 (%APPDATA%\Rime)
 - [x] Server 配置加载
   - [x] winxime-server/config.rs 模块
   - [x] config_open("xime") 读取 build/xime.yaml
   - [x] 应用到 CandidateModel (字体、颜色)
 - [x] 部署功能优化
    - [x] 标题栏全局部署按钮
    - [x] 部署结果反馈（标题栏显示消息）
 - [x] Server 配置重载机制
    - [x] IPC ReloadConfig 命令 (winxime-ipc)
    - [x] ipc_server.rs 处理 ReloadConfig → eng.deploy()
    - [x] winxime-setup 部署后调用 IpcClient::reload_config()
 - [x] **方案级详细设置 (2026-05-12)**
    - [x] SchemaConfigManager (rime_config.rs)
    - [x] 读取方案配置 (speller/translator/reverse_lookup/tradition)
    - [x] 保存方案配置到 schema.custom.yaml
    - [x] InputSchemaState 添加 schema_config 字段
    - [x] 输入方案页面展示选中方案的详细设置
    - [x] SettingsGroup 组件渲染方案配置分组
 - [ ] 词库管理功能 (导入/导出/同步)
 - [ ] 测试任务栏中/英切换功能
 - [ ] SignPath 组织注册和项目配置
 - [ ] 测试 MSI 安装流程
 - [ ] 移除调试日志 (release 版本)