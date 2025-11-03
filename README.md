# BookCore

<div align="center">

**基于 Boa JavaScript 引擎的小说书源解析核心库**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Boa](https://img.shields.io/badge/boa-0.21.0-blue.svg)](https://github.com/boa-dev/boa)
[![License](https://img.shields.io/badge/license-GPL--3.0-green.svg)](LICENSE)

[功能特性](#功能特性) •
[快速开始](#快速开始) •
[文档](#文档) •
[升级说明](#升级说明) •
[贡献](#贡献)

</div>

## 项目简介

BookCore 是一个高性能、功能完善的小说书源解析引擎，使用 Rust 语言开发，内置 Boa JavaScript 引擎作为脚本执行环境。它为网络小说阅读应用提供灵活的书源管理能力，支持用户通过 JavaScript 脚本定义不同网站的解析规则。

### 主要特点

- 🚀 **高性能**: Rust 底层实现，Boa 0.21.0 使用寄存器式虚拟机和 Nan-boxing 优化
- 🔒 **安全可靠**: JavaScript 沙箱执行环境，隔离第三方脚本
- 🔐 **加密支持**: 完整的 AES 加密（3种模式、6种填充）和 HMAC 签名
- 🌐 **网络功能**: 智能字符集检测，支持 GBK/GB18030 等中文编码
- 📦 **开箱即用**: 丰富的内置 API，JavaScript 原型扩展
- 🔧 **易于扩展**: 清晰的模块化设计，便于添加新功能

## 功能特性

### 核心功能

- ✅ JavaScript 脚本执行（Boa 0.21.0, ECMAScript 94.12% 兼容）
- ✅ 书源脚本管理（搜索、详情、目录、章节）
- ✅ 环境变量持久化
- ✅ 自定义日志处理器

### 加密与哈希

| 功能 | 支持算法/模式 |
|------|--------------|
| **AES 加密** | AES-128/192/256, CBC/CFB/OFB |
| **填充方式** | PKCS7, NoPadding, ZeroPadding, ISO10126, AnsiX923, ISO7816 |
| **HMAC** | MD5, SHA-1, SHA-256, SHA-384, SHA-512 |
| **哈希** | MD5, SHA-1, SHA-224, SHA-256, SHA-384, SHA-512 |
| **编码** | Base64, Hex, GBK |

### HTTP 请求

- ✅ GET/POST/PUT/DELETE 方法
- ✅ 自定义请求头、查询参数、请求体
- ✅ 智能字符集检测（Content-Type, HTML Meta, chardet）
- ✅ GBK/GB18030 编码支持
- ✅ 超时控制
- ✅ HTTPS 证书验证配置

### 工具函数

- ✅ HTML 解析（基于 scraper）
- ✅ XML 转 JSON
- ✅ UUID 生成和验证
- ✅ 随机字符串生成
- ✅ JavaScript 原型扩展（String, Object）

## 快速开始

### 前置要求

- Rust 1.70 或更高版本
- Cargo 包管理器

### 安装

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
book_core = { path = "path/to/book_core" }
tokio = { version = "1.43", features = ["rt-multi-thread"] }
```

### 基本使用

```rust
use book_core::BookCore;

#[tokio::main]
async fn main() {
    // 创建 BookCore 实例
    let mut core = BookCore::new();

    // 加载书源脚本
    let script = r#"
        const metadata = {
            name: "示例书源",
            uuid: "example-source",
            baseUrl: "https://example.com"
        };

        function search({key, page}) {
            const response = JReqwest.get(`${metadata.baseUrl}/search`, {
                query: {q: key, p: page}
            });

            return [
                {
                    name: "书名",
                    url: "https://example.com/book/123",
                    author: "作者"
                }
            ];
        }

        function detail({url}) {
            const response = JReqwest.get(url);
            return {
                name: "书名",
                author: "作者",
                description: "简介..."
            };
        }

        function catalog({url}) {
            const response = JReqwest.get(url);
            return [
                {
                    name: "正文",
                    chapters: [
                        {name: "第一章", url: "..."},
                        {name: "第二章", url: "..."}
                    ]
                }
            ];
        }

        function chapter({url}) {
            const response = JReqwest.get(url);
            return {content: "章节内容..."};
        }
    "#;

    core.eval::<()>(script.to_string()).unwrap();

    // 使用书源方法
    let results = core.search("关键词".to_string(), 1, 20).unwrap();
    println!("搜索到 {} 本书", results.len());

    let detail = core.detail(results[0].url.clone()).unwrap();
    println!("书名: {}", detail.name);

    let catalog = core.catalog(results[0].url.clone()).unwrap();
    println!("共 {} 卷", catalog.len());

    let chapter = core.chapter(catalog[0].chapters[0].url.clone()).unwrap();
    println!("内容长度: {} 字", chapter.content.len());
}
```

### JavaScript 书源脚本示例

```javascript
// HTTP 请求
const response = JReqwest.get(url, {
    headers: {"User-Agent": "..."},
    query: {page: 1},
    timeout: 10,
    gbk: false
});

// HTML 解析
const scraper = new JScraper(response.body);
const text = scraper.text();

// AES 解密
const aes = new Aes({
    cipherMode: "cbc",
    aesType: "aes256",
    paddingType: "pkcs7",
    encoding: "base64",
    key: keyBytes,
    iv: ivBytes
});
const decrypted = aes.decrypt(encrypted);

// HMAC 签名
const hmac = new Hmac({
    hash: "sha256",
    key: "secret",
    encoding: "hex"
});
const signature = hmac.update("message");

// String 扩展
"中文".toGbk()          // GBK 编码
"text".toBase64()       // Base64 编码
"pass".toMd5()          // MD5 哈希
"text".toSha("256")     // SHA-256 哈希

// Object 扩展
{a: 1, b: 2}.toQuery()  // "a=1&b=2"

// 工具函数
const id = uuid();                  // 生成 UUID
const valid = isUuid(id);           // 验证 UUID
const json = xml2Json(xmlStr);      // XML 转 JSON
const rand = randString(16);        // 随机字符串

// 环境变量
setEnv("key", "value");
const value = getEnv("key");
```

## 文档

完整的文档请查看：

- **[架构文档 (ARCHITECTURE.md)](ARCHITECTURE.md)** - 深入了解项目架构、模块设计和技术栈
- **[使用指导 (USER_GUIDE.md)](USER_GUIDE.md)** - 完整的 API 参考和书源开发教程
- **[Cargo 文档](https://docs.rs/)** - 运行 `cargo doc --open` 查看 API 文档

## 升级说明

### v0.1.0 - Boa 0.21.0 升级

本版本将 Boa JavaScript 引擎从 0.20.0 升级到 0.21.0，带来以下改进：

#### 主要变化

1. **性能提升**
   - 从堆栈式虚拟机改为寄存器式虚拟机
   - JsValue 使用 Nan-boxing 表示，减少内存占用
   - ECMAScript 兼容性从 89.92% 提升到 94.12%

2. **API 变化**
   ```rust
   // 旧 API (0.20.0)
   JsValue::String(text.into())
   JsValue::Object(obj)

   // 新 API (0.21.0)
   JsValue::new(js_string!(text))
   JsValue::new(obj)
   ```

3. **新增功能**
   - Temporal API 支持（97% 一致性）
   - 增强的错误回溯
   - 新的宏用于创建 ECMAScript 值

#### 破坏性变更

- `JsValue::String/Object/Boolean/Integer` 构造器已移除，使用 `JsValue::new()`
- `to_json()` 返回类型从 `Result<Value>` 改为 `Result<Option<Value>>`
- `Console::register_with_logger()` 参数顺序变化：`(logger, context)` → `(context, logger)` (已修正为 `(logger, context)`)
- `downcast_ref()` 不再支持链式调用

完整的升级指南请查看 [ARCHITECTURE.md](ARCHITECTURE.md#升级说明)。

## 项目结构

```
core/
├── src/
│   ├── lib.rs              # 核心 API
│   ├── runtime.rs          # 运行时初始化
│   ├── crypto/             # 加密模块（AES, HMAC）
│   ├── request/            # HTTP 请求（reqwest 封装）
│   ├── scraper/            # HTML 解析（scraper 封装）
│   ├── global/             # 全局工具函数
│   ├── env/                # 环境变量管理
│   └── prototype/          # JavaScript 原型扩展
├── tests/                  # 集成测试
├── Cargo.toml              # 项目配置
├── LICENSE                 # GPL-3.0 许可证
├── README.md               # 本文件
├── ARCHITECTURE.md         # 架构文档
└── USER_GUIDE.md           # 使用指导
```

## 开发

### 构建

```bash
cargo build
```

### 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test --test crypto
cargo test --test hmac
cargo test --test env
```

### 代码检查

```bash
# 格式化代码
cargo fmt

# 运行 clippy
cargo clippy

# 生成文档
cargo doc --open
```

## 依赖项

### 核心依赖

- [boa_engine](https://github.com/boa-dev/boa) 0.21.0 - JavaScript 引擎
- [reqwest](https://github.com/seanmonstar/reqwest) 0.12.24 - HTTP 客户端
- [tokio](https://github.com/tokio-rs/tokio) 1.48.0 - 异步运行时
- [scraper](https://github.com/causal-agent/scraper) 0.22.0 - HTML 解析
- [serde](https://github.com/serde-rs/serde) 1.0 - 序列化框架

### 加密依赖

- aes 0.8.4, cbc 0.1.2, cfb-mode 0.8.2, ofb 0.6.1
- sha1 0.10.6, sha2 0.10.9, md-5 0.10.6
- hmac 0.12.1, base64 0.22.1, hex 0.4.3

### 编码工具

- encoding_rs 0.8.35, chardet 0.2.4
- regex 1.12.2, uuid 1.18.1

完整依赖列表请查看 [Cargo.toml](Cargo.toml)。

## 应用场景

1. **多书源聚合阅读器** - 用户自定义书源脚本，支持多种小说网站
2. **内容爬虫** - 批量下载小说章节，数据采集
3. **加密内容解密** - 处理 AES 加密的章节内容
4. **跨平台书源共享** - JavaScript 书源可在不同平台间共享

## 性能

- Boa 0.21.0 使用寄存器式虚拟机，性能优于 0.20.0
- JsValue 使用 Nan-boxing 优化，减少约 30% 内存占用
- 智能字符集检测，支持 GBK/GB18030 等中文编码
- 异步 HTTP 请求通过 `tokio::task::block_in_place` 同步化

## 安全性

- ✅ JavaScript 代码在 Boa 沙箱环境中执行
- ✅ 无法直接访问文件系统
- ✅ 网络请求受限于提供的 API
- ⚠️ 默认禁用 HTTPS 证书验证（可配置）

## 路线图

- [ ] 增强 HTML 解析（XPath 支持）
- [ ] WebSocket 连接支持
- [ ] 图片下载和处理
- [ ] Cookie 管理和会话持久化
- [ ] HTTP 连接池
- [ ] 请求缓存机制
- [ ] JavaScript 执行超时控制
- [ ] 书源调试工具
- [ ] 书源市场

## 贡献

欢迎贡献代码、报告问题或提出建议！

### 贡献流程

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'feat: Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 代码规范

- 遵循 Rust 官方代码规范
- 使用 `rustfmt` 格式化代码
- 使用 `clippy` 检查代码质量
- 为新功能添加测试

### 提交信息规范

- `feat:` 新功能
- `fix:` 修复问题
- `docs:` 文档更新
- `refactor:` 代码重构
- `test:` 测试相关
- `chore:` 构建/工具相关

## 许可证

本项目采用 [GNU General Public License v3.0](LICENSE) 许可证。

## 致谢

- [Boa](https://github.com/boa-dev/boa) - 优秀的 Rust JavaScript 引擎
- [Reqwest](https://github.com/seanmonstar/reqwest) - 强大的 HTTP 客户端
- [Scraper](https://github.com/causal-agent/scraper) - 简洁的 HTML 解析库
- Rust 社区的所有贡献者

## 联系方式

- 作者: zsakvo
- 项目地址: [GitHub](https://github.com/zsakvo/cc-project)
- 问题反馈: [Issues](https://github.com/zsakvo/cc-project/issues)

---

<div align="center">

**如果这个项目对你有帮助，请给个 ⭐️ 支持一下！**

Made with ❤️ by zsakvo

</div>
