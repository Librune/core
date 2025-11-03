# BookCore 项目架构文档

## 项目概述

**BookCore** 是一个基于 Rust 和 Boa JavaScript 引擎的小说书源解析核心库。它为网络小说阅读应用提供灵活的书源脚本执行环境，支持多种加密算法、HTTP 请求、HTML 解析等功能。

- **项目名称**: book_core
- **版本**: 0.1.0
- **Rust Edition**: 2021
- **许可证**: GNU General Public License v3.0
- **核心依赖**: boa_engine 0.21.0

## 核心特性

### 1. JavaScript 脚本执行引擎
- 基于 **Boa 0.21.0** JavaScript 引擎
- 支持 94.12% 的 ECMAScript 规范
- 提供沙箱环境，安全执行第三方书源脚本
- 支持异步操作的同步化处理

### 2. 加密与哈希支持
#### AES 加密
- **支持的 AES 类型**: AES-128, AES-192, AES-256
- **支持的加密模式**: CBC, CFB, OFB
- **支持的填充方式**: PKCS7, NoPadding, ZeroPadding, ISO10126, AnsiX923, ISO7816
- **支持的编码格式**: Base64, Hex

#### HMAC 哈希
- **支持的算法**: MD5, SHA-1, SHA-256, SHA-384, SHA-512

#### 直接哈希
- **支持的算法**: MD5, SHA-1, SHA-224, SHA-256, SHA-384, SHA-512

### 3. HTTP 网络请求
- 基于 `reqwest` 的异步 HTTP 客户端
- 支持 GET, POST, PUT, DELETE 方法
- 自动 HTTPS 证书验证禁用（用于特殊书源站点）
- **智能字符集检测**（GBK/GB18030/UTF-8 等）
- 支持自定义请求头、查询参数、请求体
- 可配置超时时间

### 4. HTML 解析
- 基于 `scraper` 库的 HTML 解析
- CSS 选择器支持
- 文本提取功能

### 5. 实用工具
- UUID 生成和验证（v4）
- 随机字符串生成
- XML 到 JSON 转换
- Base64/Hex/GBK 编码
- 对象序列化为查询字符串

## 架构设计

### 目录结构

```
core/
├── Cargo.toml              # 项目配置和依赖管理
├── LICENSE                 # GPL-3.0 许可证
├── src/                    # 源代码目录
│   ├── lib.rs              # 库入口，BookCore 核心结构
│   ├── runtime.rs          # JavaScript 运行时初始化
│   │
│   ├── crypto/             # 加密模块
│   │   ├── mod.rs          # 模块导出
│   │   ├── aes.rs          # AES 加密/解密实现
│   │   ├── hmac.rs         # HMAC 哈希实现
│   │   └── enums.rs        # 加密相关枚举类型
│   │
│   ├── request/            # HTTP 请求模块
│   │   ├── mod.rs          # 模块导出
│   │   ├── jreqwest.rs     # Reqwest 封装
│   │   ├── options.rs      # 请求选项配置
│   │   └── charset.rs      # 字符集检测和转换
│   │
│   ├── scraper/            # HTML 解析模块
│   │   ├── mod.rs          # 模块导出
│   │   └── jscraper.rs     # Scraper 封装
│   │
│   ├── global/             # 全局工具函数
│   │   ├── mod.rs          # 模块导出
│   │   ├── console.rs      # 控制台日志
│   │   ├── uuid.rs         # UUID 生成/验证
│   │   ├── xml2json.rs     # XML 转 JSON
│   │   └── rand_str.rs     # 随机字符串生成
│   │
│   ├── env/                # 环境变量管理
│   │   ├── mod.rs          # 模块导出
│   │   └── env.rs          # 环境变量存储
│   │
│   └── prototype/          # JavaScript 原型扩展
│       ├── mod.rs          # 模块导出
│       ├── string.rs       # String 原型扩展
│       └── object.rs       # Object 原型扩展
│
└── tests/                  # 测试文件
    ├── core.rs             # 核心功能测试
    ├── crypto.rs           # AES 加密测试
    ├── hmac.rs             # HMAC 测试
    ├── sha.rs              # SHA 哈希测试
    ├── env.rs              # 环境变量测试
    └── wk8.js              # JavaScript 测试脚本
```

### 核心组件

#### 1. BookCore 结构体 (lib.rs)

BookCore 是项目的核心，封装了 Boa JavaScript 执行上下文：

```rust
pub struct BookCore {
    pub context: Context,
}
```

**主要方法**：
- `new()` - 初始化 JavaScript 运行时
- `eval<T>()` - 执行 JavaScript 代码并返回结果
- `call_func()` - 调用 JavaScript 函数
- `regist_cust_logger()` - 注册自定义日志处理器
- `search()` - 执行书源搜索
- `detail()` - 获取书籍详情
- `catalog()` - 获取目录列表
- `chapter()` - 获取章节内容
- `set_env()`/`get_env()` - 环境变量管理

