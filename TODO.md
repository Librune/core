# BookCore 优化和功能 TODO 列表

**项目定位**: 跨平台 JavaScript 书源执行引擎库
**使用场景**: Rust TUI、Tauri、Flutter (FFI)、WebAssembly
**最后更新**: 2025-01-04

---

## 🎯 高优先级 (P0)

### 1. FFI 接口优化
**问题**: 当前库主要为 Rust 内部使用设计,缺少友好的 FFI 边界
**影响**: Flutter、Tauri 等跨语言调用困难

**TODO**:
- [ ] 设计 C-ABI 兼容的公共接口
  - [ ] 创建 `ffi` 模块
  - [ ] 定义 `#[repr(C)]` 结构体用于跨语言传递
  - [ ] 使用 `#[no_mangle]` 导出关键函数
  - [ ] 错误码系统 (避免 Result<T, E>)
- [ ] 添加 cbindgen 生成 C 头文件
- [ ] 为 Flutter 创建 flutter_rust_bridge 兼容层
- [ ] 文档: FFI 使用示例和最佳实践

**参考实现**:
```rust
// ffi/mod.rs
#[repr(C)]
pub struct BookCoreHandle {
    ptr: *mut BookCore,
}

#[no_mangle]
pub extern "C" fn bookcore_new() -> *mut BookCoreHandle {
    // ...
}

#[no_mangle]
pub extern "C" fn bookcore_eval_script(
    handle: *mut BookCoreHandle,
    script: *const c_char,
    result_out: *mut *mut c_char,
) -> i32 {
    // 返回错误码,通过 result_out 传递结果
}
```

### 2. 内存管理和生命周期
**问题**: 当前 Context 生命周期管理复杂,不适合长期运行场景
**影响**: TUI 应用、桌面应用内存泄漏风险

**TODO**:
- [ ] 实现 Context 池化/复用机制
- [ ] 添加脚本执行超时控制 (防止死循环)
- [ ] 实现自动垃圾回收触发
- [ ] 监控和限制内存使用
- [ ] 添加 `reset()` 方法清理 Context 状态

**示例**:
```rust
pub struct ContextPool {
    pool: Vec<Context>,
    max_size: usize,
}

impl BookCore {
    pub fn eval_with_timeout(&mut self, script: &str, timeout_ms: u64) -> BookResult<String> {
        // 使用 tokio::time::timeout 或线程中断
    }
}
```

### 3. WebAssembly 支持
**问题**: 部分功能依赖 std、threads,不兼容 wasm32-unknown-unknown
**影响**: 无法在浏览器中运行

**TODO**:
- [ ] 条件编译: `#[cfg(not(target_arch = "wasm32"))]`
- [ ] 替换 `std::thread` 为 wasm 兼容方案
- [ ] HTTP 客户端使用 `reqwest` 的 wasm 特性
- [ ] 定时器使用 `wasm-bindgen-futures`
- [ ] 文件系统操作抽象 (wasm 使用 virtual fs)
- [ ] 添加 wasm 打包示例和文档

**关键改动**:
```rust
// timers.rs
#[cfg(not(target_arch = "wasm32"))]
pub fn register_set_timeout(ctx: &mut Context) {
    // 使用 std::thread
}

#[cfg(target_arch = "wasm32")]
pub fn register_set_timeout(ctx: &mut Context) {
    // 使用 wasm_bindgen_futures::spawn_local
}
```

---

## 🚀 中优先级 (P1)

### 4. 异步执行支持
**问题**: 当前所有操作都是同步的,阻塞调用方
**影响**: TUI/桌面应用 UI 卡顿

**TODO**:
- [ ] 创建异步 API 接口
  ```rust
  pub async fn eval_async(&mut self, script: &str) -> BookResult<String>
  ```
- [ ] 使用 tokio runtime 运行脚本
- [ ] 支持 JavaScript Promise/async-await
- [ ] 添加取消令牌 (CancellationToken)
- [ ] 并发控制 (限制同时执行的脚本数)

