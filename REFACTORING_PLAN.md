# BookCore 项目重构优化方案

## 概述

基于项目深入分析，制定全面的重构优化计划，提升代码质量、性能和可维护性。

## 一、模块结构重构

### 当前问题
- 模块划分不够清晰，职责混杂
- 缺少统一的错误处理机制
- 没有明确的层次结构

### 优化方案

#### 1.1 新的模块结构

```
src/
├── lib.rs                      # 公共 API 导出
├── core/                       # 核心模块
│   ├── mod.rs
│   ├── context.rs              # BookCore 主结构
│   ├── runtime.rs              # JS 运行时管理
│   └── error.rs                # 统一错误类型
│
├── crypto/                     # 加密模块（完整重构）
│   ├── mod.rs
│   ├── cipher/                 # 对称加密
│   │   ├── mod.rs
│   │   ├── aes.rs              # AES 加密
│   │   ├── des.rs              # DES 加密（新增）
│   │   └── triple_des.rs       # 3DES 加密（新增）
│   ├── asymmetric/             # 非对称加密（新增）
│   │   ├── mod.rs
│   │   └── rsa.rs              # RSA 加密（新增）
│   ├── hash/                   # 哈希算法
│   │   ├── mod.rs
│   │   ├── md5.rs
│   │   ├── sha.rs
│   │   └── hmac.rs
│   ├── encoding/               # 编码工具
│   │   ├── mod.rs
│   │   ├── base64.rs
│   │   └── hex.rs
│   └── types.rs                # 共享类型定义
│
├── network/                    # 网络模块（重命名自 request）
│   ├── mod.rs
│   ├── client.rs               # HTTP 客户端
│   ├── request.rs              # 请求构建器
│   ├── response.rs             # 响应处理
│   ├── charset.rs              # 字符集检测
│   └── cookie.rs               # Cookie 管理（新增）
│
├── parser/                     # 解析模块（重命名自 scraper）
│   ├── mod.rs
│   ├── html.rs                 # HTML 解析
│   ├── json.rs                 # JSON 解析（新增）
│   └── xml.rs                  # XML 解析
│
├── runtime/                    # JS 运行时扩展
│   ├── mod.rs
│   ├── console.rs              # 控制台 API
│   ├── timer.rs                # 定时器 API（新增）
│   ├── fetch.rs                # Fetch API（新增）
│   └── encoding.rs             # TextEncoder/Decoder（新增）
│
├── extensions/                 # JS 原型扩展
│   ├── mod.rs
│   ├── string.rs               # String 扩展
│   ├── object.rs               # Object 扩展
│   ├── array.rs                # Array 扩展（新增）
│   └── promise.rs              # Promise 扩展（新增）
│
├── storage/                    # 存储模块（重命名自 env）
│   ├── mod.rs
│   ├── env.rs                  # 环境变量
│   └── cache.rs                # 缓存管理（新增）
│
└── utils/                      # 工具模块（重命名自 global）
    ├── mod.rs
    ├── uuid.rs
    ├── random.rs               # 随机数生成
    └── url.rs                  # URL 工具（新增）
```

## 二、命名规范统一

### 2.1 函数命名

**当前问题**：
- `regist_xxx` 拼写错误（应为 register）
- 命名不一致（toGbk vs to_gbk）
- 缺少动词前缀

**优化方案**：

```rust
// 注册函数：register_xxx
register_uuid()
register_console()
register_fetch_api()

// 扩展函数：extend_xxx_prototype
extend_string_prototype()
extend_object_prototype()

// 创建函数：create_xxx
create_http_client()
create_cipher()

// 初始化函数：init_xxx
init_runtime()
init_crypto_module()
```

### 2.2 类型命名

```rust
// 结果类型
pub type Result<T> = std::result::Result<T, BookCoreError>;

// 错误类型
pub enum BookCoreError {
    JavaScript(JsError),
    Network(NetworkError),
    Crypto(CryptoError),
    Parse(ParseError),
}

// 配置类型
pub struct CipherConfig { ... }
pub struct HttpConfig { ... }
pub struct RuntimeConfig { ... }
```

## 三、错误处理优化

### 3.1 自定义错误类型