#### 2. 数据模型

```rust
// 书源元数据
pub struct MetaData {
    pub name: String,
    pub uuid: String,
    pub base_url: String,
    pub author: Option<String>,
    pub user_agent: Option<String>,
    // ...更多字段
}

// 搜索结果
pub struct SearchBook {
    pub name: String,
    pub url: String,
    pub author: Option<String>,
    pub cover: Option<String>,
    pub category: Option<String>,
}

// 书籍详情
pub struct BookDetail {
    pub name: String,
    pub author: Option<String>,
    pub cover: Option<String>,
    pub description: Option<String>,
    pub category: Option<Vec<String>>,
}

// 目录章节
pub struct CatalogChapter {
    pub name: String,
    pub url: String,
}

// 章节内容
pub struct Chapter {
    pub content: String,
}
```

#### 3. 加密模块 (crypto/)

**AES 加密类**：
```javascript
const aes = new Aes({
    cipherMode: "cbc",      // 加密模式: cbc/cfb/ofb
    aesType: "aes256",      // AES 类型: aes128/aes192/aes256
    paddingType: "pkcs7",   // 填充方式: pkcs7/nopadding/zeropadding等
    encoding: "base64",     // 编码格式: base64/hex
    key: keyBytes,          // 密钥
    iv: ivBytes             // 初始化向量
});

const encrypted = aes.encrypt("plaintext");
const decrypted = aes.decrypt(encrypted);
```

**HMAC 哈希类**：
```javascript
const hmac = new Hmac({
    hash: "sha256",         // 哈希算法: md5/sha1/sha256/sha384/sha512
    key: "secret-key",      // 密钥
    encoding: "hex"         // 编码格式: base64/hex
});

const signature = hmac.update("message");
```

#### 4. HTTP 请求模块 (request/)

**JReqwest 类**：
```javascript
const response = JReqwest.get(url, {
    headers: {              // 自定义请求头
        "User-Agent": "...",
        "Cookie": "..."
    },
    query: {                // 查询参数
        page: 1,
        size: 20
    },
    timeout: 10,            // 超时时间（秒）
    gbk: true              // 使用 GBK 编码
});

// 响应对象
{
    body: "...",           // 响应体
    status: 200,           // 状态码
    headers: {...}         // 响应头
}
```

**字符集检测流程**：
1. 从 Content-Type header 提取 charset
2. 从 HTML meta 标签提取 charset
3. 使用 chardet 库自动检测
4. 默认使用 UTF-8

#### 5. JavaScript 原型扩展 (prototype/)

**String 原型扩展**：
```javascript
"中文".toGbk()              // GBK URL 编码
"text".toBase64()           // Base64 编码
"password".toMd5()          // MD5 哈希
"ABC".toAscii()             // [65, 66, 67]
"text".toSha("256")         // SHA-256 哈希
"text".toSha("256", true)   // SHA-256 哈希（返回字符串）
```

**Object 原型扩展**：
```javascript
{a: 1, b: 2}.toQuery()      // "a=1&b=2"
```

#### 6. 环境变量管理 (env/)

```javascript
// 设置单个环境变量
setEnv("token", "abc123");

// 获取单个环境变量
const token = getEnv("token");

// 批量设置环境变量
setEnvs({user: "name", pass: "pwd"});

// 获取所有环境变量
const allEnvs = getEnvs();

// 清空所有环境变量
clearEnvs();
```

## 技术栈

### 核心依赖

| 依赖库 | 版本 | 用途 |
|--------|------|------|
| **boa_engine** | 0.21.0 | JavaScript 引擎核心 |
| **boa_runtime** | 0.21.0 | JavaScript 运行时 |
| **boa_gc** | 0.21.0 | 垃圾回收器 |
| **reqwest** | 0.12.24 | HTTP 客户端 |
| **tokio** | 1.48.0 | 异步运行时 |
| **scraper** | 0.22.0 | HTML 解析 |
| **serde/serde_json** | 1.0 | 序列化/反序列化 |

### 加密依赖

| 依赖库 | 版本 | 用途 |
|--------|------|------|
| **aes** | 0.8.4 | AES 加密算法 |
| **cbc/cfb-mode/ofb** | - | 加密模式 |
| **sha1/sha2** | 0.10+ | SHA 哈希算法 |
| **md-5** | 0.10.6 | MD5 哈希 |
| **hmac** | 0.12.1 | HMAC 实现 |
| **base64** | 0.22.1 | Base64 编码 |
| **hex** | 0.4.3 | 十六进制编码 |

### 编码工具