### 5. 错误处理增强
**问题**: 错误信息不够详细,跨语言传递困难
**影响**: 调试困难,用户体验差

**TODO**:
- [ ] JavaScript 错误堆栈追踪
- [ ] 源码位置映射 (行号、列号)
- [ ] 结构化错误信息 (JSON 格式)
- [ ] 错误分类和恢复建议
- [ ] FFI 友好的错误码系统

**示例**:
```rust
#[derive(Serialize)]
pub struct ScriptError {
    pub error_type: String,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub stack_trace: Vec<String>,
    pub suggestion: Option<String>,
}
```

### 6. 配置系统
**问题**: 硬编码配置,不灵活
**影响**: 不同场景需要不同配置

**TODO**:
- [ ] 创建 `BookCoreConfig` 结构体
- [ ] 可配置项:
  - [ ] 脚本执行超时
  - [ ] 最大内存限制
  - [ ] HTTP 连接池大小
  - [ ] 缓存策略 (大小、TTL)
  - [ ] 启用/禁用特定 API (安全沙箱)
- [ ] 支持运行时动态调整

**示例**:
```rust
pub struct BookCoreConfig {
    pub timeout_ms: u64,
    pub max_memory_mb: usize,
    pub enable_network: bool,
    pub enable_timers: bool,
    pub cache_size_mb: usize,
}

impl BookCore {
    pub fn with_config(config: BookCoreConfig) -> Self { ... }
    pub fn update_config(&mut self, config: BookCoreConfig) { ... }
}
```

### 7. 性能监控和分析
**问题**: 无法了解脚本执行性能
**影响**: 难以优化慢脚本

**TODO**:
- [ ] 脚本执行时间统计
- [ ] HTTP 请求耗时统计
- [ ] 内存使用追踪
- [ ] 缓存命中率统计
- [ ] 导出 Prometheus 指标
- [ ] 性能分析报告 API

**示例**:
```rust
pub struct PerformanceMetrics {
    pub script_duration_ms: u64,
    pub http_requests: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub memory_used_bytes: usize,
}

impl BookCore {
    pub fn get_metrics(&self) -> PerformanceMetrics { ... }
}
```

### 8. 日志系统集成
**问题**: 内部日志难以集成到应用日志系统
**影响**: 调试困难

**TODO**:
- [ ] 使用 `tracing` 替代 `println!`
- [ ] 支持自定义日志处理器
- [ ] 日志级别控制
- [ ] 结构化日志输出
- [ ] FFI 日志回调

**示例**:
```rust
use tracing::{info, warn, error};

pub trait LogHandler {
    fn log(&self, level: LogLevel, message: &str);
}

impl BookCore {
    pub fn set_log_handler(&mut self, handler: Box<dyn LogHandler>) { ... }
}
```

---

## 💡 低优先级 (P2)

### 9. 沙箱安全增强
**问题**: JavaScript 可以访问所有 API,存在安全风险
**影响**: 恶意脚本可能滥用资源

**TODO**:
- [ ] 资源配额限制
  - [ ] 最大 HTTP 请求数
  - [ ] 文件大小限制
  - [ ] 执行时间限制
- [ ] API 权限控制 (白名单/黑名单)
- [ ] 网络访问限制 (允许的域名列表)
- [ ] 脚本签名验证

### 10. 调试工具
**问题**: 难以调试 JavaScript 脚本
**影响**: 开发效率低

**TODO**:
- [ ] REPL 模式 (交互式执行)
- [ ] 断点调试支持
- [ ] 变量查看器
- [ ] 性能分析器
- [ ] 网络请求查看器
- [ ] Chrome DevTools Protocol 支持 (远程调试)

### 11. 书源脚本标准化
**问题**: 没有统一的书源脚本规范
**影响**: 兼容性差,维护困难

**TODO**:
- [ ] 定义书源 Schema (JSON Schema)
- [ ] 脚本验证器
- [ ] 脚本模板生成器
- [ ] 版本控制和迁移工具
- [ ] 测试框架 (单元测试书源脚本)