```rust
// src/core/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BookCoreError {
    #[error("JavaScript error: {0}")]
    JavaScript(#[from] boa_engine::JsError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Custom(String),
}

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Connection timeout")]
    Timeout,

    #[error("Charset detection failed")]
    CharsetError,
}

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("Invalid IV length: expected {expected}, got {actual}")]
    InvalidIvLength { expected: usize, actual: usize },

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("HTML parse error: {0}")]
    Html(String),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("XML parse error: {0}")]
    Xml(String),
}
```

## 四、性能优化

### 4.1 HTTP 连接池

```rust
// src/network/client.rs
use reqwest::Client;
use once_cell::sync::Lazy;

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        .timeout(Duration::from_secs(30))
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to create HTTP client")
});

pub fn get_client() -> &'static Client {
    &HTTP_CLIENT
}
```

### 4.2 响应缓存

```rust
// src/storage/cache.rs
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct Cache<T> {
    data: HashMap<String, CacheEntry<T>>,
    ttl: Duration,
}

struct CacheEntry<T> {
    value: T,
    created_at: Instant,
}

impl<T: Clone> Cache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: HashMap::new(),
            ttl,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<T> {
        if let Some(entry) = self.data.get(key) {
            if entry.created_at.elapsed() < self.ttl {
                return Some(entry.value.clone());
            }
            self.data.remove(key);
        }
        None
    }

    pub fn set(&mut self, key: String, value: T) {
        self.data.insert(key, CacheEntry {
            value,
            created_at: Instant::now(),
        });
    }

    pub fn clear_expired(&mut self) {
        self.data.retain(|_, entry| entry.created_at.elapsed() < self.ttl);
    }
}
```

### 4.3 字符串池化

```rust
// 使用 Arc<str> 而不是 String 来减少克隆开销
use std::sync::Arc;

pub struct MetaData {
    pub name: Arc<str>,
    pub uuid: Arc<str>,
    pub base_url: Arc<str>,
    // ...
}
```

## 五、扩展加密方案

### 5.1 DES/3DES 加密

```rust
// src/crypto/cipher/des.rs
use des::Des;
use cipher::{BlockEncrypt, BlockDecrypt, KeyInit};

pub struct DesCipher {
    cipher: Des,
}

impl DesCipher {
    pub fn new(key: &[u8]) -> Result<Self> {
        if key.len() != 8 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 8,
                actual: key.len(),
            });
        }
        Ok(Self {
            cipher: Des::new_from_slice(key)?,
        })
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // 实现 DES 加密
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // 实现 DES 解密
    }
}
```

### 5.2 RSA 加密

```rust
// src/crypto/asymmetric/rsa.rs
use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt};

pub struct RsaCipher {
    public_key: Option<RsaPublicKey>,
    private_key: Option<RsaPrivateKey>,
}

impl RsaCipher {
    pub fn from_public_key(pem: &str) -> Result<Self> {
        // 从 PEM 加载公钥
    }

    pub fn from_private_key(pem: &str) -> Result<Self> {
        // 从 PEM 加载私钥
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // RSA 加密
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // RSA 解密
    }

    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        // RSA 签名
    }

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool> {
        // RSA 验证
    }
}
```

## 六、浏览器 API 扩展

### 6.1 Fetch API

```rust
// src/runtime/fetch.rs
use boa_engine::{Context, JsValue, JsResult};

pub fn register_fetch(context: &mut Context) -> JsResult<()> {
    let fetch = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let url = args.get_or_undefined(0).to_string(ctx)?;
        let options = args.get_or_undefined(1);

        // 实现 fetch 逻辑
        // 返回 Promise
    });

    context.register_global_builtin_callable(
        js_string!("fetch"),
        2,
        fetch
    )?;

    Ok(())
}
```

### 6.2 定时器 API

```rust
// src/runtime/timer.rs
pub fn register_timers(context: &mut Context) -> JsResult<()> {
    // setTimeout
    let set_timeout = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let callback = args.get_or_undefined(0);
        let delay = args.get_or_undefined(1).to_number(ctx)? as u64;

        // 实现 setTimeout
    });

    // setInterval
    let set_interval = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        // 实现 setInterval
    });

    // clearTimeout / clearInterval
    let clear_timer = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        // 实现清除定时器
    });

    context.register_global_builtin_callable(js_string!("setTimeout"), 2, set_timeout)?;
    context.register_global_builtin_callable(js_string!("setInterval"), 2, set_interval)?;
    context.register_global_builtin_callable(js_string!("clearTimeout"), 1, clear_timer.clone())?;
    context.register_global_builtin_callable(js_string!("clearInterval"), 1, clear_timer)?;

    Ok(())
}
```