| 依赖库 | 版本 | 用途 |
|--------|------|------|
| **encoding_rs** | 0.8.35 | 字符编码转换 |
| **chardet** | 0.2.4 | 字符集检测 |
| **regex** | 1.12.2 | 正则表达式 |
| **uuid** | 1.18.1 | UUID 生成 |

## 升级说明

### Boa 0.20.0 → 0.21.0 主要变化

#### 1. API 变化
- `JsValue::String/Object/Boolean/Integer` 已移除
- 新的 `JsValue::new()` 方法用于创建值
- `to_json()` 返回类型从 `Result<Value>` 改为 `Result<Option<Value>>`
- `Console::register_with_logger()` 参数顺序变化
- `downcast_ref()` 不再支持链式调用

#### 2. 兼容性修改
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

```rust
// 旧版本 (0.20.0)
let value = js_value.to_json(ctx).unwrap();

// 新版本 (0.21.0)
let value = js_value.to_json(ctx).unwrap().unwrap_or(serde_json::Value::Null);
```

#### 3. 性能优化
- JsValue 现在使用 Nan-boxing 表示，减少内存占用
- 从堆栈式虚拟机转为寄存器式虚拟机
- 改进的异步 Rust 支持

#### 4. 新增功能
- Temporal API 支持（97% 一致性）
- 增强的错误回溯支持
- 新的宏用于创建 ECMAScript 值、类和模块

## 应用场景

### 1. 多书源聚合阅读器
用户可以通过 JavaScript 编写不同网站的书源规则：

```javascript
// 书源脚本示例
const metadata = {
    name: "某小说网",
    uuid: "unique-id",
    baseUrl: "https://example.com",
    author: "zsakvo",
    version: "1.0.0"
};

function search({key, page}) {
    const response = JReqwest.get(`${metadata.baseUrl}/search`, {
        query: { q: key, p: page }
    });
    // 解析并返回搜索结果
    return books;
}

function detail({url}) {
    const response = JReqwest.get(url);
    // 解析并返回书籍详情
    return detail;
}

function catalog({url}) {
    const response = JReqwest.get(url);
    // 解析并返回目录列表
    return chapters;
}

function chapter({url}) {
    const response = JReqwest.get(url);
    // 解析并返回章节内容
    return {content: "..."};
}
```

### 2. 内容爬虫和数据采集
- 小说章节批量下载
- 图书信息收集
- 内容更新监控

### 3. 加密内容解密
- 解密 AES 加密的章节内容
- 处理需要签名验证的 API（HMAC）
- 解码特殊编码的文本（GBK、Base64 等）

### 4. 跨平台书源共享
- 书源脚本可在不同平台间共享
- 通过网络分发和更新
- 社区协作维护

## 性能考虑

### 1. 异步处理
当前使用 `tokio::task::block_in_place` 将异步请求转为同步：
```rust
tokio::task::block_in_place(|| {
    rt.block_on(async {
        // 异步操作
    })
})
```

### 2. 内存管理
- Boa 0.21.0 使用 Nan-boxing 优化内存占用
- 垃圾回收由 boa_gc 管理
- 避免在循环中创建大量临时对象

### 3. 缓存策略
目前未实现缓存，建议在应用层添加：
- HTTP 响应缓存
- DNS 缓存
- 解析结果缓存

## 安全考虑

### 1. 沙箱执行
- JavaScript 代码在 Boa 引擎的沙箱环境中执行
- 无法直接访问文件系统或网络（除了提供的 API）

### 2. 证书验证
- 默认禁用 HTTPS 证书验证以支持某些书源站点
- 生产环境建议启用证书验证

### 3. 输入验证
- 对用户输入的 JavaScript 代码进行语法检查
- 限制执行时间和内存使用

## 未来计划

### 功能增强
- [ ] 增加更多 HTML 解析方法（XPath 支持）
- [ ] 支持 WebSocket 连接
- [ ] 添加图片下载和处理能力
- [ ] Cookie 管理和会话持久化
- [ ] 连接池和请求重试机制

### 性能优化
- [ ] 实现 HTTP 连接池
- [ ] 添加缓存机制
- [ ] 并发请求控制
- [ ] JavaScript 执行超时控制

### 开发体验
- [ ] 添加调试工具
- [ ] 书源脚本开发 SDK
- [ ] 在线书源市场
- [ ] 更完善的错误提示

## 贡献指南

### 代码规范
- 遵循 Rust 官方代码规范
- 使用 `rustfmt` 格式化代码
- 使用 `clippy` 进行代码检查

### 提交规范
- feat: 新功能
- fix: 修复问题
- docs: 文档更新
- refactor: 代码重构
- test: 测试相关
- chore: 构建/工具相关

## 许可证

本项目采用 GNU General Public License v3.0 许可证。

---

**最后更新**: 2025-01-03
**当前版本**: 0.1.0
**Boa 版本**: 0.21.0
