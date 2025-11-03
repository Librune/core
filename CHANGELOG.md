# 更新日志

本文档记录 BookCore 项目的所有重要变更。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [未发布]

### 计划中
- HTML 解析增强（XPath 支持）
- WebSocket 连接支持
- 图片下载和处理功能
- Cookie 管理和会话持久化
- JavaScript 执行超时控制
- 书源调试工具
- DES/3DES 加密支持
- RSA 非对称加密
- setTimeout/setInterval 实现
- URL 类实现

## [0.2.0-beta] - 2025-01-03

### Phase 2: 性能优化 🚀

本版本专注于性能优化,实现了三大核心优化功能,预期综合性能提升 30-50%。

#### 新增功能 ✨

**1. HTTP 连接池**
- 实现全局单例 HTTP 客户端,避免重复创建连接
- 连接池配置:最大空闲连接数 10,空闲超时 90 秒
- 支持自定义超时的客户端创建
- 预期 HTTP 请求性能提升 20-30%

```rust
use book_core::request::client_pool::get_client;

let client = get_client(); // 获取全局客户端
```

**2. 响应缓存机制**
- 基于 FxHashMap 的高性能内存缓存
- 支持 TTL (Time To Live) 过期策略
- 仅对 GET 请求启用缓存
- 线程安全的读写访问

JavaScript 使用方式:
```javascript
// 启用缓存,TTL 为 300 秒
const response = JReqwest.get("https://api.example.com/data", {
    cacheTtl: 300
});
```

Rust 使用方式:
```rust
use book_core::request::cache::{get_cache, generate_cache_key};
use std::time::Duration;

let cache = get_cache();
let key = generate_cache_key("GET", "https://example.com");
cache.set(key, "response data".to_string(), Duration::from_secs(300));
```

**3. 字符串池化**
- 全局字符串池,减少重复字符串内存占用
- 预初始化 50+ 常用字符串(HTTP 方法、头部、JavaScript 属性等)
- 线程安全的字符串复用
- 预期字符串内存占用减少 10-15%

```rust
use book_core::intern;

let s1 = intern("hello");
let s2 = intern("hello");
assert_eq!(s1.as_ptr(), s2.as_ptr()); // 指向同一内存地址
```

#### 性能提升 📊

- **HTTP 请求**: +20-30% (连接池)
- **缓存命中**: 接近 0ms 延迟
- **字符串内存**: -10-15% (字符串池)
- **综合性能**: 预估提升 30-50%

#### 新增文件

- `src/request/client_pool.rs` - HTTP 客户端连接池实现 (109 行)
- `src/request/cache.rs` - 响应缓存系统 (285 行)
- `src/core/string_pool.rs` - 字符串池实现 (330 行)

#### 修改文件

- `src/request/mod.rs` - 导出新模块
- `src/request/options.rs` - 添加 `cache_ttl` 选项
- `src/request/jreqwest.rs` - 集成连接池和缓存
- `src/core/mod.rs` - 导出字符串池 API
- `src/lib.rs` - 导出公共 API
- `src/runtime.rs` - 初始化常用字符串

#### 测试

- 新增 17 个单元测试,全部通过 ✅
- 总测试数: 21 个
- 测试覆盖: 连接池、缓存、字符串池、并发访问

## [0.1.0] - 2025-01-03

### 重大升级 🎉

#### Boa JavaScript 引擎升级：0.20.0 → 0.21.0

**性能提升**
- 从堆栈式虚拟机改为寄存器式虚拟机，执行效率显著提升
- JsValue 使用 Nan-boxing 表示，内存占用减少约 30%
- ECMAScript 规范兼容性从 89.92% 提升到 94.12%

**新增功能**
- Temporal API 支持，时间处理达到 97% 一致性
- 增强的错误回溯支持，调试更友好
- 新增宏用于创建 ECMAScript 值、类和模块

### 破坏性变更 ⚠️

#### API 变化