### 6.3 编码 API

```rust
// src/runtime/encoding.rs
pub fn register_encoding_apis(context: &mut Context) -> JsResult<()> {
    // atob - base64 解码
    let atob = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let input = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
        let decoded = BASE64.decode(&input)
            .map_err(|e| js_error!("Invalid base64: {}", e))?;
        let result = String::from_utf8(decoded)
            .map_err(|e| js_error!("Invalid UTF-8: {}", e))?;
        Ok(JsValue::new(js_string!(result)))
    });

    // btoa - base64 编码
    let btoa = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let input = args.get_or_undefined(0).to_string(ctx)?.to_std_string_escaped();
        let encoded = BASE64.encode(input.as_bytes());
        Ok(JsValue::new(js_string!(encoded)))
    });

    context.register_global_builtin_callable(js_string!("atob"), 1, atob)?;
    context.register_global_builtin_callable(js_string!("btoa"), 1, btoa)?;

    Ok(())
}
```

### 6.4 URL API

```rust
// src/runtime/url.rs
#[derive(Debug, Trace, Finalize, JsData)]
pub struct JsUrl {
    url: url::Url,
}

impl JsUrl {
    fn parse(input: &str, base: Option<&str>) -> Result<Self> {
        let url = if let Some(base_url) = base {
            url::Url::options()
                .base_url(Some(&url::Url::parse(base_url)?))
                .parse(input)?
        } else {
            url::Url::parse(input)?
        };

        Ok(Self { url })
    }

    // 属性：href, protocol, hostname, port, pathname, search, hash
    fn get_href(&self) -> String {
        self.url.as_str().to_string()
    }

    fn get_protocol(&self) -> String {
        format!("{}:", self.url.scheme())
    }

    // ... 其他属性
}

impl Class for JsUrl {
    const NAME: &'static str = "URL";

    fn data_constructor(/* ... */) -> JsResult<Self> {
        // 实现构造函数
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        // 注册属性和方法
    }
}
```

### 6.5 TextEncoder/TextDecoder

```rust
// src/runtime/encoding.rs
#[derive(Debug, Trace, Finalize, JsData)]
pub struct TextEncoder {
    encoding: String,
}

impl TextEncoder {
    fn encode(&self, input: &str) -> Vec<u8> {
        input.as_bytes().to_vec()
    }
}

#[derive(Debug, Trace, Finalize, JsData)]
pub struct TextDecoder {
    encoding: &'static Encoding,
}

impl TextDecoder {
    fn new(label: Option<&str>) -> Result<Self> {
        let encoding = label
            .and_then(|l| Encoding::for_label(l.as_bytes()))
            .unwrap_or(encoding_rs::UTF_8);

        Ok(Self { encoding })
    }

    fn decode(&self, input: &[u8]) -> String {
        let (cow, _, _) = self.encoding.decode(input);
        cow.into_owned()
    }
}
```

## 七、代码复用优化

### 7.1 通用构建器模式

```rust
// src/crypto/builder.rs
pub struct CipherBuilder {
    algorithm: Algorithm,
    mode: Option<CipherMode>,
    padding: Option<PaddingType>,
    key: Option<Vec<u8>>,
    iv: Option<Vec<u8>>,
}

impl CipherBuilder {
    pub fn new(algorithm: Algorithm) -> Self {
        Self {
            algorithm,
            mode: None,
            padding: None,
            key: None,
            iv: None,
        }
    }

    pub fn mode(mut self, mode: CipherMode) -> Self {
        self.mode = Some(mode);
        self
    }

    pub fn padding(mut self, padding: PaddingType) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn key(mut self, key: Vec<u8>) -> Self {
        self.key = Some(key);
        self
    }

    pub fn iv(mut self, iv: Vec<u8>) -> Self {
        self.iv = Some(iv);
        self
    }

    pub fn build(self) -> Result<Box<dyn Cipher>> {
        match self.algorithm {
            Algorithm::AES128 => Ok(Box::new(AesCipher::new(/* ... */))),
            Algorithm::DES => Ok(Box::new(DesCipher::new(/* ... */))),
            Algorithm::RSA => Ok(Box::new(RsaCipher::new(/* ... */))),
        }
    }
}
```

