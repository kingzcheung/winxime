# Xime 五笔输入法 - 进度跟踪

## 当前状态
librime submodule 已添加，需要编译

## 实现计划（按顺序）

### 1. Rime 引擎 FFI 绑定层 (胶水代码) - 等待编译验证
- [x] FFI 声明：RimeApi, RimeContext, RimeCommit 等结构体
- [x] rime_struct 宏：初始化结构体并设置 data_size
- [x] RimeEngine：安全封装（process_key, get_candidates, get_commit）
- [x] build.rs：自动化链接 librime
- [x] build-librime.bat：自动化编译脚本
- [ ] 编译 librime（需要用户执行）
- [ ] 验证 cargo build 通过

### 2. TSF 文本服务框架 (系统接口)
- [x] COM 基础：IUnknown, ClassFactory
- [x] ITfTextInputProcessor：输入法入口点
- [x] ITfKeyEventSink：键盘事件接收器
- [ ] 集成 RimeEngine 到 KeyEventSink
- [ ] ITfEditSession：文本编辑会话（提交文字）

### 3. UI 候选窗口 (用户界面)
- [ ] 无焦点弹出窗口
- [ ] 候选词绘制

### 4. 主逻辑控制器 (大脑)
- [ ] 事件循环：按键 → Rime → 输出

## 下一步
用户需要编译 librime：

### 方法 1：使用自动化脚本（推荐）
打开 **Developer Command Prompt for VS 2022**，执行：
```powershell
cd C:\Users\ibuddy\Documents\winxime
build-librime.bat
```

### 方法 2：手动编译
```powershell
cd C:\Users\ibuddy\Documents\winxime\librime
# 创建环境配置（如果需要）
copy env.bat.template env.bat
# 编译依赖
build.bat deps
# 编译 librime
build.bat librime
```

编译完成后运行 `cargo build` 验证。