# 🐾 NetTamer (网络驯兽师)

> **Tame your runaway network traffic.**
> 一款专为解决“后台程序偷偷满速上传、吃满带宽、导致游戏跳 ping 和网页卡顿”而生的现代轻量级桌面端网络监控与流量整形工具。

---

## 💡 核心特性

- ⚡ **零损耗内核监控**：基于 Windows 原生 **ETW (Event Tracing for Windows)** 内核网络会话（`Microsoft-Windows-Kernel-Network`），以极低 CPU 开销实时捕获每个进程的 TCP/UDP 发送与接收速率。
- 🎯 **精准双向限速**：集成 **WinDivert (WFP 驱动)** 与**令牌桶算法 (Token Bucket)**，支持按进程名以 **`KB/s`** 粒度独立限制上传、下载或双向带宽。
- 🚗 **流量高速公路可视化 (Traffic Highway)**：独创拟物化悬浮多车道 3D 视觉流，将各进程网络吞吐实时映射为穿梭车辆与路灯流光，在中央绿化带动态展示全局实时上行/下行速率，支持沉浸式全屏视图。
- 📌 **桌面置顶网速悬浮窗**：
  - 默认停靠屏幕右下角黄金区域，置顶常驻桌面。
  - 全区域支持原生丝滑拖拽移动，双击一键唤出仪表盘主界面。
  - 右键操作系统级原生上下文菜单，支持 `100% / 80% / 60% / 40%` 透明度调节与**鼠标穿透模式**。
  - 自动随主界面深浅色主题自适应同步外观，并提供设置页面与托盘菜单双重一键还原通道。
- 💻 **Windows 任务栏实时网速**：无缝悬浮于任务栏托盘区域，背景全透明、单行垂直精准居中，鼠标点击完全穿透。
- 🔍 **智能进程识别**：通过 Windows `OpenProcess`、`QueryFullProcessImageNameW` 与快照机制解析真实可执行程序路径与名称，并结合 `GetExtendedTcpTable` / `GetExtendedUdpTable` 维护实时动态端口与 PID 映射。
- 🚨 **智能超额预警**：支持按进程通配符（如 `chrome*`、`*update*`）自定义上传/下载阈值与冷却时间，超速即时触发 Windows 原生系统气泡通知。
- 📊 **现代极简 UI**：基于 **Tauri 2.0 + Vue 3 + TypeScript + Tailwind CSS** 构建，原生支持系统托盘常驻、暗黑/浅色模式无缝切换与平滑实时折线图。
- 💾 **轻量本地存储**：内嵌 SQLite 数据库（WAL 模式），本地持久化保存限速规则、预警策略、悬浮窗配置与历史告警日志。

---

## 🏗️ 架构设计

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Frontend (Vue 3 + TS)                             │
│ Dashboard │ Process List │ Throttle Manager │ Visualizer │ Floating Widget  │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │ Tauri 2.0 IPC (Invoke / Event)
┌──────────────────────────────────────▼──────────────────────────────────────┐
│                            Backend (Rust / Tauri)                           │
├──────────────────────────────────────┬──────────────────────────────────────┤
│             ETW Monitor              │           WinDivert Engine           │
│  - Real-time Trace Session           │  - WFP Packet Filtering              │
│  - UserData Byte Decoder             │  - Token-Bucket Limiter              │
│  - PortPidMap (Tcp/UdpTable)         │  - Inbound / Outbound Drop           │
│  - EWMA Rate Aggregator              │  - Case-Insensitive Matching         │
├──────────────────────────────────────┴──────────────────────────────────────┤
│  Alert Engine  │  SQLite Store (WAL)  │  Tray & Taskbar  │  Floating Window │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 技术栈选型

| 模块 | 技术选型 | 说明 |
| :--- | :--- | :--- |
| **应用框架** | **Tauri 2.0** | 轻量级、高安全性的跨端框架，Rust 原生驱动 |
| **后端语言** | **Rust 2021** | 零成本抽象，高并发无锁设计，直接调用 Win32 API |
| **网络监控** | **ETW (`windows-sys` 0.52)** | Windows 内核网络跟踪，毫秒级流速捕获 |
| **网络限速** | **WinDivert 1.4.3** | 结合 Windows Filtering Platform 的数据包过滤与整形 |
| **前端架构** | **Vue 3 (Composition API)** | TypeScript、Pinia 状态管理、Vue Router |
| **样式与组件** | **Tailwind CSS + Lucide Icons** | 精美暗黑/亮色主题、响应式卡片与平滑图表 |
| **本地存储** | **SQLite + rusqlite (r2d2)** | 连接池管理与自动版本迁移 |

---

## 🚀 快速开始

### 1. 前置环境要求

- **操作系统**：Windows 10 / 11 (x64)
- **Node.js**：Node.js >= 18
- **Rust 工具链**：Rust >= 1.77 (推荐 `stable-x86_64-pc-windows-msvc`)
- **Visual Studio C++ Build Tools**：MSVC 编译器与 Windows SDK

> ⚠️ **权限要求**：由于 ETW 实时内核跟踪会话以及 WinDivert 内核驱动需要系统级网络控制能力，开发运行与最终打包程序**必须以管理员身份运行（Run as Administrator）**。

### 2. 安装依赖

```cmd
cd app
npm install
```

### 3. 开发模式启动

以**管理员身份**打开命令提示符（CMD）或 PowerShell：

```cmd
cd /d E:\UGit\NetTamer\app
npm run tauri dev
```

### 4. 生产打包构建

```cmd
npm run tauri build
```
编译产物将生成于 `app/src-tauri/target/release/`。

---

## 📂 项目结构

```text
NetTamer/
├── app/
│   ├── src/                         # Vue 3 前端源码
│   │   ├── components/              # UI 组件库、图表与布局控件
│   │   ├── composables/             # 格式化与业务逻辑组合式函数
│   │   ├── stores/                  # Pinia 状态中心 (进程/限速/预警/设置)
│   │   ├── views/                   # 页面视图 (仪表盘/进程列表/限速/公路/悬浮窗等)
│   │   └── types.ts                 # TypeScript 类型定义
│   ├── src-tauri/                   # Rust 后端源码
│   │   ├── bin/                     # WinDivert 64 位原生运行库与驱动
│   │   ├── capabilities/            # Tauri 2.0 窗口多实例安全授权配置
│   │   ├── src/
│   │   │   ├── alert/               # 预警引擎与通配符匹配器
│   │   │   ├── commands/            # Tauri IPC 指令处理器 (监控/配置/窗口等)
│   │   │   ├── etw/                 # ETW 实时会话与事件解码器
│   │   │   ├── monitor/             # 流量聚合器与 EWMA 平滑算法
│   │   │   ├── process/             # 进程路径/名称解析与端口映射表
│   │   │   ├── store/               # SQLite 持久化层与数据库迁移
│   │   │   ├── throttle/            # 限速策略管理器与令牌桶表
│   │   │   ├── tray/                # 系统托盘图标、任务栏网速与悬浮窗调度
│   │   │   └── windivert/           # WinDivert FFI 与数据包捕获引擎
│   │   ├── build.rs                 # 原生库链接与清单注入构建脚本
│   │   └── Cargo.toml               # 后端依赖配置
│   └── package.json                 # 前端依赖与构建脚本
└── doc/
    └── architecture.md              # 架构设计与技术演进详述
```

---

## 📄 许可证

本项目基于 [MIT License](LICENSE) 开源。