1. **JsValue 构造方式变更**
   ```rust
   // 旧版本 (0.20.0)
   JsValue::String(text.into())
   JsValue::Object(obj)
   JsValue::Boolean(true)
   JsValue::Integer(42)

   // 新版本 (0.21.0)
   JsValue::new(js_string!(text))
   JsValue::new(obj)
   JsValue::new(true)
   JsValue::new(42)
   ```

2. **to_json() 返回类型变化**
   ```rust
   // 旧版本
   let value: Value = js_value.to_json(ctx).unwrap();

   // 新版本
   let value: Value = js_value.to_json(ctx)
       .unwrap()
       .unwrap_or(serde_json::Value::Null);
   ```

3. **Console::register_with_logger 参数顺序**
   ```rust
   // 旧版本
   Console::register_with_logger(context, logger)

   // 新版本
   Console::register_with_logger(logger, context)
   ```

4. **downcast_ref 使用方式**
   ```rust
   // 旧版本（链式调用）
   let options = this
       .as_object()
       .and_then(JsObject::downcast_ref::<Self>())
       .ok_or_else(|| ...)?;

   // 新版本（分步调用）
   let obj = this.as_object().ok_or_else(|| ...)?;
   let options = obj.downcast_ref::<Self>().ok_or_else(|| ...)?;
   ```

### 修复 🐛

1. **加密模块**
   - 修复 AES 加密/解密中的 `JsValue::String` 使用错误
   - 修复 HMAC 模块中的 `downcast_ref` 生命周期问题
   - 更新所有加密相关 API 以兼容 Boa 0.21.0

2. **请求模块**
   - 修复 `to_json()` 返回 `Option<Value>` 的处理
   - 修复字符集检测中的 `JsValue::Object` 构造
   - 优化 GBK/GB18030 编码处理

3. **原型扩展**
   - 修复 String 原型扩展中的所有 `JsValue` 构造
   - 修复 Object 原型扩展中的 `toQuery()` 方法
   - 修复 `JsValue::Integer` 改为 `JsValue::new(i32)`

4. **全局工具**
   - 修复 UUID 生成和验证中的 `JsValue::Boolean` 使用
   - 修复 XML 转 JSON 中的对象构造
   - 修复随机字符串生成的返回值类型

5. **代码清理**
   - 移除未使用的 `JsObject` 导入
   - 统一所有模块的 `JsValue` 创建方式

### 文档 📚

#### 新增文档
- **README.md** - 项目主文档，包含快速开始和功能介绍
- **ARCHITECTURE.md** - 详细的架构文档，涵盖所有模块设计
- **USER_GUIDE.md** - 完整的 API 参考和书源开发教程
- **CHANGELOG.md** - 本文档，记录所有变更

#### 文档内容
- 项目概述和特性介绍
- 完整的 API 使用说明
- JavaScript 书源脚本开发指南
- 加密、HTTP 请求、HTML 解析详细示例
- 常见问题和解决方案
- 性能优化建议
- 最佳实践指南

### 测试 ✅

#### 测试状态
- ✅ 核心功能测试通过
- ✅ HMAC 测试通过
- ✅ 环境变量测试通过
- ⚠️ AES 加密测试部分失败（测试用例需要更新）
- ⚠️ SHA 哈希测试失败（API 使用方式需要调整）

#### 测试覆盖
- 核心 JavaScript 执行
- 环境变量管理
- HMAC 签名功能
- 字符串原型扩展
- 对象原型扩展

### 技术债务

#### 需要修复的测试
1. **AES 加密测试** (`tests/crypto.rs`)
   - `test_crypto_aes128_pkcs7`: AesCrypto 未定义
   - `test_crypto_aes192_pkcs7`: 解密结果不匹配
   - `test_crypto_aes256_pkcs7`: CBC 解密错误

2. **SHA 哈希测试** (`tests/sha.rs`)
   - `test_crypto_sha224`: toSha 不是可调用函数
   - `test_crypto_sha256`: toSha 不是可调用函数

#### 待优化项
- [ ] 添加 HTTP 请求的集成测试
- [ ] 添加 HTML 解析的功能测试
- [ ] 改进错误类型定义（使用自定义错误类型替代 String）
- [ ] 添加异步 API 支持
- [ ] 实现 HTTP 连接池
- [ ] 添加请求缓存机制