### 12. 插件系统
**问题**: 所有功能都内置,不够灵活
**影响**: 代码膨胀,难以扩展

**TODO**:
- [ ] 动态加载 JavaScript 模块
- [ ] 原生插件接口 (Rust)
- [ ] 插件依赖管理
- [ ] 插件市场/仓库

### 13. 多语言绑定
**问题**: 只有 Rust API
**影响**: 跨语言使用困难

**TODO**:
- [ ] Python 绑定 (PyO3)
- [ ] Node.js 绑定 (napi-rs)
- [ ] Go 绑定 (cgo)
- [ ] Java 绑定 (JNI)
- [ ] Swift 绑定 (iOS)

### 14. 增量编译和缓存
**问题**: 每次都重新解析脚本
**影响**: 性能浪费

**TODO**:
- [ ] 脚本字节码缓存
- [ ] 增量编译支持
- [ ] 持久化缓存到磁盘

### 15. 更多浏览器 API
**问题**: 部分书源需要更多浏览器兼容 API

**TODO**:
- [ ] `fetch` API (现代 HTTP 接口)
- [ ] `localStorage` / `sessionStorage`
- [ ] `WebSocket` 支持
- [ ] `Blob` / `ArrayBuffer`
- [ ] `TextEncoder` / `TextDecoder`
- [ ] `crypto.subtle` (Web Crypto API)
- [ ] `FormData`
- [ ] `URLSearchParams`

---

## 🔧 技术债务

### 16. 代码质量
- [ ] 增加单元测试覆盖率 (目标 80%)
- [ ] 添加集成测试
- [ ] 基准测试 (criterion.rs)
- [ ] CI/CD 自动化 (GitHub Actions)
- [ ] 代码审计工具 (clippy pedantic)
- [ ] 安全审计 (cargo audit)

### 17. 文档完善
- [ ] 完整的 API 文档 (rustdoc)
- [ ] 跨平台集成示例
- [ ] 性能调优指南
- [ ] 故障排除手册
- [ ] 迁移指南 (版本升级)

### 18. 依赖管理
- [ ] 定期更新依赖
- [ ] 移除未使用的依赖
- [ ] 可选特性拆分 (features)
  ```toml
  [features]
  default = ["http", "crypto"]
  http = ["reqwest"]
  crypto = ["aes", "des", "hmac"]
  wasm = ["wasm-bindgen"]
  ```

---

## 📋 优先级矩阵

| 任务 | 影响 | 难度 | 优先级 | 预计工时 |
|------|------|------|--------|---------|
| FFI 接口优化 | 高 | 中 | P0 | 2-3 周 |
| 内存管理 | 高 | 中 | P0 | 1-2 周 |
| WASM 支持 | 高 | 高 | P0 | 3-4 周 |
| 异步执行 | 中 | 高 | P1 | 2-3 周 |
| 错误处理增强 | 中 | 低 | P1 | 1 周 |
| 配置系统 | 中 | 低 | P1 | 3-5 天 |
| 性能监控 | 中 | 中 | P1 | 1-2 周 |
| 日志系统 | 低 | 低 | P1 | 3-5 天 |
| 沙箱安全 | 中 | 中 | P2 | 1-2 周 |
| 调试工具 | 低 | 高 | P2 | 2-3 周 |

---

## 🎯 短期目标 (1-2 个月)

1. **P0 任务**: FFI 接口 + 内存管理
2. **P1 任务**: 配置系统 + 日志系统 + 错误处理
3. **文档**: 跨平台集成示例

## 🚢 中期目标 (3-6 个月)

1. **P0 任务**: WASM 完整支持
2. **P1 任务**: 异步执行 + 性能监控
3. **测试**: 覆盖率达到 80%

## 🌟 长期目标 (6-12 个月)

1. **P2 任务**: 调试工具 + 插件系统
2. **多语言绑定**: Python, Node.js
3. **1.0 正式版发布**

---

**维护者**: [@zsakvo](https://github.com/zsakvo)
**贡献**: 欢迎提交 Issue 和 PR!
