# 🐾 NetTamer — 项目架构设计文档

> **版本**: v2.0.0-draft  
> **日期**: 2026-08-14  
> **状态**: 设计阶段  
> **重大变更**: v2 将原 v1 的 **Wails v3 (Go)** 方案替换为 **Tauri 2.0 (Rust)**；ETW 监控由 Go 库改为 **windows-rs / windows-sys**；限速由 Windows QoS Policy (PowerShell) 改为 **WinDivert (Rust crate)** 数据包拦截 / 修改 / 重发 / 过滤。

---

## 目录

1. [项目概述](#1-项目概述)
2. [核心需求](#2-核心需求)
3. [技术选型](#3-技术选型)
4. [系统架构总览](#4-系统架构总览)
5. [后端架构设计 (Rust)](#5-后端架构设计-rust)
6. [前端架构设计 (Vue 3)](#6-前端架构设计-vue-3)
7. [ETW 网络事件跟踪模块](#7-etw-网络事件跟踪模块)
8. [WinDivert 数据包拦截与限速模块](#8-windivert-数据包拦截与限速模块)
9. [预警系统设计](#9-预警系统设计)
10. [数据模型设计](#10-数据模型设计)
11. [Tauri 命令与事件层设计](#11-tauri-命令与事件层设计)
12. [项目目录结构](#12-项目目录结构)
13. [安全与权限](#13-安全与权限)
14. [性能设计](#14-性能设计)
15. [部署与发布](#15-部署与发布)
16. [开发路线图](#16-开发路线图)

---

## 1. 项目概述

**NetTamer（网络驯兽师）** 是一款专为 Windows 平台设计的轻量级桌面端网络监控与流量整形工具。它解决了后台程序偷偷满速上传、吃满带宽的痛点问题。

### 1.1 产品定位

| 维度 | 描述 |
|------|------|
| **目标用户** | Windows 桌面用户、开发者、网络管理人员 |
| **核心价值** | 基于 Windows 原生 API + 用户态数据包驱动，实现进程级网络监控与限速 |
| **竞品对比** | 区别于 NetLimiter 等方案，NetTamer 在用户态即可完成**双向**（上传/下载）限速，无需安装独立的第三方内核驱动 |

### 1.2 核心特性一览

```
┌─────────────────────────────────────────────────────────────┐
│                      NetTamer 核心特性                       │
├─────────────┬──────────────┬───────────────┬────────────────┤
│  📊 实时监控  │  ⚡ 上行预警   │  🚦 双向限速   │  🎨 精美界面   │
│  进程级速率   │  阈值可配置   │  WinDivert    │  shadcn-vue   │
│  上传/下载    │  自动提醒     │  拦截/重发    │  暗色主题     │
└─────────────┴──────────────┴───────────────┴────────────────┘
```

---

## 2. 核心需求

### 2.1 功能需求

#### F1: 进程级实时速率监控
- 显示系统中所有活跃网络进程的 **上传速率** 和 **下载速率**
- 支持按速率、进程名、PID 等维度排序
- 速率刷新频率可配置（默认 1 秒）
- 显示进程图标、路径等辅助信息
- **数据来源**：ETW（`Microsoft-Windows-Kernel-Network`）按 PID 聚合统计

#### F2: 上传速率预警
- 用户可对指定进程或全局设置 **上传速率阈值**
- 当上传速率超出阈值时，触发 **系统级通知弹窗**
- 支持预警规则的增删改查与持久化
- 支持预警历史记录查询

#### F3: 进程级限速（双向）
- 基于 **WinDivert** 对指定进程设置上传 / 下载带宽上限
- 支持拦截、修改、重发数据包与过滤规则处理
- 支持限速策略的创建、修改、删除，即时生效
- 显示当前已生效的限速策略列表

#### F4: 系统托盘与后台运行
- 支持最小化到系统托盘，后台持续监控
- 托盘图标显示当前总上传/下载速率
- 右键菜单快捷操作

### 2.2 非功能需求

| 需求类别 | 要求 |
|----------|------|
| **性能** | CPU 占用 < 2%（空闲态），内存占用 < 50MB |
| **启动速度** | 冷启动 < 3 秒 |
| **兼容性** | Windows 10 1809+ / Windows 11 |
| **权限** | 需要管理员权限（ETW + WinDivert） |
| **安装** | 单文件便携版 + 安装包两种分发方式 |

---

## 3. 技术选型

### 3.1 技术栈总览

```
┌──────────────────────────────────────────────────────────────────┐
│                           技术栈                                  │
├──────────────┬───────────────────────────────────────────────────┤
│ 桌面框架      │ Tauri 2.0                                        │
│ 后端语言      │ Rust (Edition 2021)                              │
│ 前端框架      │ Vue 3 (Composition API) + TypeScript             │
│ UI 组件库     │ shadcn-vue + Tailwind CSS                        │
│ 状态管理      │ Pinia                                            │
│ 图表库        │ Apache ECharts / uPlot                           │
│ 网络监控      │ ETW (windows-rs / windows-sys)                   │
│ 数据包控制    │ WinDivert (Rust crate) — 拦截/修改/重发/过滤      │
│ 数据持久化    │ SQLite (via rusqlite / sqlx)                     │
│ 异步运行时    │ Tokio                                            │
│ 构建工具      │ Vite 5 (前端) + Cargo (后端)                     │
│ 包管理        │ pnpm (前端) / Cargo (后端)                       │
└──────────────┴───────────────────────────────────────────────────┘
```

### 3.2 关键技术选型说明

#### 3.2.1 Tauri 2.0 — 桌面应用框架

**选型理由：**
- 基于 **Rust** 后端 + 系统 **WebView2**（Windows 默认内置），安装包体积极小（~5-10MB），无需 Node.js 运行时
- 安全模型成熟：默认禁止任意系统调用，所有前端↔后端交互经由显式声明的 **Tauri Command**（带类型与权限校验）
- 跨平台原生 API 抽象，插件生态完善（托盘、通知、自动启动、Updater 等）
- 前端技术栈解耦：Vue 3 + shadcn-vue + Tailwind 完全复用，UI 实现零改动
- 原生多窗口、系统托盘、通知中心、自动启动等一等支持

**Tauri 2.0 vs Wails v3（原方案）关键变化：**

| 特性 | Wails v3 (原方案) | Tauri 2.0 (新方案) |
|------|---------|----------|
| 后端语言 | Go | Rust |
| 绑定机制 | 静态源码分析生成 TS 绑定 | `#[tauri::command]` 宏 + 类型安全的 `invoke` |
| 安全模型 | 反射/服务暴露 | 显式命令 + 权限 Capability 配置 |
| 系统托盘 | 原生一等支持 | 原生插件 (`tray-icon`) |
| 通知 | 自定义 | 原生插件 (`notification`) |
| 驱动集成 | 原生调用 | 通过 Rust crate 直接集成 WinDivert |

> **决策**：WinDivert / windows-sys 均为 Rust 原生生态，Tauri 2.0 的 Rust 后端可零摩擦集成，因此后端语言随之由 Go 切换为 Rust。

#### 3.2.2 ETW — 网络事件跟踪（windows-rs / windows-sys）

**选型理由：**
- Windows 原生内核级事件跟踪设施，零额外驱动
- `Microsoft-Windows-Kernel-Network` Provider 直接提供 TCP Send/Recv 事件，携带 PID 和字节数
- 通过 **windows-rs**（高层封装）与 **windows-sys**（零依赖纯绑定、编译期生成）调用 ETW Trace API（`StartTrace` / `OpenTrace` / `ProcessTrace` / `EnableTraceEx2`）
- 高性能、低开销，内核层面数据采集，仅用于 **统计/监控**，不直接干预流量

**Rust 集成方式：**

| 库 | 特点 | 选择 |
|----|------|------|
| `windows-sys` | 纯 FFI 绑定，零运行时依赖，编译期生成 | **首选** ✅ |
| `windows` | 高层安全封装（带类型包装），体积略大 | 备选（便于快速开发时使用） |

> **决策**：ETW 消费者使用 `windows-sys`（特性 `Windows::Win32::System::Diagnostics::Etw`）。如开发期需要更友好的类型封装，可临时切到 `windows` crate，最终发布态回归 `windows-sys` 以减小体积。

#### 3.2.3 WinDivert — 数据包拦截 / 修改 / 重发 / 过滤

**选型理由：**
- WinDivert 是基于 **Windows Filtering Platform (WFP)** 的用户态数据包捕获 / 注入库，可在用户态拦截、修改、重发、丢弃网络数据包
- 通过 **Rust crate**（如 `windivert` / `windivert-sys`，FFI 封装 `WinDivert.dll` + `WinDivert.sys` 驱动）集成，无需编写内核驱动
- 支持 **过滤表达式语言**（filter），可精确匹配进程、协议、端口、方向
- 可实现 **双向限速**（上传 + 下载），弥补原 QoS Policy 仅支持出站的缺陷
- 原生支持「拦截 → 修改 → 重发」链路，为未来数据包改写 / 规则处理预留扩展点

**职责边界：**

| 能力 | 实现方式 | 用途 |
|------|---------|------|
| 拦截 | `WinDivertRecv` 捕获 IP 数据包 | 限速 / 过滤前置 |
| 修改 | 解析并改写包头（端口、地址、载荷标记） | 规则处理、未来扩展 |
| 重发 | `WinDivertSend` 将数据包重新注入网络栈 | 限速缓冲后按速率重投 |
| 过滤 | WinDivert 过滤表达式 | 仅匹配目标进程流量 |

**限制与应对：**

| 限制 | 应对策略 |
|------|---------|
| 需要管理员权限加载驱动 | 应用启动时请求 UAC 提升 |
| 数据包→进程映射需端口→PID 关联 | 结合 ETW 连接事件 + `GetExtendedTcpTable` 维护端口→PID 表 |
| 限速精度依赖丢包/重发策略 | 采用令牌桶 + 受控重发，避免误伤连接 |

#### 3.2.4 shadcn-vue — UI 组件库

**选型理由：**
- 组件以源码形式集成到项目中，完全可定制
- 基于 Radix Vue，无障碍访问原生支持
- 配合 Tailwind CSS 实现高度一致的设计系统
- 暗色/亮色主题一键切换

---

## 4. 系统架构总览

### 4.1 分层架构图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          用户界面层 (Vue 3 + shadcn-vue)                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │ 进程列表  │ │ 速率图表  │ │ 预警配置  │ │ 限速管理  │ │ 系统设置  │     │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘     │
│       └────────────┴────────────┴────────────┴────────────┘            │
│                              │ Tauri Commands + Events                 │
├──────────────────────────────┼──────────────────────────────────────────┤
│                       应用核心层 (Rust / Tokio)                          │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │                     Tauri App (State + Commands)             │      │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────┐  │      │
│  │  │ MonitorSvc │ │ AlertSvc   │ │ ThrottleSvc│ │ ConfigSvc│  │      │
│  │  └─────┬──────┘ └─────┬──────┘ └─────┬──────┘ └────┬─────┘  │      │
│  └────────┼──────────────┼──────────────┼─────────────┼────────┘      │
│           │              │              │             │                │
├───────────┼──────────────┼──────────────┼─────────────┼────────────────┤
│           ▼              ▼              ▼             ▼                │
│                       核心引擎层 (Rust)                                   │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐  │
│  │  ETW Engine  │ │ Alert Engine │ │ WinDivert    │ │ Store Engine │  │
│  │ (windows-sys │ │ (规则引擎)   │ │  Engine      │ │ (SQLite)     │  │
│  │  事件消费)    │ │             │ │ (拦截/重发/   │ │              │  │
│  │              │ │             │ │  过滤/限速)   │ │              │  │
│  └──────┬───────┘ └──────────────┘ └──────┬───────┘ └──────────────┘  │
│         │                                  │                           │
├─────────┼──────────────────────────────────┼───────────────────────────┤
│         ▼                                  ▼                           │
│                     操作系统层 (Windows)                                 │
│  ┌────────────────────────────┐  ┌────────────────────────────┐       │
│  │ ETW Kernel Session         │  │ WinDivert (用户态 + WFP)    │       │
│  │ Microsoft-Windows-          │  │ WinDivert.dll              │       │
│  │   Kernel-Network           │  │   ↓ WinDivert.sys          │       │
│  │ GUID: {7dd42a49-5329-      │  │ WFP (TCPIP / FWPM)         │       │
│  │   4832-8dfd-43d979153a88}  │  │ 数据包 拦截/注入/过滤        │       │
│  └────────────────────────────┘  └────────────────────────────┘       │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4.2 数据流架构

```
                    ┌───────────────────────────────────┐
                    │          Windows Kernel            │
                    │   TCP Send / TCP Recv Events       │
                    └──────────────┬────────────────────┘
                                   │ ETW Real-time Session (windows-sys)
                                   ▼
                    ┌───────────────────────────────────┐
                    │       ETW Consumer (Rust)          │
                    │  解析事件 → 提取 PID + 字节数        │
                    │  维护 端口→PID 映射表               │
                    └──────────────┬────────────────────┘
                                   │
                    ┌──────────────▼────────────────────┐
                    │       Aggregator (聚合器)           │
                    │  按 PID 聚合 → 计算瞬时速率          │
                    │  滑动窗口 (1s/5s/30s)              │
                    └──────┬───────────────┬────────────┘
                           │               │
              ┌────────────▼───┐   ┌───────▼───────────┐
              │   Alert Engine │   │   Speed Cache      │
              │   检查预警规则   │   │   进程速率缓存      │
              └────────┬───────┘   └───────┬───────────┘
                       │                   │
              ┌────────▼───────┐   ┌───────▼───────────┐
              │ Windows Toast  │   │ Tauri Event       │
              │ 系统通知弹窗    │   │ 推送到前端渲染      │
              └────────────────┘   └───────────────────┘

                    ┌───────────────────────────────────┐
                    │       WinDivert Engine (Rust)      │
                    │  WinDivertRecv → 分类(端口→PID)    │
                    │  ├─ 未限速: 直接 WinDivertSend 重发 │
                    │  └─ 已限速: 令牌桶裁决 → 丢弃/缓冲  │
                    │                    → 按速率 WinDivertSend │
                    └───────────────────────────────────┘
```

---

## 5. 后端架构设计 (Rust)

### 5.1 模块划分

后端采用 **分层 + 模块化** 架构（Rust crate 内部 module），各模块职责明确、低耦合：

```
src-tauri/src/
├── etw/            # ETW 事件跟踪模块 (windows-sys)
├── monitor/        # 速率监控与聚合模块
├── windivert/      # 数据包拦截 / 修改 / 重发 / 过滤模块
├── throttle/       # 限速策略编排（基于 WinDivert）
├── alert/          # 预警引擎模块
├── process/        # 进程信息查询 + 端口→PID 映射
├── store/          # 数据持久化模块 (SQLite)
├── config/         # 配置管理模块
├── notify/         # 系统通知模块
├── tray/           # 系统托盘模块
└── commands/       # Tauri Command 暴露层
```

### 5.2 核心模块详细设计

#### 5.2.1 ETW 模块 (`etw`)

```rust
// etw/session.rs — ETW 会话管理 (基于 windows-sys)

/// Microsoft-Windows-Kernel-Network 的 Provider GUID
pub const KERNEL_NETWORK_PROVIDER: windows_sys::core::GUID = windows_sys::core::GUID {
    data1: 0x7dd42a49,
    data2: 0x5329,
    data3: 0x4832,
    data4: [0x8d, 0xfd, 0x43, 0xd9, 0x79, 0x15, 0x3a, 0x88],
};

/// 一次 TCP/UDP 网络事件
#[derive(Debug, Clone)]
pub struct NetworkEvent {
    pub timestamp: std::time::SystemTime,
    pub pid: u32,
    pub direction: Direction,   // Send / Recv
    pub size: u32,              // 字节数
    pub local_addr: std::net::SocketAddr,
    pub remote_addr: std::net::SocketAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction { Send, Recv }

/// ETW 实时事件会话
pub struct Session {
    name: String,
    event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
    // 内部持有 trace handle，Drop 时自动 StopTrace
}

impl Session {
    /// 创建并启动一个 ETW 实时事件跟踪会话
    pub fn start(buffer_size: u32) -> Result<(Self, Receiver<NetworkEvent>), EtwError> { todo!() }
    /// 停止事件跟踪会话并释放资源
    pub fn stop(self) -> Result<(), EtwError> { todo!() }
}
```

#### 5.2.2 监控聚合模块 (`monitor`)

```rust
// monitor/aggregator.rs — 按进程聚合网络速率

/// 单个进程的网络统计快照
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProcessStats {
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub upload_rate: f64,      // bytes/sec
    pub download_rate: f64,    // bytes/sec
    pub total_upload: u64,
    pub total_download: u64,
}

/// 速率聚合器
pub struct Aggregator {
    processes: std::sync::RwLock<HashMap<u32, ProcessAccumulator>>,
    window: std::time::Duration,
}

impl Aggregator {
    pub fn new(window: std::time::Duration) -> Self { todo!() }
    /// 接收 ETW 原始事件并累加
    pub fn ingest(&self, ev: NetworkEvent) { todo!() }
    /// 返回当前所有进程的速率快照（线程安全）
    pub fn snapshot(&self) -> Vec<ProcessStats> { todo!() }
    /// 返回按指定维度排序的前 N 个进程
    pub fn top_n(&self, n: usize, sort_by: SortField, order: SortOrder) -> Vec<ProcessStats> { todo!() }
}
```

**速率计算算法：**

```
使用 滑动窗口 + 指数加权移动平均 (EWMA) 实现平滑速率计算：

    rate_t = α × instant_rate + (1 - α) × rate_(t-1)

    其中 α = 2 / (N + 1), N 为窗口采样数
    默认 N = 5 (对应 5 秒窗口，1 秒采样间隔)

优点：
  - 避免瞬时毛刺导致的速率跳变
  - 响应灵敏度可通过 α 参数调节
```

#### 5.2.3 WinDivert 引擎模块 (`windivert`)

```rust
// windivert/engine.rs — 数据包拦截 / 修改 / 重发 / 过滤

use windivert::WinDivert;

/// 捕获层
#[derive(Debug, Clone, Copy)]
pub enum Layer { Network, NetworkForward }

/// WinDivert 引擎
pub struct WinDivertEngine {
    handle: WinDivert,
    throttle: Arc<ThrottleTable>,   // 进程 → 速率上限
    port_map: Arc<PortPidMap>,      // 端口 → PID（来自 ETW / IPHelper）
    tx: tokio::sync::mpsc::Sender<Packet>,
}

impl WinDivertEngine {
    /// 以过滤表达式打开 WinDivert（如 "tcp or udp" 或按进程）
    pub fn open(filter: &str, layer: Layer) -> Result<Self, WinDivertError> { todo!() }

    /// 捕获循环：WinDivertRecv → 解析 → 分类 → 限速裁决 → 重发/丢弃
    pub async fn run(&self) { todo!() }

    /// 修改数据包头（地址/端口/标记）后通过 WinDivertSend 重发
    pub fn modify_and_send(&self, pkt: &mut Packet) -> Result<(), WinDivertError> { todo!() }

    /// 应用/更新过滤规则
    pub fn set_filter(&self, filter: &str) -> Result<(), WinDivertError> { todo!() }

    pub fn stop(&self) -> Result<(), WinDivertError> { todo!() }
}
```

#### 5.2.4 限速编排模块 (`throttle`)

```rust
// throttle/manager.rs — 基于 WinDivert 的限速策略管理

/// 限速策略
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub process_name: String,       // 可执行文件名 (如 "chrome.exe")
    pub rate_limit_bps: u64,        // 限速值 (bits/sec)，0 表示不限
    pub limit_upload: bool,
    pub limit_download: bool,
    pub active: bool,
    pub created_at: i64,
}

/// 限速表（进程 → 令牌桶）
pub struct ThrottleTable {
    buckets: std::sync::RwLock<HashMap<u32, TokenBucket>>,
}

impl ThrottleTable {
    pub fn apply_policy(&self, policy: Policy, pid: u32) { todo!() }
    pub fn remove_policy(&self, id: &str) { todo!() }
    pub fn list_policies(&self) -> Vec<Policy> { todo!() }
    /// 令牌桶裁决：true = 允许通过，false = 需丢弃/缓冲
    pub fn admit(&self, pid: u32, bytes: usize) -> bool { todo!() }
}
```

**WinDivert 调用封装（令牌桶 + 受控重发）：**

```rust
// windivert/rate_limiter.rs — 令牌桶限速裁决

/// 令牌桶
pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_rate: f64,   // tokens/sec == bytes/sec
    last: std::time::Instant,
}

impl TokenBucket {
    pub fn new(rate_bps: u64) -> Self {
        let rate = rate_bps as f64 / 8.0; // bits/sec → bytes/sec
        Self { capacity: rate, tokens: rate, refill_rate: rate, last: std::time::Instant::now() }
    }
    /// 尝试领取 n 字节的令牌，成功返回 true
    pub fn try_consume(&mut self, n: usize) -> bool { todo!() }
}
```

#### 5.2.5 进程信息模块 (`process`)

```rust
// process/info.rs — 进程元数据 + 端口→PID 映射

/// 进程基础信息
pub struct Info {
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub icon_b64: String,   // Base64 编码的进程图标
    pub user: String,
}

/// 端口→PID 映射维护器（结合 ETW 连接事件 与 GetExtendedTcpTable）
pub struct PortPidMap {
    map: std::sync::RwLock<HashMap<SocketAddr, u32>>,
}
impl PortPidMap {
    pub fn lookup(&self, addr: &SocketAddr) -> Option<u32> { todo!() }
    pub fn refresh(&self) { todo!() }
}
```

#### 5.2.6 数据持久化模块 (`store`)

```rust
// store/store.rs — SQLite 数据存储 (rusqlite)

pub struct Db {
    pool: r2d2::Pool<rusqlite::ConnectionManager>,
}

pub trait AlertStore {
    fn save_rule(&self, rule: &alert::Rule) -> Result<()>;
    fn delete_rule(&self, id: &str) -> Result<()>;
    fn list_rules(&self) -> Result<Vec<alert::Rule>>;
    fn save_alert_event(&self, ev: &alert::AlertEvent) -> Result<()>;
    fn list_alert_events(&self, filter: AlertEventFilter) -> Result<Vec<alert::AlertEvent>>;
}

pub trait ThrottleStore {
    fn save_policy(&self, p: &throttle::Policy) -> Result<()>;
    fn delete_policy(&self, id: &str) -> Result<()>;
    fn list_policies(&self) -> Result<Vec<throttle::Policy>>;
}

pub trait ConfigStore {
    fn get(&self, key: &str) -> Result<Option<String>>;
    fn set(&self, key: &str, value: &str) -> Result<()>;
}
```

---

## 6. 前端架构设计 (Vue 3)

### 6.1 技术栈

| 技术 | 用途 |
|------|------|
| Vue 3 + Composition API | 核心框架 |
| TypeScript | 类型安全 |
| shadcn-vue | UI 组件库（源码集成） |
| Tailwind CSS | 原子化样式 |
| Pinia | 状态管理 |
| Vue Router | 页面路由 |
| uPlot / ECharts | 实时速率图表 |
| VueUse | 通用组合式函数 |
| Tauri JS API (`@tauri-apps/api`) | 命令调用 + 事件监听 |

### 6.2 页面设计

```
┌─────────────────────────────────────────────────────────────────┐
│  NetTamer                              ─  □  ×  │
├────────┬────────────────────────────────────────────────────────┤
│        │                                                        │
│  📊    │  ┌──────────────────────────────────────────────────┐  │
│ 仪表盘  │  │            总上传: 2.5 MB/s  总下载: 15.3 MB/s    │  │
│        │  └──────────────────────────────────────────────────┘  │
│  📋    │  ┌──────────────────────────────────────────────────┐  │
│ 进程    │  │  ▂▃▅▆█▇▅▃▂▁▂▃▅▆█  实时速率折线图                  │  │
│ 列表    │  └──────────────────────────────────────────────────┘  │
│        │  ┌──────────────────────────────────────────────────┐  │
│  ⚠️    │  │  进程名      PID   ↑上传     ↓下载    操作       │  │
│ 预警    │  │  chrome.exe  1234  1.2MB/s  5.3MB/s  [限速][预警]│  │
│ 配置    │  │  steam.exe   5678  800KB/s  12MB/s   [限速][预警]│  │
│        │  │  svchost.exe  890  200KB/s  100KB/s  [限速][预警]│  │
│  🚦    │  │  ...                                             │  │
│ 限速    │  └──────────────────────────────────────────────────┘  │
│ 管理    │                                                        │
│        │                                                        │
│  ⚙️    │                                                        │
│ 设置    │                                                        │
│        │                                                        │
└────────┴────────────────────────────────────────────────────────┘
```

### 6.3 页面路由

| 路由 | 页面 | 功能 |
|------|------|
| `/` | Dashboard | 仪表盘总览，实时速率图表 + Top 进程 |
| `/processes` | ProcessList | 全部活跃进程列表，支持搜索、排序、操作 |
| `/alerts` | AlertConfig | 预警规则管理 + 预警历史 |
| `/throttle` | ThrottleManager | 限速策略管理 |
| `/settings` | Settings | 全局设置（刷新频率、主题、开机启动等） |

### 6.4 前端目录结构

```
frontend/src/
├── assets/                  # 静态资源
│   ├── fonts/
│   └── images/
├── components/              # 全局共享组件
│   ├── ui/                  # shadcn-vue 组件（自动生成）
│   ├── layout/              # 布局组件
│   ├── charts/              # 图表组件
│   └── common/              # 通用业务组件
├── composables/             # 组合式函数（封装 Tauri invoke/listen）
│   ├── useProcessMonitor.ts # 封装 Tauri 命令: 进程监控
│   ├── useAlertRules.ts     # 封装 Tauri 命令: 预警规则
│   ├── useThrottle.ts       # 封装 Tauri 命令: 限速管理
│   ├── useConfig.ts         # 封装 Tauri 命令: 配置管理
│   └── useFormatters.ts     # 速率格式化 (bytes → KB/s, MB/s)
├── lib/                     # 工具函数
│   └── utils.ts             # shadcn-vue cn() 工具
├── router/                  # 路由配置
├── stores/                  # Pinia 状态管理
│   ├── processStore.ts
│   ├── alertStore.ts
│   ├── throttleStore.ts
│   └── settingsStore.ts
├── views/                   # 页面视图
├── App.vue
├── main.ts
└── style.css                # Tailwind 入口 + 全局样式
```

### 6.5 状态管理设计 (Pinia)

```typescript
// stores/processStore.ts

interface ProcessStats {
  pid: number
  name: string
  path: string
  iconB64: string
  uploadRate: number      // bytes/sec
  downloadRate: number    // bytes/sec
  totalUpload: number
  totalDownload: number
}

interface ProcessState {
  processes: ProcessStats[]
  totalUploadRate: number
  totalDownloadRate: number
  isMonitoring: boolean
  refreshInterval: number   // ms
  sortField: 'uploadRate' | 'downloadRate' | 'name' | 'pid'
  sortOrder: 'asc' | 'desc'
  searchQuery: string
}

export const useProcessStore = defineStore('process', () => {
  // 通过 @tauri-apps/api/event 的 listen() 接收后端推送的实时数据
  // 前端不轮询，后端定时推送
})
```

### 6.6 前后端通信机制

NetTamer 的前后端通信采用 **Tauri 命令（Command）+ 事件（Event）** 机制：

- **方法调用**：前端通过 `@tauri-apps/api/core` 的 `invoke()` 调用后端 `#[tauri::command]`，类型安全（TS 侧可手写类型或使用代码生成）
- **事件推送**：后端通过 `app.emit()` 向前端推送实时数据，前端通过 `@tauri-apps/api/event` 的 `listen()` 监听

```
┌─────────────┐                    ┌─────────────┐
│   Vue 前端   │                    │  Rust 后端   │
├─────────────┤                    ├─────────────┤
│             │  ── invoke ──→      │             │
│ composable  │  (类型安全)         │ #[tauri::   │
│ useXxx()    │                    │  command]   │
│             │  ←── 返回值 ──      │             │
│             │                    │             │
│             │  ←── emit 事件 ──   │             │
│ listen()    │  app.emit          │ emit 定时   │
│             │  ("speed:update",  │ 推送速率    │
│             │    processStats)   │ 数据        │
└─────────────┘                    └─────────────┘
```

**关键事件定义：**

| 事件名 | 方向 | 数据 | 频率 |
|--------|------|------|------|
| `speed:update` | 后端 → 前端 | `ProcessStats[]` | 1s |
| `alert:triggered` | 后端 → 前端 | `AlertEvent` | 按需 |
| `throttle:changed` | 后端 → 前端 | `Policy` | 按需 |
| `system:stats` | 后端 → 前端 | 总速率、CPU 等 | 1s |

---

## 7. ETW 网络事件跟踪模块

### 7.1 ETW Provider 详情

```
Provider:   Microsoft-Windows-Kernel-Network
GUID:       {7dd42a49-5329-4832-8dfd-43d979153a88}
Session:    Real-time (非文件记录模式)

关键事件:
┌────────┬────────────────────┬──────────────────────────────────┐
│ EID    │ 事件名              │ 包含字段                          │
├────────┼────────────────────┼──────────────────────────────────┤
│ 10     │ TcpIp/Send         │ PID, size, saddr, sport,        │
│        │                    │ daddr, dport, connid            │
├────────┼────────────────────┼──────────────────────────────────┤
│ 11     │ TcpIp/Recv         │ PID, size, saddr, sport,        │
│        │                    │ daddr, dport, connid            │
├────────┼────────────────────┼──────────────────────────────────┤
│ 12     │ UdpIp/Send         │ PID, size, saddr, sport,        │
│        │                    │ daddr, dport                    │
├────────┼────────────────────┼──────────────────────────────────┤
│ 13     │ UdpIp/Recv         │ PID, size, saddr, sport,        │
│        │                    │ daddr, dport                    │
└────────┴────────────────────┴──────────────────────────────────┘
```

### 7.2 事件处理流水线

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│ ETW      │    │ Decoder  │    │ Filter   │    │ Agg      │    │ Dispatch │
│ Session  │───▶│ 事件解码  │───▶│ 过滤无效  │───▶│ PID聚合  │───▶│ 分发事件  │
│          │    │ 字节解析  │    │ PID=0等  │    │ 速率计算  │    │ 前端+预警 │
└──────────┘    └──────────┘    └──────────┘    └──────────┘    └──────────┘
     ▲               │                                               │
     │          高吞吐场景                                             │
     │          批量处理                                               ▼
     │          Ring Buffer                                     ┌──────────┐
     │                                                          │ EventBus │
     └──────────────────────────────────────────────────────────┘
```

### 7.3 性能优化策略

| 策略 | 说明 |
|------|------|
| **Ring Buffer** | ETW 事件先写入无锁环形缓冲区，消费者批量读取，避免 channel 锁争用 |
| **批量聚合** | 每 100ms 批量处理一次累积事件，而非逐条处理 |
| **PID 缓存** | 进程名/路径/图标等元信息使用 LRU 缓存，避免重复系统调用 |
| **端口→PID 表** | 由 ETW 连接事件 + `GetExtendedTcpTable` 共同维护，供 WinDivert 限速时查表 |
| **采样降级** | 当事件速率超过阈值（如 10万条/秒）时，自动降低采样率 |
| **零分配解析** | 事件解析过程中尽量复用 buffer，减少堆分配压力 |

---

## 8. WinDivert 数据包拦截与限速模块

### 8.1 WinDivert 工作原理

```
用户空间                                   内核空间 (WFP)
┌──────────────────────┐            ┌──────────────────────────┐
│ NetTamer             │            │   TCP/IP Stack / WFP      │
│                      │            │                          │
│  WinDivert.dll ──────┼──────────▶ │  WFP Filter (callout)    │
│   WinDivertRecv      │ 捕获数据包  │  数据包被重定向到用户态    │
│                      │            │                          │
│  ┌────────────────┐  │            │                          │
│  │ 分类(端口→PID) │  │            │                          │
│  │ 令牌桶裁决     │  │            │                          │
│  └──────┬─────────┘  │            │                          │
│         │ 放行        │            │                          │
│         │ 丢弃        │            │                          │
│         │ 修改+重发 ───┼──────────▶ │  WinDivertSend 重新注入   │
│         │             │            │                          │
└──────────────────────┘            └──────────────────────────┘
         ▲                                   │
         │                          ┌────────▼──────────┐
         │                          │   Network NIC     │
         │                          └───────────────────┘
```

WinDivert 基于 **Windows Filtering Platform (WFP)**，在用户态即可：
- **拦截 (Recv)**：从网络栈捕获 IP 数据包
- **修改**：解析并改写包头（源/目的地址、端口、可选载荷标记）
- **重发 (Send)**：将（可能修改后的）数据包重新注入网络栈
- **过滤**：通过 WinDivert 过滤表达式精确匹配目标流量

### 8.2 限速实现策略

WinDivert 限速采用 **令牌桶 + 受控重发**，支持 **上传 (Egress) 与下载 (Ingress) 双向**：

```
对每个被限速进程维护一个 TokenBucket（速率 = rate_limit_bps / 8 bytes/sec）：

  数据包到达 (WinDivertRecv):
    1. 通过 端口→PID 表 定位所属进程
    2. 查该进程是否处于限速策略（且方向匹配）
    3. TokenBucket.try_consume(packet_len)?
       ├─ 成功  → 直接/修改后 WinDivertSend 放行
       └─ 失败  → 上传方向: 直接丢弃（触发 TCP 拥塞控制自然降速）
                 下载方向: 缓冲到有界队列，按令牌补充节奏 WinDivertSend 重发
```

| 方向 | 实现 | 说明 |
|------|------|------|
| 上传 (Egress) | `WinDivertOpen("outbound", ...)` 捕获出站包 | 超量直接丢弃即可有效限速 |
| 下载 (Ingress) | `WinDivertOpen("inbound", ...)` 捕获入站包 | 采用缓冲 + 受控重发，避免连接中断 |

### 8.3 过滤规则与策略命名

WinDivert 过滤表达式示例：

```
# 仅捕获 TCP/UDP 流量
"tcp or udp"

# 仅捕获某端口段（可结合端口→PID 表在应用层细分到进程）
"tcp.DstPort >= 1 and tcp.DstPort <= 65535"
```

限速策略在 SQLite 中以 `Policy` 结构持久化（见 §10），并通过 `ThrottleTable` 维护运行时令牌桶。策略命名使用统一前缀避免冲突：

```
策略名称格式: NT_{ProcessName}_{UniqueID}
示例:
  NT_chrome_a1b2c3
  NT_steam_d4e5f6
```

### 8.4 策略生命周期管理

```
┌────────┐    ┌────────┐    ┌────────┐    ┌────────┐
│ Create │───▶│ Active │───▶│ Update │───▶│ Active │
│ 创建策略│    │ 策略生效│    │ 修改策略│    │ 重新生效│
└────────┘    └───┬────┘    └────────┘    └────────┘
                  │
                  │ 用户移除 / 应用退出
                  ▼
              ┌────────┐
              │ Remove │
              │ 清理策略│
              └────────┘
```

应用退出时的清理策略：
1. 停止 WinDivert 捕获循环，关闭 handle
2. 清空 `ThrottleTable` 令牌桶
3. 同步更新本地数据库状态

### 8.5 过滤/修改扩展点

> **预留能力**
>
> WinDivert 的「修改 + 重发」链路为未来的规则处理预留空间：
> - 数据包改写（端口映射、标记）
> - 基于规则的流量重定向 / 阻断
> - 自定义过滤表达式 DSL（前端配置 → 后端 `set_filter`）

| 能力 | v2.0 状态 | 说明 |
|------|----------|------|
| 上传限速 | ✅ | WinDivert 令牌桶 + 丢弃 |
| 下载限速 | ✅ | WinDivert 缓冲 + 受控重发 |
| 数据包修改/重发 | 🔧 基础 | 框架已具备，界面/规则待扩展 |

---

## 9. 预警系统设计

### 9.1 预警规则模型

```
┌─────────────────────────────────────────┐
│              预警规则 (Rule)              │
├──────────────┬──────────────────────────┤
│ 匹配条件      │ 进程名 (支持通配符 *)     │
├──────────────┼──────────────────────────┤
│ 触发条件      │ 上传速率 > 阈值 (bytes/s) │
│              │ 持续时间 > N 秒           │
├──────────────┼──────────────────────────┤
│ 动作         │ 系统通知弹窗              │
│              │ 前端 Toast 提醒          │
│              │ (可选) 自动应用限速策略    │
├──────────────┼──────────────────────────┤
│ 冷却控制      │ 冷却时间 (默认 60s)       │
└──────────────┴──────────────────────────┘
```

### 9.2 预警判定流程

```rust
// 伪代码: 预警判定逻辑 (Rust)

impl Engine {
    pub fn evaluate(&self, stats: &[ProcessStats]) {
        for stat in stats {
            for rule in self.rules.values() {
                if !rule.enabled { continue; }
                if !match_process(&rule.process_name, &stat.name) { continue; }

                let rate = stat.upload_rate;
                if rate > rule.threshold {
                    let key = format!("{}:{}", rule.id, stat.name);
                    if let Some(last) = self.cooldowns.get(&key) {
                        if last.elapsed() < rule.cooldown() { continue; }
                    }
                    let event = AlertEvent { /* ... */ };
                    self.alert_tx.send(event.clone()).ok();
                    self.cooldowns.insert(key, Instant::now());
                    self.store.save_alert_event(&event).ok();
                }
            }
        }
    }
}
```

### 9.3 通知方式

| 方式 | 实现 | 说明 |
|------|------|------|
| Windows Toast 通知 | Tauri `notification` 插件 / Win32 API | 系统级弹窗，即使应用最小化也可见 |
| 应用内 Toast | shadcn-vue `<Toast>` | 应用窗口内的轻量提醒 |
| 声音提示 | 系统音效 | 可配置开关 |
| 自动限速 | ThrottleTable | 可选：触发预警后自动创建限速策略 |

---

## 10. 数据模型设计

### 10.1 SQLite 表结构

```sql
-- 预警规则表
CREATE TABLE alert_rules (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    process_name TEXT NOT NULL,        -- 进程名匹配模式
    threshold   REAL NOT NULL,         -- 阈值 (bytes/sec)
    direction   INTEGER DEFAULT 0,     -- 0=Upload, 1=Download, 2=Both
    cooldown_sec INTEGER DEFAULT 60,
    enabled     INTEGER DEFAULT 1,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 预警事件历史表
CREATE TABLE alert_events (
    id           TEXT PRIMARY KEY,
    rule_id      TEXT NOT NULL,
    process_name TEXT NOT NULL,
    pid          INTEGER NOT NULL,
    current_rate REAL NOT NULL,
    threshold    REAL NOT NULL,
    triggered_at DATETIME NOT NULL,
    FOREIGN KEY (rule_id) REFERENCES alert_rules(id) ON DELETE CASCADE
);
CREATE INDEX idx_alert_events_time ON alert_events(triggered_at DESC);

-- 限速策略表
CREATE TABLE throttle_policies (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL UNIQUE,   -- 策略名 (NT_ 前缀)
    process_name  TEXT NOT NULL,          -- 可执行文件名
    rate_limit_bps INTEGER NOT NULL,      -- 限速值 (bits/sec)
    limit_upload  INTEGER DEFAULT 1,
    limit_download INTEGER DEFAULT 1,
    active        INTEGER DEFAULT 1,
    created_at    DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at    DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 配置表 (KV 存储)
CREATE TABLE config (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- 预填充默认配置
INSERT INTO config (key, value) VALUES
    ('refresh_interval_ms', '1000'),
    ('theme', 'dark'),
    ('auto_start', 'false'),
    ('minimize_to_tray', 'true'),
    ('alert_sound', 'true');
```

> SQLite 实现采用 `rusqlite`（同步，配 `r2d2` 连接池）或 `sqlx`（异步）。数据库文件存放于用户 AppData 目录。

---

## 11. Tauri 命令与事件层设计

### 11.1 Command 架构

Tauri 2.0 采用显式 **Command** 暴露后端能力。所有命令在 `lib.rs` 中通过 `#[tauri::command]` 定义，并在 `tauri::Builder` 中 `invoke_handler(tauri::generate_handler![...])` 注册。通过 **Capability** 配置限定前端可调用范围，强化安全。

```rust
// src-tauri/src/lib.rs — Tauri 应用入口

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())   // 系统通知
        .plugin(tauri_plugin_tray_icon::init())       // 系统托盘
        .plugin(tauri_plugin_autostart::init(...))    // 开机自启
        .setup(|app| {
            // 初始化 ETW 会话、WinDivert 引擎、Store、Aggregator
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_monitoring,
            stop_monitoring,
            get_process_list,
            create_alert_rule,
            list_alert_rules,
            apply_throttle_policy,
            remove_throttle_policy,
            list_throttle_policies,
            get_config,
            set_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running NetTamer");
}
```

### 11.2 命令示例

```rust
// commands/monitor.rs

#[tauri::command]
pub async fn start_monitoring(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.etw_session.start().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_process_list(state: tauri::State<'_, AppState>) -> Result<Vec<ProcessStats>, String> {
    Ok(state.aggregator.snapshot())
}

#[tauri::command]
pub async fn apply_throttle_policy(
    state: tauri::State<'_, AppState>,
    policy: Policy,
) -> Result<(), String> {
    state.throttle.apply(policy).map_err(|e| e.to_string())
}
```

### 11.3 命令清单

#### 监控相关

| 命令 | 签名 | 说明 |
|------|------|------|
| `start_monitoring` | `() -> Result<(), String>` | 启动 ETW 监控 |
| `stop_monitoring` | `() -> Result<(), String>` | 停止 ETW 监控 |
| `get_process_list` | `() -> Result<Vec<ProcessStats>, String>` | 获取当前进程列表快照 |
| `set_refresh_interval` | `(ms: u64) -> Result<(), String>` | 设置刷新频率 |

#### 预警相关

| 命令 | 签名 | 说明 |
|------|------|------|
| `create_alert_rule` | `(rule: Rule) -> Result<(), String>` | 创建预警规则 |
| `update_alert_rule` | `(rule: Rule) -> Result<(), String>` | 更新预警规则 |
| `delete_alert_rule` | `(id: String) -> Result<(), String>` | 删除预警规则 |
| `list_alert_rules` | `() -> Result<Vec<Rule>, String>` | 获取所有预警规则 |
| `get_alert_history` | `(filter: Filter) -> Result<Vec<AlertEvent>, String>` | 查询预警历史 |

#### 限速相关

| 命令 | 签名 | 说明 |
|------|------|------|
| `apply_throttle_policy` | `(policy: Policy) -> Result<(), String>` | 创建/更新限速策略（驱动 WinDivert 令牌桶） |
| `remove_throttle_policy` | `(id: String) -> Result<(), String>` | 移除限速策略 |
| `list_throttle_policies` | `() -> Result<Vec<Policy>, String>` | 列出所有限速策略 |

#### 配置相关

| 命令 | 签名 | 说明 |
|------|------|------|
| `get_config` | `(key: String) -> Result<Option<String>, String>` | 获取配置项 |
| `set_config` | `(key: String, value: String) -> Result<(), String>` | 设置配置项 |
| `get_all_config` | `() -> Result<HashMap<String, String>, String>` | 获取所有配置 |

---

## 12. 项目目录结构

```
NetTamer/
├── src-tauri/                       # Rust 后端 (Tauri 2.0)
│   ├── Cargo.toml                   # Rust 依赖 (tauri, windows-sys, windivert, rusqlite, tokio, serde)
│   ├── build.rs                     # Tauri 构建脚本
│   ├── tauri.conf.json              # Tauri 配置 (窗口/权限/插件)
│   ├── capabilities/                # 权限 Capability 配置
│   │   └── default.json
│   ├── icons/                       # 应用图标
│   ├── src/
│   │   ├── main.rs                  # 入口 (调用 lib::run)
│   │   ├── lib.rs                   # Tauri Builder + 插件 + 命令注册
│   │   ├── state.rs                 # 全局共享状态 (AppState)
│   │   ├── etw/                     # ETW 事件跟踪 (windows-sys)
│   │   │   ├── session.rs
│   │   │   ├── decoder.rs
│   │   │   └── provider.rs
│   │   ├── monitor/                 # 速率监控与聚合
│   │   │   ├── aggregator.rs
│   │   │   └── ewma.rs
│   │   ├── windivert/               # 数据包拦截/修改/重发/过滤
│   │   │   ├── engine.rs
│   │   │   └── rate_limiter.rs      # 令牌桶
│   │   ├── throttle/                # 限速策略编排
│   │   │   └── manager.rs
│   │   ├── alert/                   # 预警引擎
│   │   │   ├── engine.rs
│   │   │   └── matcher.rs
│   │   ├── process/                 # 进程信息 + 端口→PID
│   │   │   ├── info.rs
│   │   │   └── port_map.rs
│   │   ├── store/                   # 数据存储 (SQLite)
│   │   │   ├── store.rs
│   │   │   └── migrations.rs
│   │   ├── config/                  # 配置管理
│   │   │   └── config.rs
│   │   ├── notify/                  # 系统通知
│   │   │   └── toast.rs
│   │   ├── tray/                    # 系统托盘
│   │   │   └── tray.rs
│   │   └── commands/                # Tauri Command 暴露层
│   │       ├── monitor.rs
│   │       ├── alert.rs
│   │       ├── throttle.rs
│   │       └── config.rs
│   └── bin/                         # WinDivert 驱动/二进制随包分发
│       └── WinDivert.dll / WinDivert.sys
│
├── package.json                     # 前端依赖与脚本 (dev/build/tauri)
├── index.html                       # 前端入口
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
├── tsconfig.json
├── tsconfig.node.json
├── components.json                  # shadcn-vue 配置
├── public/                          # 静态资源 (favicon 等)
├── src/                             # Vue 3 前端源码 (与 src-tauri 平级，官方布局)
│   ├── lib/
│   │   ├── ipc.ts                   # Tauri invoke/listen 安全封装
│   │   └── utils.ts                 # cn() 等工具
│   ├── types.ts                     # 前后端共享数据契约
│   ├── components/
│   │   ├── ui/                      # shadcn-vue 组件
│   │   ├── layout/
│   │   ├── charts/
│   │   └── common/
│   ├── composables/                 # 封装 Tauri 命令
│   ├── router/
│   ├── stores/                      # Pinia 状态
│   ├── views/                       # 页面视图
│   ├── App.vue
│   ├── main.ts
│   └── style.css
│
├── doc/                             # 项目文档
│   └── architecture.md              # 架构设计文档 (本文件)
├── README.md
└── .gitignore
```

---

## 13. 安全与权限

### 13.1 UAC 管理员权限

NetTamer 的核心功能（ETW 和 WinDivert）均需要管理员权限：

- **ETW**：实时内核会话需要 `SeSystemProfilePrivilege` / 管理员组
- **WinDivert**：加载 `WinDivert.sys` 驱动需要管理员权限

Tauri 2.0 已从 `tauri.conf.json` 中移除 `bundle.windows.requestedExecutionLevel` 字段，
管理员权限改由编译期嵌入**应用清单 (manifest)** 实现。在 `src-tauri/build.rs` 中通过
`tauri_build::WindowsAttributes::app_manifest()` 嵌入 `src-tauri/manifest.xml`
（含 `requestedExecutionLevel level="requireAdministrator"`）：

```rust
// src-tauri/build.rs (Windows 分支)
use tauri_build::WindowsAttributes;
let windows = WindowsAttributes::new().app_manifest(include_str!("manifest.xml"));
let attrs = tauri_build::Attributes::new().windows_attributes(windows);
tauri_build::try_build(attrs).expect("failed to run tauri-build");
```

> 注意：因 exe 内嵌了 `requireAdministrator` 清单，**必须以管理员身份**运行
> `tauri dev` / `tauri build`（即从“管理员: 终端”启动），否则 Windows 会返回
> `os error 740`（请求的操作需要提升）。

### 13.2 安全设计考量

| 风险 | 缓解措施 |
|------|---------|
| 命令越权调用 | Tauri 2.0 Capability 机制限定前端可调用命令集合 |
| WinDivert 驱动加载 | 仅使用官方签名驱动；随包分发并校验版本，禁止从网络动态下载 |
| 数据包误丢/误改 | 限速仅在明确策略下生效；令牌桶策略可灰度和回滚 |
| 数据库安全 | SQLite 参数化查询，数据库文件存放于用户 AppData 目录 |
| 进程信息泄露 | 仅在本地 WebView 中渲染，Tauri 默认不暴露任何网络端口 |

### 13.3 WinDivert 驱动部署

```
随包分发文件:
  WinDivert.dll   (用户态库)
  WinDivert.sys   (内核驱动，需管理员加载)

首次启动时:
  1. 校验驱动文件签名与版本
  2. 以管理员权限加载 WinDivert.sys（若尚未加载）
  3. 打开 WinDivert 句柄，应用默认过滤表达式
```

---

## 14. 性能设计

### 14.1 性能预算

| 指标 | 目标值 | 测量方式 |
|------|--------|---------|
| CPU (空闲) | < 1% | 无活跃网络进程时 |
| CPU (高负载) | < 3% | 100+ 活跃网络进程 |
| 内存 (常驻) | < 40MB | 稳定运行 30 分钟后 |
| 内存 (峰值) | < 80MB | 大量进程活跃时 |
| 事件处理延迟 | < 50ms | 从内核事件到前端显示 |
| 启动时间 | < 2s | 冷启动到界面显示 |

### 14.2 关键优化策略

```
┌──────────────────────────────────────────────────────────────┐
│                     性能优化策略矩阵                          │
├──────────────┬───────────────────────────────────────────────┤
│ 后端 ETW     │ • Ring Buffer 无锁队列                        │
│  (windows-sys)│ • 批量事件处理 (100ms 批次)                   │
│              │ • 对象复用 (避免频繁堆分配)                    │
│              │ • 高频事件采样降级                              │
├──────────────┼───────────────────────────────────────────────┤
│ 后端 WinDivert│ • 端口→PID 查表 O(1)                          │
│              │ • 令牌桶增量计算，避免全量遍历                   │
│              │ • 下载限速有界缓冲，防止内存膨胀                 │
├──────────────┼───────────────────────────────────────────────┤
│ 前后端通信   │ • 事件推送而非轮询                              │
│  (Tauri)     │ • 差量更新 (仅推送变化的进程数据)                │
│              │ • 数据序列化优化 (serde)                        │
├──────────────┼───────────────────────────────────────────────┤
│ 前端渲染     │ • 虚拟滚动列表 (100+ 进程场景)                 │
│              │ • 图表数据降采样                               │
│              │ • requestAnimationFrame 节流                  │
│              │ • Web Worker 处理数据转换                      │
└──────────────┴───────────────────────────────────────────────┘
```

---

## 15. 部署与发布

### 15.1 构建流程

```bash
# 前端依赖安装与构建
pnpm install
pnpm build              # 输出 frontend/dist

# Rust / Tauri 构建（需 Windows + 管理员权限用于打包签名校验）
cargo tauri build       # 或: pnpm tauri build

# 开发模式 (热重载)
cargo tauri dev         # 或: pnpm tauri dev
```

### 15.2 发布产物

| 产物 | 说明 | 大小预估 |
|------|------|---------|
| `NetTamer.exe` | 便携版单文件（含 WinDivert 驱动分发） | ~10MB |
| `NetTamer-Setup.exe` | NSIS / WiX 安装包 | ~12MB |

### 15.3 运行时依赖

| 依赖 | 说明 | 内置情况 |
|------|------|---------|
| WebView2 | 界面渲染引擎 | Windows 10 1809+ / Windows 11 内置 |
| WinDivert 驱动 | 数据包拦截/重发 | **随包分发**（WinDivert.dll / .sys） |
| .NET / Node.js | 无 | **不需要** |
| Rust 运行时 | 静态链接 | 无需额外运行时 |

---

## 16. 开发路线图

### Phase 1: MVP — 核心监控 (2 周)

```
[ ] 项目脚手架搭建 (Tauri 2.0 + Vue 3 + shadcn-vue)
[ ] ETW 事件跟踪模块开发 (windows-sys)
[ ] 速率聚合计算引擎 (EWMA)
[ ] 进程列表基础界面
[ ] 实时速率显示 (上传/下载)
```

### Phase 2: 预警系统 (1 周)

```
[ ] 预警规则 CRUD
[ ] 预警引擎判定逻辑
[ ] Windows Toast 系统通知 (Tauri notification 插件)
[ ] 预警历史记录
[ ] 预警配置界面
```

### Phase 3: WinDivert 限速 (1 周)

```
[ ] WinDivert 引擎集成 (拦截/重发/过滤)
[ ] 端口→PID 映射与令牌桶限速
[ ] 限速策略管理 (创建/修改/删除)
[ ] 双向 (上传+下载) 限速管理界面
```

### Phase 4: 打磨与增强 (1 周)

```
[ ] 仪表盘总览页面
[ ] 实时速率折线图
[ ] 系统托盘支持 (Tauri tray 插件)
[ ] 暗色/亮色主题切换
[ ] 开机自启配置 (Tauri autostart 插件)
[ ] 性能优化与压力测试
[ ] 打包发布 (便携版 + 安装包)
```

### Phase 5: 未来规划

```
[ ] 数据包修改/重写规则 DSL（利用 WinDivert 修改+重发能力）
[ ] 网络连接详情 (IP / 域名 / 端口)
[ ] 历史流量统计与图表
[ ] 进程分组与标签
[ ] 规则模板市场
[ ] 多语言支持 (i18n)
```

---

## 附录

### A. 参考资料

| 资源 | 链接 |
|------|------|
| Tauri 2.0 官方文档 | https://v2.tauri.app/ |
| shadcn-vue 文档 | https://www.shadcn-vue.com |
| ETW 概述 (Microsoft) | https://learn.microsoft.com/en-us/windows/win32/etw |
| windows-rs (windows / windows-sys) | https://github.com/microsoft/windows-rs |
| WinDivert 官方 | https://www.reqrypt.org/windivert.html |
| WinDivert Rust crate | https://crates.io/crates/windivert (用户态 FFI 封装) |
| WFP 概述 (Microsoft) | https://learn.microsoft.com/en-us/windows/win32/fwp |

### B. 术语表

| 术语 | 全称 | 说明 |
|------|------|------|
| ETW | Event Tracing for Windows | Windows 内核级事件跟踪设施（用于监控统计） |
| WFP | Windows Filtering Platform | Windows 过滤平台，WinDivert 底层依托 |
| WinDivert | Windows Divert | 用户态数据包拦截/修改/重发/过滤库 |
| QoS | Quality of Service | 网络服务质量；本方案改由 WinDivert 实现 |
| EWMA | Exponentially Weighted Moving Average | 指数加权移动平均 |
| PID | Process Identifier | 进程标识符 |
| UAC | User Account Control | 用户账户控制 |