### 7.2 统一的 Trait 设计

```rust
// src/crypto/traits.rs
pub trait Cipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>>;
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>>;
}

pub trait Hash {
    fn hash(&self, data: &[u8]) -> Vec<u8>;
    fn hash_string(&self, data: &[u8]) -> String {
        hex::encode(self.hash(data))
    }
}

pub trait Signer {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool>;
}
```

## 八、测试优化

### 8.1 单元测试覆盖

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_128_cbc_pkcs7() {
        let cipher = AesCipher::new(/* ... */);
        let plaintext = b"Hello, World!";
        let encrypted = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(plaintext, &decrypted[..]);
    }

    // 更多测试...
}
```

### 8.2 集成测试

```rust
// tests/integration/http_test.rs
#[tokio::test]
async fn test_http_request_with_gbk() {
    let mut core = BookCore::new();
    let result = core.eval::<String>(r#"
        const response = JReqwest.get("http://example.com", {gbk: true});
        response.body;
    "#.to_string()).unwrap();

    assert!(!result.is_empty());
}
```

## 九、文档完善

### 9.1 API 文档

为所有公共 API 添加文档注释：

```rust
/// AES 加密器
///
/// # Examples
///
/// ```rust
/// use book_core::crypto::AesCipher;
///
/// let cipher = AesCipher::builder()
///     .mode(CipherMode::CBC)
///     .padding(PaddingType::PKCS7)
///     .key(key_bytes)
///     .iv(iv_bytes)
///     .build()?;
///
/// let encrypted = cipher.encrypt(b"plaintext")?;
/// let decrypted = cipher.decrypt(&encrypted)?;
/// ```
pub struct AesCipher { /* ... */ }
```

### 9.2 架构文档更新

更新 ARCHITECTURE.md 以反映新的模块结构和设计决策。

### 9.3 迁移指南

创建 MIGRATION.md 指导用户从旧版本迁移：

```markdown
# 从 0.1.0 迁移到 0.2.0

## API 变更

### 加密模块

```rust
// 旧 API
let aes = new Aes({...});

// 新 API
const cipher = Crypto.createCipher('aes-256-cbc', options);
```

### 网络请求

```rust
// 旧 API
JReqwest.get(url, options)

// 新 API
Http.get(url, options)
// 或使用标准 fetch
fetch(url, options)
```
```

## 十、实施路线图

### Phase 1: 基础重构（1-2周）
- [ ] 创建新的错误类型系统
- [ ] 重构模块结构
- [ ] 统一命名规范
- [ ] 添加基础单元测试

### Phase 2: 性能优化（1周）
- [ ] 实现 HTTP 连接池
- [ ] 添加缓存机制
- [ ] 优化内存使用

### Phase 3: 功能扩展（2周）
- [ ] 添加 DES/3DES/RSA 加密
- [ ] 实现浏览器 API（fetch, setTimeout, etc.）
- [ ] 添加更多 JS 原型扩展

### Phase 4: 文档和测试（1周）
- [ ] 完善 API 文档
- [ ] 增加测试覆盖率到 80%+
- [ ] 编写迁移指南
- [ ] 更新示例代码

### Phase 5: 发布准备（3-5天）
- [ ] 性能基准测试
- [ ] 安全审计
- [ ] 发布 0.2.0-beta
- [ ] 收集反馈并修复问题

## 十一、风险和注意事项

### 破坏性变更
- 模块路径变更可能影响现有代码
- API 重命名需要提供兼容层或明确的迁移路径

### 性能影响
- 新增功能可能增加二进制大小
- 需要进行性能回归测试

### 安全考虑
- RSA 密钥管理需要特别注意
- 缓存机制可能引入安全风险
- 新的 API 需要沙箱验证

## 十二、成功指标

- [ ] 编译警告 = 0
- [ ] 测试覆盖率 >= 80%
- [ ] 文档覆盖率 >= 90%
- [ ] 性能提升 >= 20%
- [ ] 代码复用率提升 >= 30%
- [ ] 用户满意度 >= 4.5/5

---

**文档版本**: 1.0
**创建日期**: 2025-01-03
**预计完成**: 2025-02-28
