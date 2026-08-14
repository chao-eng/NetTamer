# 🐾 NetTamer — 项目架构设计文档

> **版本**: v1.0.0-draft  
> **日期**: 2024-08-14  
> **状态**: 设计阶段

---

## 目录

1. [项目概述](#1-项目概述)
2. [核心需求](#2-核心需求)
3. [技术选型](#3-技术选型)
4. [系统架构总览](#4-系统架构总览)
5. [后端架构设计 (Go)](#5-后端架构设计-go)
6. [前端架构设计 (Vue 3)](#6-前端架构设计-vue-3)
7. [ETW 网络事件跟踪模块](#7-etw-网络事件跟踪模块)
8. [QoS 限速模块](#8-qos-限速模块)
9. [预警系统设计](#9-预警系统设计)
10. [数据模型设计](#10-数据模型设计)
11. [Wails 绑定层设计](#11-wails-绑定层设计)
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
| **核心价值** | 无需安装第三方驱动，即可实现进程级网络监控与限速 |
| **竞品对比** | 区别于 NetLimiter 等需要安装内核驱动的方案，NetTamer 完全基于 Windows 原生 API |

### 1.2 核心特性一览

```
┌─────────────────────────────────────────────────────────────┐
│                      NetTamer 核心特性                       │
├─────────────┬──────────────┬───────────────┬────────────────┤
│  📊 实时监控  │  ⚡ 上行预警   │  🚦 QoS 限速   │  🎨 精美界面   │
│  进程级速率   │  阈值可配置   │  系统原生策略   │  shadcn-vue   │
│  上传/下载    │  自动提醒     │  无需驱动      │  暗色主题     │
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

#### F2: 上传速率预警
- 用户可对指定进程或全局设置 **上传速率阈值**
- 当上传速率超出阈值时，触发 **系统级通知弹窗**
- 支持预警规则的增删改查与持久化
- 支持预警历史记录查询

#### F3: 进程级限速
- 基于 **Windows 原生 QoS 策略** 对指定进程设置上传带宽上限
- 支持限速策略的创建、修改、删除
- 限速操作即时生效，无需重启进程
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
| **权限** | 需要管理员权限（ETW + QoS） |
| **安装** | 单文件便携版 + 安装包两种分发方式 |

---

## 3. 技术选型

### 3.1 技术栈总览

```
┌──────────────────────────────────────────────────────────────────┐
│                           技术栈                                  │
├──────────────┬───────────────────────────────────────────────────┤
│ 桌面框架      │ Wails v3 (Beta)                                  │
│ 后端语言      │ Go 1.22+                                        │
│ 前端框架      │ Vue 3 (Composition API) + TypeScript             │
│ UI 组件库     │ shadcn-vue + Tailwind CSS                        │
│ 状态管理      │ Pinia                                            │
│ 图表库        │ Apache ECharts / uPlot                           │
│ 网络监控      │ Windows ETW (Microsoft-Windows-Kernel-Network)   │
│ 流量整形      │ Windows QoS Policy (New-NetQosPolicy)            │
│ 数据持久化    │ SQLite (via modernc.org/sqlite, 纯 Go)            │
│ 构建工具      │ Vite 5                                           │
│ 包管理        │ pnpm                                             │
└──────────────┴───────────────────────────────────────────────────┘
```

### 3.2 关键技术选型说明

#### 3.2.1 Wails v3 — 桌面应用框架

**选型理由：**
- Go 原生绑定，无需 Node.js 运行时
- 使用系统 WebView2（Windows 默认内置），安装包体积极小（~8MB）
- **Service 化架构**：后端逻辑封装为模块化的 Service，支持生命周期钩子（`ServiceStartup` / `ServiceShutdown`）
- **静态源码分析绑定**：通过 `wails3 generate bindings` 基于源码分析生成 TypeScript 绑定，保留 JSDoc 注释与参数名
- **原生多窗口支持**：一等公民级别的多窗口创建、管理与销毁
- **原生系统托盘支持**：内置统一的 `app.SystemTray` API，支持托盘图标、菜单及点击事件
- **Taskfile 构建系统**：透明可检查的 `Taskfile.yml`，替代 v2 的不透明构建流程

**Wails v3 vs v2 关键变化：**

| 特性 | Wails v2 | Wails v3 |
|------|---------|----------|
| API 风格 | 声明式 (`wails.Run`) | 过程式（显式 app/window 生命周期） |
| 窗口管理 | 单窗口 | 多窗口（动态管理） |
| 绑定机制 | 反射 (Reflection) | 静态源码分析（更丰富的类型绑定） |
| 构建系统 | 内置不透明 | Taskfile（可检查/可扩展） |
| 系统托盘 | 社区方案 / 受限 | 原生一等支持 |
| 服务注册 | `Bind: []interface{}` | `Services: []application.Service` |

#### 3.2.2 ETW — 网络事件跟踪

**选型理由：**
- Windows 原生内核级事件跟踪设施，零额外驱动
- `Microsoft-Windows-Kernel-Network` Provider 直接提供 TCP Send/Recv 事件，携带 PID 和字节数
- 高性能、低开销，内核层面数据采集

**Go 库选型对比：**

| 库 | CGO | 特点 | 选择 |
|----|-----|------|------|
| `bi-zone/etw` | ✅ 需要 | 成熟稳定，社区活跃 | 备选 |
| `tekert/goetw` | ❌ 不需要 | 纯 Go，无外部依赖，高性能 | **首选** ✅ |
| `secDre4mer/etw` | ✅ 需要 | bi-zone fork，增强功能 | 备选 |

> **决策**: 优先选用 `tekert/goetw`，避免 CGO 依赖以简化交叉编译与 CI 流程。若遇到功能不足，回退至 `bi-zone/etw`。

#### 3.2.3 QoS Policy — 流量整形

**选型理由：**
- Windows 内置的 `New-NetQosPolicy` cmdlet 支持按可执行文件名限制出站带宽
- 无需安装任何驱动，通过 `os/exec` 调用 PowerShell 即可实现
- 系统级策略，对进程透明，无侵入性

**限制与应对：**

| 限制 | 应对策略 |
|------|---------|
| QoS Policy 仅支持**出站（上传）**限速 | MVP 阶段仅支持上传限速；后续可通过 WFP 扩展下载限速 |
| 需要管理员权限 | 应用启动时请求 UAC 提升 |
| 按可执行文件名匹配（非 PID） | 策略粒度为进程名级别，同名进程共享策略 |

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
│                              │ Wails Bindings (TypeScript)             │
├──────────────────────────────┼──────────────────────────────────────────┤
│                              │ Wails Runtime Bridge                    │
├──────────────────────────────┼──────────────────────────────────────────┤
│                       应用服务层 (Go)                                   │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │                     Wails App Service                        │      │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────┐  │      │
│  │  │ MonitorSvc │ │ AlertSvc   │ │ ThrottleSvc│ │ ConfigSvc│  │      │
│  │  └─────┬──────┘ └─────┬──────┘ └─────┬──────┘ └────┬─────┘  │      │
│  └────────┼──────────────┼──────────────┼─────────────┼────────┘      │
│           │              │              │             │                │
├───────────┼──────────────┼──────────────┼─────────────┼────────────────┤
│           ▼              ▼              ▼             ▼                │
│                       核心引擎层 (Go)                                   │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐  │
│  │  ETW Engine  │ │ Alert Engine │ │ QoS Engine   │ │ Store Engine │  │
│  │ (内核事件消费) │ │ (规则引擎)   │ │ (策略管理)    │ │ (SQLite)    │  │
│  └──────┬───────┘ └──────────────┘ └──────┬───────┘ └──────────────┘  │
│         │                                  │                           │
├─────────┼──────────────────────────────────┼───────────────────────────┤
│         ▼                                  ▼                           │
│                     操作系统层 (Windows)                                 │
│  ┌────────────────────────────┐  ┌────────────────────────────┐       │
│  │ ETW Kernel Session         │  │ PowerShell QoS Cmdlets     │       │
│  │ Microsoft-Windows-          │  │ New-NetQosPolicy           │       │
│  │   Kernel-Network           │  │ Set-NetQosPolicy           │       │
│  │ GUID: {7dd42a49-5329-      │  │ Remove-NetQosPolicy        │       │
│  │   4832-8dfd-43d979153a88}  │  │ Get-NetQosPolicy           │       │
│  └────────────────────────────┘  └────────────────────────────┘       │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4.2 数据流架构

```
                    ┌───────────────────────────────────┐
                    │          Windows Kernel            │
                    │   TCP Send / TCP Recv Events       │
                    └──────────────┬────────────────────┘
                                   │ ETW Real-time Session
                                   ▼
                    ┌───────────────────────────────────┐
                    │       ETW Consumer (Go)            │
                    │  解析事件 → 提取 PID + 字节数        │
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
              │ Windows Toast  │   │ Wails EventEmit   │
              │ 系统通知弹窗    │   │ 推送到前端渲染      │
              └────────────────┘   └───────────────────┘
```

---

## 5. 后端架构设计 (Go)

### 5.1 模块划分

后端采用 **分层 + 模块化** 架构，各模块职责明确、低耦合：

```
internal/
├── etw/            # ETW 事件跟踪模块
├── monitor/        # 速率监控与聚合模块
├── alert/          # 预警引擎模块
├── throttle/       # QoS 限速模块
├── process/        # 进程信息查询模块
├── store/          # 数据持久化模块
├── config/         # 配置管理模块
├── notify/         # 系统通知模块
└── tray/           # 系统托盘模块
```

### 5.2 核心模块详细设计

#### 5.2.1 ETW 模块 (`internal/etw`)

```go
// etw/session.go — ETW 会话管理

package etw

// KernelNetworkProviderGUID 是 Microsoft-Windows-Kernel-Network 的 Provider GUID
const KernelNetworkProviderGUID = "{7dd42a49-5329-4832-8dfd-43d979153a88}"

// NetworkEvent 表示一次 TCP 网络事件
type NetworkEvent struct {
    Timestamp  time.Time
    PID        uint32
    Direction  Direction  // Send / Recv
    Size       uint32     // 字节数
    LocalIP    net.IP
    LocalPort  uint16
    RemoteIP   net.IP
    RemotePort uint16
}

// Direction 网络方向
type Direction int
const (
    DirectionSend Direction = iota  // 上传
    DirectionRecv                    // 下载
)

// Session 管理 ETW 实时事件会话
type Session struct {
    sessionName string
    eventCh     chan NetworkEvent
    stopCh      chan struct{}
}

// NewSession 创建并启动一个 ETW 实时事件跟踪会话
func NewSession(bufferSize int) (*Session, error) { ... }

// Events 返回事件通道，供消费者读取
func (s *Session) Events() <-chan NetworkEvent { ... }

// Stop 停止事件跟踪会话并释放资源
func (s *Session) Stop() error { ... }
```

#### 5.2.2 监控聚合模块 (`internal/monitor`)

```go
// monitor/aggregator.go — 按进程聚合网络速率

package monitor

// ProcessStats 单个进程的网络统计快照
type ProcessStats struct {
    PID           uint32  `json:"pid"`
    Name          string  `json:"name"`
    Path          string  `json:"path"`
    UploadRate    float64 `json:"uploadRate"`    // bytes/sec
    DownloadRate  float64 `json:"downloadRate"`  // bytes/sec
    TotalUpload   uint64  `json:"totalUpload"`   // 累计上传字节
    TotalDownload uint64  `json:"totalDownload"` // 累计下载字节
}

// Aggregator 速率聚合器
type Aggregator struct {
    mu        sync.RWMutex
    processes map[uint32]*processAccumulator  // PID → 累加器
    window    time.Duration                    // 统计窗口
    ticker    *time.Ticker
}

// NewAggregator 创建聚合器，window 为速率计算窗口（如 1s）
func NewAggregator(window time.Duration) *Aggregator { ... }

// Ingest 接收 ETW 原始事件并累加
func (a *Aggregator) Ingest(event etw.NetworkEvent) { ... }

// Snapshot 返回当前所有进程的速率快照（线程安全）
func (a *Aggregator) Snapshot() []ProcessStats { ... }

// TopN 返回按指定维度排序的前 N 个进程
func (a *Aggregator) TopN(n int, sortBy SortField, order SortOrder) []ProcessStats { ... }
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

#### 5.2.3 预警引擎模块 (`internal/alert`)

```go
// alert/engine.go — 预警规则引擎

package alert

// Rule 预警规则
type Rule struct {
    ID            string    `json:"id"`
    Name          string    `json:"name"`
    ProcessName   string    `json:"processName"`   // 进程名匹配（支持通配符）
    Threshold     float64   `json:"threshold"`      // 阈值 (bytes/sec)
    Direction     Direction `json:"direction"`      // Upload / Download / Both
    CooldownSec   int       `json:"cooldownSec"`    // 冷却时间，防止重复告警
    Enabled       bool      `json:"enabled"`
    CreatedAt     time.Time `json:"createdAt"`
}

// AlertEvent 一次预警事件
type AlertEvent struct {
    ID          string    `json:"id"`
    RuleID      string    `json:"ruleId"`
    ProcessName string    `json:"processName"`
    PID         uint32    `json:"pid"`
    CurrentRate float64   `json:"currentRate"`   // 当前速率
    Threshold   float64   `json:"threshold"`     // 触发阈值
    TriggeredAt time.Time `json:"triggeredAt"`
}

// Engine 预警引擎
type Engine struct {
    rules      map[string]*Rule
    cooldowns  map[string]time.Time  // ruleID+processName → 上次触发时间
    alertCh    chan AlertEvent
    store      store.AlertStore
}

// NewEngine 创建预警引擎
func NewEngine(store store.AlertStore) *Engine { ... }

// Evaluate 对一组进程快照执行规则匹配
func (e *Engine) Evaluate(stats []monitor.ProcessStats) { ... }

// Alerts 返回预警事件通道
func (e *Engine) Alerts() <-chan AlertEvent { ... }
```

#### 5.2.4 QoS 限速模块 (`internal/throttle`)

```go
// throttle/qos.go — 基于 Windows QoS Policy 的限速管理

package throttle

// Policy 限速策略
type Policy struct {
    ID           string    `json:"id"`
    Name         string    `json:"name"`          // QoS 策略名称
    ProcessName  string    `json:"processName"`   // 可执行文件名 (如 "chrome.exe")
    RateLimitBps uint64    `json:"rateLimitBps"`  // 限速值 (bits per second)
    Active       bool      `json:"active"`
    CreatedAt    time.Time `json:"createdAt"`
}

// Manager QoS 策略管理器
type Manager struct {
    mu       sync.Mutex
    policies map[string]*Policy
    store    store.ThrottleStore
}

// ApplyPolicy 创建或更新限速策略
// 内部调用: New-NetQosPolicy -Name <name> -AppPathName <exe> -ThrottleRateActionBitsPerSecond <rate>
func (m *Manager) ApplyPolicy(policy Policy) error { ... }

// RemovePolicy 移除限速策略
// 内部调用: Remove-NetQosPolicy -Name <name> -Confirm:$False
func (m *Manager) RemovePolicy(policyID string) error { ... }

// ListPolicies 列出所有由 NetTamer 创建的限速策略
func (m *Manager) ListPolicies() ([]Policy, error) { ... }

// SyncWithSystem 同步系统中实际存在的 QoS 策略与本地记录
func (m *Manager) SyncWithSystem() error { ... }
```

**PowerShell 调用封装：**

```go
// throttle/powershell.go — PowerShell 命令封装

package throttle

// execPowerShell 执行 PowerShell 命令并返回输出
func execPowerShell(script string) (string, error) {
    cmd := exec.Command("powershell", "-NoProfile", "-NonInteractive", "-Command", script)
    cmd.SysProcAttr = &syscall.SysProcAttr{HideWindow: true}
    output, err := cmd.CombinedOutput()
    return string(output), err
}

// createQosPolicy 创建 QoS 策略
func createQosPolicy(name, appPath string, rateBps uint64) error {
    script := fmt.Sprintf(
        `New-NetQosPolicy -Name "%s" -AppPathNameMatchCondition "%s" -ThrottleRateActionBitsPerSecond %d -PolicyStore ActiveStore`,
        name, appPath, rateBps,
    )
    _, err := execPowerShell(script)
    return err
}
```

#### 5.2.5 进程信息模块 (`internal/process`)

```go
// process/info.go — 进程元数据查询

package process

// Info 进程基础信息
type Info struct {
    PID      uint32 `json:"pid"`
    Name     string `json:"name"`
    Path     string `json:"path"`
    IconB64  string `json:"iconB64"`  // Base64 编码的进程图标
    User     string `json:"user"`
}

// Resolver 进程信息解析器（带缓存）
type Resolver struct {
    cache  *lru.Cache[uint32, *Info]
    mu     sync.RWMutex
}

// Resolve 根据 PID 获取进程信息，优先从缓存读取
func (r *Resolver) Resolve(pid uint32) (*Info, error) { ... }
```

#### 5.2.6 数据持久化模块 (`internal/store`)

```go
// store/store.go — SQLite 数据存储

package store

// DB 数据库管理器
type DB struct {
    db *sql.DB
}

// 接口定义
type AlertStore interface {
    SaveRule(rule alert.Rule) error
    DeleteRule(id string) error
    ListRules() ([]alert.Rule, error)
    SaveAlertEvent(event alert.AlertEvent) error
    ListAlertEvents(filter AlertEventFilter) ([]alert.AlertEvent, error)
}

type ThrottleStore interface {
    SavePolicy(policy throttle.Policy) error
    DeletePolicy(id string) error
    ListPolicies() ([]throttle.Policy, error)
}

type ConfigStore interface {
    Get(key string) (string, error)
    Set(key, value string) error
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
|------|------|------|
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
│   │   ├── button/
│   │   ├── card/
│   │   ├── dialog/
│   │   ├── table/
│   │   ├── toast/
│   │   └── ...
│   ├── layout/              # 布局组件
│   │   ├── AppSidebar.vue
│   │   ├── AppHeader.vue
│   │   └── AppLayout.vue
│   ├── charts/              # 图表组件
│   │   ├── SpeedChart.vue
│   │   └── SpeedGauge.vue
│   └── common/              # 通用业务组件
│       ├── ProcessIcon.vue
│       ├── SpeedBadge.vue
│       └── StatusIndicator.vue
├── composables/             # 组合式函数
│   ├── useProcessMonitor.ts # 封装 Wails 绑定: 进程监控
│   ├── useAlertRules.ts     # 封装 Wails 绑定: 预警规则
│   ├── useThrottle.ts       # 封装 Wails 绑定: 限速管理
│   ├── useConfig.ts         # 封装 Wails 绑定: 配置管理
│   └── useFormatters.ts     # 速率格式化 (bytes → KB/s, MB/s)
├── lib/                     # 工具函数
│   └── utils.ts             # shadcn-vue cn() 工具
├── router/                  # 路由配置
│   └── index.ts
├── stores/                  # Pinia 状态管理
│   ├── processStore.ts      # 进程数据状态
│   ├── alertStore.ts        # 预警状态
│   ├── throttleStore.ts     # 限速状态
│   └── settingsStore.ts     # 全局设置状态
├── views/                   # 页面视图
│   ├── DashboardView.vue
│   ├── ProcessListView.vue
│   ├── AlertConfigView.vue
│   ├── ThrottleManagerView.vue
│   └── SettingsView.vue
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
  // Wails EventsOn 监听后端推送的实时数据
  // 前端不轮询，后端定时推送
})
```

### 6.6 前后端通信机制

NetTamer 的前后端通信采用 **Wails v3 双向绑定** 机制：

- **方法调用**：前端通过 `wails3 generate bindings` 自动生成的 TypeScript 客户端直接调用 Go Service 方法，类型安全
- **事件推送**：后端通过 `application.EmitEvent()` 向前端推送实时数据，前端通过 `wails.Events.On()` 监听

```
┌─────────────┐                    ┌─────────────┐
│   Vue 前端   │                    │   Go 后端    │
├─────────────┤                    ├─────────────┤
│             │  ── 方法调用 ──→     │             │
│ composable  │  生成的 TS 绑定      │ Service     │
│ useXxx()    │  (类型安全)         │ Methods     │
│             │  ←── 返回值 ──      │             │
│             │                    │             │
│             │  ←── 事件推送 ──    │             │
│ Events.On() │  app.EmitEvent     │ EmitEvent   │
│             │  ("speed:update",  │  定时推送    │
│             │    processStats)   │  速率数据    │
└─────────────┘                    └─────────────┘
```

**关键事件定义：**

| 事件名 | 方向 | 数据 | 频率 |
|--------|------|------|------|
| `speed:update` | 后端 → 前端 | `[]ProcessStats` | 1s |
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
| **采样降级** | 当事件速率超过阈值（如 10万条/秒）时，自动降低采样率 |
| **零分配解析** | 事件解析过程中尽量复用 buffer，减少 GC 压力 |

---

## 8. QoS 限速模块

### 8.1 Windows QoS Policy 工作原理

```
用户空间                    内核空间

┌──────────┐            ┌──────────────────┐
│ NetTamer │            │   TCP/IP Stack   │
│          │            │                  │
│ PowerShell───────────▶│  QoS 策略引擎     │
│ New-Net   │           │  ┌────────────┐  │
│ QosPolicy │           │  │ 令牌桶算法   │  │
│          │            │  │ Token      │  │
│          │            │  │  Bucket    │  │
│          │            │  └────┬───────┘  │
│          │            │       │          │
│          │            │  ┌────▼───────┐  │
│          │            │  │ 出站队列    │  │
│          │            │  │ 按策略限速  │  │
│          │            │  └────┬───────┘  │
│          │            │       │          │
└──────────┘            └───────┼──────────┘
                                │
                        ┌───────▼──────────┐
                        │   Network NIC     │
                        └──────────────────┘
```

### 8.2 策略命名规范

为避免与系统中其他 QoS 策略冲突，NetTamer 创建的策略使用统一前缀：

```
策略名称格式: NT_{ProcessName}_{UniqueID}

示例:
  NT_chrome_a1b2c3
  NT_steam_d4e5f6
  NT_onedrive_g7h8i9
```

### 8.3 策略生命周期管理

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

应用退出时的清理策略:
  1. 列出所有 NT_ 前缀的 QoS 策略
  2. 逐一调用 Remove-NetQosPolicy 清理
  3. 同步更新本地数据库状态
```

### 8.4 限速范围说明

> **⚠️ 重要说明**
>
> **Windows 原生 QoS Policy 仅支持出站（上传）方向的流量限速。**
> 下载方向的限速无法通过原生 QoS 实现，需要 WFP 内核驱动或第三方方案。
> NetTamer v1.0 仅提供上传限速功能，下载限速纳入未来路线图。

| 方向 | 支持状态 | 实现方案 |
|------|---------|---------|
| 上传 (Egress) | ✅ v1.0 | Windows QoS Policy |
| 下载 (Ingress) | 🔮 未来 | WFP Callout Driver（需评估） |

---

## 9. 预警系统设计

### 9.1 预警规则模型

```
┌─────────────────────────────────────────┐
│              预警规则 (Rule)              │
├──────────────┬──────────────────────────┤
│ 匹配条件      │ 进程名 (支持通配符 *)     │
│              │ 如: "chrome.exe"         │
│              │ 如: "*.exe"             │
├──────────────┼──────────────────────────┤
│ 触发条件      │ 上传速率 > 阈值 (bytes/s) │
│              │ 持续时间 > N 秒           │
├──────────────┼──────────────────────────┤
│ 动作         │ 系统通知弹窗              │
│              │ 前端 Toast 提醒          │
│              │ (可选) 自动应用限速策略    │
├──────────────┼──────────────────────────┤
│ 冷却控制      │ 冷却时间 (默认 60s)       │
│              │ 防止短时间内重复告警       │
└──────────────┴──────────────────────────┘
```

### 9.2 预警判定流程

```go
// 伪代码: 预警判定逻辑

func (e *Engine) Evaluate(stats []ProcessStats) {
    for _, stat := range stats {
        for _, rule := range e.rules {
            if !rule.Enabled { continue }
            if !matchProcess(rule.ProcessName, stat.Name) { continue }

            rate := stat.UploadRate  // 当前上传速率
            if rate > rule.Threshold {
                key := rule.ID + ":" + stat.Name

                // 检查冷却期
                if lastFired, ok := e.cooldowns[key]; ok {
                    if time.Since(lastFired) < rule.CooldownDuration() {
                        continue  // 仍在冷却期内，跳过
                    }
                }

                // 触发预警
                event := AlertEvent{
                    RuleID:      rule.ID,
                    ProcessName: stat.Name,
                    PID:         stat.PID,
                    CurrentRate: rate,
                    Threshold:   rule.Threshold,
                    TriggeredAt: time.Now(),
                }
                e.alertCh <- event
                e.cooldowns[key] = time.Now()
                e.store.SaveAlertEvent(event)
            }
        }
    }
}
```

### 9.3 通知方式

| 方式 | 实现 | 说明 |
|------|------|------|
| Windows Toast 通知 | `go-toast` 或 Win32 API | 系统级弹窗，即使应用最小化也可见 |
| 应用内 Toast | shadcn-vue `<Toast>` | 应用窗口内的轻量提醒 |
| 声音提示 | 系统音效 | 可配置开关 |
| 自动限速 | QoS Manager | 可选：触发预警后自动创建限速策略 |

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
    name          TEXT NOT NULL UNIQUE,   -- QoS 策略名 (NT_ 前缀)
    process_name  TEXT NOT NULL,          -- 可执行文件名
    rate_limit_bps INTEGER NOT NULL,      -- 限速值 (bits/sec)
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

---

## 11. Wails v3 绑定层设计

### 11.1 Service 架构

Wails v3 采用 **Service 化架构**，每个 Service 是一个自包含的模块化组件，支持生命周期钩子。通过 `wails3 generate bindings` 静态源码分析自动生成类型安全的 TypeScript 绑定（保留 JSDoc 注释和参数名）。

```go
// main.go — Wails v3 应用主入口

package main

import (
    "embed"
    "log"

    "github.com/wailsapp/wails/v3/pkg/application"
)

//go:embed frontend/dist
var assets embed.FS

func main() {
    // 创建 Wails v3 应用 (过程式 API)
    app := application.New(application.Options{
        Name:        "NetTamer",
        Description: "进程级网络监控与流量整形工具",
        Services: []application.Service{
            // 注册各 Service，Wails 自动生成 TS 绑定
            application.NewService(service.NewMonitorService()),
            application.NewService(service.NewAlertService()),
            application.NewService(service.NewThrottleService()),
            application.NewService(service.NewConfigService()),
        },
        Assets: application.AssetOptions{
            Handler: application.AssetFileServerFS(assets),
        },
    })

    // 创建主窗口 (v3 支持多窗口)
    app.NewWebviewWindowWithOptions(application.WebviewWindowOptions{
        Title:     "NetTamer - 网络驯兽师",
        Width:     1200,
        Height:    800,
        MinWidth:  900,
        MinHeight: 600,
    })

    // 运行应用
    if err := app.Run(); err != nil {
        log.Fatal(err)
    }
}
```

### 11.2 Service 生命周期

每个 Service 可实现可选的生命周期钩子：

```go
// service/monitor_service.go — 监控服务示例

package service

type MonitorService struct {
    app       *application.App
    etw       *etw.Session
    aggregator *monitor.Aggregator
}

func NewMonitorService() *MonitorService {
    return &MonitorService{}
}

// ServiceStartup 在应用启动时自动调用
func (s *MonitorService) ServiceStartup(ctx context.Context, options application.ServiceOptions) error {
    s.app = options.App
    // 初始化 ETW 会话和聚合器
    return s.init()
}

// ServiceShutdown 在应用退出时自动调用
func (s *MonitorService) ServiceShutdown() error {
    // 清理 ETW 会话资源
    return s.etw.Stop()
}
```

### 11.2 Service 方法清单

#### MonitorService

| 方法 | 签名 | 说明 |
|------|------|------|
| `StartMonitoring` | `() error` | 启动 ETW 监控 |
| `StopMonitoring` | `() error` | 停止 ETW 监控 |
| `GetProcessList` | `() []ProcessStats` | 获取当前进程列表快照 |
| `GetProcessDetail` | `(pid uint32) *ProcessDetail` | 获取单个进程详细信息 |
| `SetRefreshInterval` | `(ms int) error` | 设置刷新频率 |

#### AlertService

| 方法 | 签名 | 说明 |
|------|------|------|
| `CreateRule` | `(rule Rule) error` | 创建预警规则 |
| `UpdateRule` | `(rule Rule) error` | 更新预警规则 |
| `DeleteRule` | `(id string) error` | 删除预警规则 |
| `ListRules` | `() []Rule` | 获取所有预警规则 |
| `GetAlertHistory` | `(filter Filter) []AlertEvent` | 查询预警历史 |
| `ClearAlertHistory` | `() error` | 清空预警历史 |

#### ThrottleService

| 方法 | 签名 | 说明 |
|------|------|------|
| `ApplyPolicy` | `(policy Policy) error` | 创建/更新限速策略 |
| `RemovePolicy` | `(id string) error` | 移除限速策略 |
| `ListPolicies` | `() []Policy` | 列出所有限速策略 |
| `SyncPolicies` | `() error` | 与系统 QoS 策略同步 |

#### ConfigService

| 方法 | 签名 | 说明 |
|------|------|------|
| `GetConfig` | `(key string) string` | 获取配置项 |
| `SetConfig` | `(key, value string) error` | 设置配置项 |
| `GetAllConfig` | `() map[string]string` | 获取所有配置 |
| `ResetConfig` | `() error` | 重置为默认配置 |

---

## 12. 项目目录结构

```
NetTamer/
├── build/                          # Wails 构建配置与平台资源
│   ├── appicon.png                 # 应用图标
│   ├── windows/                    # Windows 特定构建资源
│   │   ├── icon.ico
│   │   ├── info.json               # 版本信息
│   │   └── wails.exe.manifest      # UAC 管理员权限清单
│   ├── darwin/                     # macOS 构建资源 (可选)
│   └── linux/                      # Linux 构建资源 (可选)
│
├── doc/                            # 项目文档
│   ├── architecture.md             # 架构设计文档 (本文件)
│   ├── api-reference.md            # API 参考
│   └── dev-guide.md                # 开发指南
│
├── frontend/                       # Vue 3 前端项目
│   ├── public/
│   ├── dist/                       # 前端构建产物 (go:embed)
│   ├── bindings/                   # Wails v3 自动生成的 TS 绑定 (勿手动编辑)
│   │   └── github.com/
│   │       └── .../service/        # 按 Go 包路径组织的绑定
│   ├── src/
│   │   ├── assets/
│   │   ├── components/
│   │   │   ├── ui/                 # shadcn-vue 组件
│   │   │   ├── layout/
│   │   │   ├── charts/
│   │   │   └── common/
│   │   ├── composables/
│   │   ├── lib/
│   │   ├── router/
│   │   ├── stores/
│   │   ├── views/
│   │   ├── App.vue
│   │   ├── main.ts
│   │   └── style.css
│   ├── index.html
│   ├── components.json             # shadcn-vue 配置
│   ├── tailwind.config.js
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── package.json
│
├── internal/                       # Go 内部包 (不对外导出)
│   ├── etw/                        # ETW 事件跟踪
│   │   ├── session.go              # ETW 会话管理
│   │   ├── decoder.go              # 事件解码器
│   │   ├── provider.go             # Provider 常量定义
│   │   └── session_test.go
│   │
│   ├── monitor/                    # 速率监控与聚合
│   │   ├── aggregator.go           # 速率聚合器
│   │   ├── ewma.go                 # EWMA 算法
│   │   └── aggregator_test.go
│   │
│   ├── alert/                      # 预警引擎
│   │   ├── engine.go               # 规则引擎
│   │   ├── matcher.go              # 进程名匹配
│   │   └── engine_test.go
│   │
│   ├── throttle/                   # QoS 限速
│   │   ├── manager.go              # 策略管理器
│   │   ├── powershell.go           # PowerShell 命令封装
│   │   └── manager_test.go
│   │
│   ├── process/                    # 进程信息
│   │   ├── resolver.go             # 进程信息解析
│   │   ├── icon.go                 # 图标提取
│   │   └── resolver_test.go
│   │
│   ├── store/                      # 数据存储
│   │   ├── sqlite.go               # SQLite 实现
│   │   ├── migrations.go           # 数据库迁移
│   │   └── sqlite_test.go
│   │
│   ├── config/                     # 配置管理
│   │   └── config.go
│   │
│   ├── notify/                     # 系统通知
│   │   └── toast.go                # Windows Toast 通知
│   │
│   └── tray/                       # 系统托盘 (v3 原生支持)
│       └── tray.go
│
├── service/                        # Wails v3 Service 层 (自动生成 TS 绑定)
│   ├── monitor_service.go          # 监控服务 (ServiceStartup/Shutdown)
│   ├── alert_service.go            # 预警服务
│   ├── throttle_service.go         # 限速服务
│   └── config_service.go           # 配置服务
│
├── main.go                         # Wails v3 应用入口 (application.New)
├── Taskfile.yml                    # Wails v3 构建任务配置 (替代 Makefile)
├── go.mod
├── go.sum
├── README.md
└── .gitignore
```

---

## 13. 安全与权限

### 13.1 UAC 管理员权限

NetTamer 的核心功能（ETW 和 QoS）均需要管理员权限。通过 Windows 清单文件 (`wails.exe.manifest`) 声明 `requireAdministrator`：

```xml
<!-- build/windows/wails.exe.manifest -->
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
```

### 13.2 安全设计考量

| 风险 | 缓解措施 |
|------|---------|
| PowerShell 注入 | 所有参数使用白名单校验 + 参数化拼接，禁止拼接用户输入到命令字符串 |
| QoS 策略残留 | 应用退出时自动清理 NT_ 前缀策略；异常退出后下次启动自动检查清理 |
| 数据库安全 | SQLite 使用参数化查询，数据库文件存放在用户 AppData 目录 |
| 进程信息泄露 | 仅在本地 WebView 中渲染，不暴露任何网络端口 |

### 13.3 QoS Policy 注册表配置

在非域环境下，需要确保本地 QoS 策略被系统遵守：

```
注册表路径: HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Services\Tcpip\QoS
键名: Do not use NLA
值:  1 (DWORD)

NetTamer 首次启动时自动检查并设置此注册表项。
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
│              │ • 批量事件处理 (100ms 批次)                    │
│              │ • 对象池复用 (sync.Pool)                       │
│              │ • 高频事件采样降级                              │
├──────────────┼───────────────────────────────────────────────┤
│ 后端聚合     │ • EWMA 平滑算法                               │
│              │ • 增量计算，避免全量遍历                        │
│              │ • 进程退出时自动清理过期数据                     │
├──────────────┼───────────────────────────────────────────────┤
│ 前后端通信   │ • 事件推送而非轮询                              │
│              │ • 差量更新 (仅推送变化的进程数据)                │
│              │ • 数据序列化优化                               │
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
# 安装 Wails v3 CLI
go install github.com/wailsapp/wails/v3/cmd/wails3@latest

# 初始化项目 (Vue + TypeScript 模板)
wails3 init -n NetTamer -t vue-ts

# 生成 TypeScript 绑定 (静态源码分析)
wails3 generate bindings

# 开发模式 (热重载)
wails3 dev

# 生产构建
wails3 build

# 使用 Taskfile 运行自定义任务
wails3 task build:windows
wails3 task package:nsis
```

### 15.2 发布产物

| 产物 | 说明 | 大小预估 |
|------|------|---------|
| `NetTamer.exe` | 便携版单文件 | ~10MB |
| `NetTamer-Setup.exe` | NSIS 安装包 | ~12MB |

### 15.3 运行时依赖

| 依赖 | 说明 | 内置情况 |
|------|------|---------|
| WebView2 | 界面渲染引擎 | Windows 10 1809+ / Windows 11 内置 |
| PowerShell | QoS 策略管理 | Windows 内置 |
| .NET / Node.js | 无 | **不需要** |

---

## 16. 开发路线图

### Phase 1: MVP — 核心监控 (2 周)

```
[ ] 项目脚手架搭建 (Wails v3 + Vue 3 + shadcn-vue)
[ ] ETW 事件跟踪模块开发
[ ] 速率聚合计算引擎
[ ] 进程列表基础界面
[ ] 实时速率显示 (上传/下载)
```

### Phase 2: 预警系统 (1 周)

```
[ ] 预警规则 CRUD
[ ] 预警引擎判定逻辑
[ ] Windows Toast 系统通知
[ ] 预警历史记录
[ ] 预警配置界面
```

### Phase 3: QoS 限速 (1 周)

```
[ ] PowerShell QoS 策略封装
[ ] 限速策略管理 (创建/修改/删除)
[ ] 策略生命周期管理 (清理/同步)
[ ] 限速管理界面
```

### Phase 4: 打磨与增强 (1 周)

```
[ ] 仪表盘总览页面
[ ] 实时速率折线图
[ ] 系统托盘支持
[ ] 暗色/亮色主题切换
[ ] 开机自启配置
[ ] 性能优化与压力测试
[ ] 打包发布 (便携版 + 安装包)
```

### Phase 5: 未来规划

```
[ ] 下载限速 (WFP 方案调研)
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
| Wails v3 官方文档 | https://v3.wails.io/ |
| shadcn-vue 文档 | https://www.shadcn-vue.com |
| ETW 概述 (Microsoft) | https://learn.microsoft.com/en-us/windows/win32/etw |
| tekert/goetw | https://github.com/tekert/goetw |
| New-NetQosPolicy | https://learn.microsoft.com/en-us/powershell/module/netqos/new-netqospolicy |
| Windows QoS 策略 | https://learn.microsoft.com/en-us/windows-server/networking/technologies/qos/qos-policy-top |

### B. 术语表

| 术语 | 全称 | 说明 |
|------|------|------|
| ETW | Event Tracing for Windows | Windows 内核级事件跟踪设施 |
| QoS | Quality of Service | 网络服务质量，用于流量整形 |
| WFP | Windows Filtering Platform | Windows 过滤平台，用于网络包过滤 |
| EWMA | Exponentially Weighted Moving Average | 指数加权移动平均 |
| PID | Process Identifier | 进程标识符 |
| UAC | User Account Control | 用户账户控制 |