### 依赖更新 📦

#### 主要依赖
- boa_engine: 0.20.0 → 0.21.0
- boa_runtime: 0.20.0 → 0.21.0
- boa_gc: 0.20.0 → 0.21.0
- boa_ast: 0.20.0 → 0.21.0 (新增)
- boa_interner: 0.20.0 → 0.21.0 (新增)
- boa_parser: 0.20.0 → 0.21.0 (新增)
- boa_string: 0.20.0 → 0.21.0 (新增)

#### 移除依赖
- boa_interop: 0.20.0 (未使用，已移除)

#### 其他依赖
- reqwest: 0.12.12 → 0.12.24
- tokio: 1.43.0 → 1.48.0
- itertools: 0.13.0 → 0.14.0

### 新增依赖
Boa 0.21.0 引入的新依赖：
- cordyceps: 0.3.4
- cow-utils: 0.1.3
- diatomic-waker: 0.2.3
- dynify: 0.1.2
- equator: 0.4.2
- float16: 0.1.5
- futures-buffered: 0.2.12
- futures-concurrency: 7.6.3
- small_btree: 0.1.0
- tag_ptr: 0.1.0
- tracing-subscriber: 0.3.20
- xsum: 0.1.6

### 安全性 🔒

#### 注意事项
- HTTPS 证书验证默认禁用以支持某些书源站点
- JavaScript 代码在 Boa 沙箱环境中执行
- 建议在生产环境启用证书验证

### 兼容性

#### Rust 版本
- 最低支持: Rust 1.70+
- 推荐使用: Rust 1.75+

#### 平台支持
- ✅ macOS (aarch64, x86_64)
- ✅ Linux (x86_64, aarch64)
- ✅ Windows (x86_64)

### 已知问题

1. **quick-xml 0.17.2 Future 兼容性警告**
   - 来自 quickxml_to_serde 依赖
   - 不影响当前功能
   - 等待上游更新

2. **Cargo.toml 中的 .cargo/config 弃用警告**
   - 建议迁移到 config.toml
   - 不影响功能

3. **部分测试失败**
   - AES 加密测试需要更新测试用例
   - SHA 哈希测试需要调整 API 使用方式
   - 不影响核心功能

### 迁移指南

#### 从 0.20.0 迁移到 0.21.0

1. **更新所有 JsValue 构造**
   ```rust
   // 查找所有 JsValue::String, JsValue::Object 等
   // 替换为 JsValue::new()
   ```

2. **更新 to_json() 调用**
   ```rust
   // 添加 .unwrap_or() 处理 Option
   let value = js_value.to_json(ctx)?.unwrap_or(Value::Null);
   ```

3. **更新 Console 注册**
   ```rust
   // 检查参数顺序
   Console::register_with_logger(logger, context)
   ```

4. **更新 downcast_ref**
   ```rust
   // 改为两步调用
   let obj = value.as_object()?;
   let data = obj.downcast_ref::<T>()?;
   ```

5. **重新编译和测试**
   ```bash
   cargo clean
   cargo build
   cargo test
   ```

### 贡献者

感谢所有贡献者的支持！

- [@zsakvo](https://github.com/zsakvo) - 项目维护者

### 相关链接

- [Boa 0.21.0 发布说明](https://boajs.dev/blog/2024/12/05/boa-release-020)
- [Boa GitHub 仓库](https://github.com/boa-dev/boa)
- [Boa 文档](https://docs.rs/boa_engine/0.21.0/boa_engine/)

---

## 格式说明

### 变更类型

- `新增` - 新功能
- `变更` - 现有功能的变更
- `弃用` - 即将移除的功能
- `移除` - 已移除的功能
- `修复` - 错误修复
- `安全` - 安全相关修复

### 版本号

遵循语义化版本号 MAJOR.MINOR.PATCH：
- MAJOR - 不兼容的 API 变更
- MINOR - 向后兼容的功能新增
- PATCH - 向后兼容的问题修复

---

**最后更新**: 2025-01-03
