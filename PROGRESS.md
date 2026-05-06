# Xime 五笔输入法 - 进度跟踪

## 当前状态
- [x] cargo build / cargo test 零错误零警告
- [x] librime 引擎编译 (build.rs 自动化)
- [x] 五笔配置: kingzcheung/rime-wubi → rime-data/

## 实现情况

### 1. FFI 绑定层 (librime-sys) - 完成
- 全部 98 个 RimeApi 函数指针，精确对齐 rime_api.h
- 所有 C 结构体的 Rust 定义
- rime_get_api() 动态加载 rime.dll
- VK ↔ XK 按键映射 + 修饰键检测

### 2. Rime 引擎封装 (winxime-rime) - 完成
- [x] setup → initialize → create_session 正确初始化
- [x] process_key, get_commit, get_composition, get_candidates
- [x] select_candidate, change_page
- [x] set_option, get_option
- [x] get_schema_list, select_schema, get_current_schema
- [x] deploy, get_version
- [x] set_notification_handler (C 回调)
- [x] Drop: destroy_session → finalize

### 3. TSF 文本服务 (winxime-tsf) - 完成
- [x] COM DLL 入口 + DllRegisterServer + DllUnregisterServer
- [x] IClassFactory 实现
- [x] ITfTextInputProcessor (Activate/Deactivate)
- [x] ITfKeyEventSink 注册卸载, OnTestKeyDown, OnKeyDown
- [x] RimeEngine::process_key 联动
- [x] ITfEditSession 文字提交 (ITextStoreACP::SetText)
- [x] ITfComposition start/update/end 预编辑显示
- [x] 候选选择 (数字键 1-9)
- [x] 翻页 (PageUp/PageDown)

### 4. UI 候选窗口 (winxime-ui) - 待实现
- [ ] winit 窗口 + skia-safe 候选词绘制

### 5. 核心逻辑 (winxime-core) - 基础就绪
- [x] InputContext, SharedInputContext
- [ ] 配置保存/加载

## 说明
- TSF 层在 OnKeyDown 中捕获 Rime 引擎输出 (commit/composition/preedit)
- 通过 XimeEditSession::DoEditSession 统一处理文字提交和组合更新
- 数字键 1-9 自动调用 select_candidate 进行候选选择
- 通知回调用于调试输出 (deploy/option 事件)
- Windows 注册需要管理员权限运行 regsvr32
