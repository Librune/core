# BookCore 使用指导

## 快速开始

### 安装

在 `Cargo.toml` 中添加依赖：

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
    // 1. 创建 BookCore 实例
    let mut core = BookCore::new();

    // 2. 加载书源脚本
    let script = r#"
        const metadata = {
            name: "示例书源",
            uuid: "example-source",
            baseUrl: "https://example.com"
        };

        function search({key, page}) {
            // 搜索实现
            return [];
        }
    "#;

    core.eval::<()>(script.to_string()).unwrap();

    // 3. 调用书源方法
    let results = core.search("关键词".to_string(), 1, 20).unwrap();
    println!("搜索结果: {:?}", results);
}
```

## 核心 API

### 1. 初始化与配置

#### 创建实例

```rust
let mut core = BookCore::new();
```

#### 注册自定义日志处理器

```rust
use boa_runtime::console::Logger;

struct CustomLogger;

impl Logger for CustomLogger {
    fn log(&self, level: boa_runtime::console::LogLevel, message: &str, _state: &mut ()) -> bool {
        println!("[{}] {}", level, message);
        true
    }
}

core.regist_cust_logger(CustomLogger);
```

### 2. 执行 JavaScript 代码

#### 执行并返回结果

```rust
// 返回字符串
let result: String = core.eval("'Hello, World!'".to_string()).unwrap();

// 返回数字
let num: i32 = core.eval("1 + 2".to_string()).unwrap();

// 返回 JSON 对象
use serde_json::Value;
let obj: Value = core.eval("{a: 1, b: 2}".to_string()).unwrap();
```

#### 调用 JavaScript 函数

```rust
use serde_json::json;

// 定义函数
core.eval::<()>(r#"
    function add(a, b) {
        return a + b;
    }
"#.to_string()).unwrap();

// 调用函数
let result = core.call_func(
    "add".to_string(),
    vec![json!(10), json!(20)]
).unwrap();

println!("Result: {:?}", result);
```

### 3. 书源相关方法

#### 获取元数据

```rust
let metadata = core.get_metadata().unwrap();
println!("书源名称: {}", metadata.name);
println!("作者: {:?}", metadata.author);
println!("版本: {:?}", metadata.version);
```

#### 搜索书籍

```rust
let results = core.search(
    "诡秘之主".to_string(),  // 关键词
    1,                        // 页码
    20                        // 每页数量
).unwrap();

for book in results {
    println!("书名: {}", book.name);
    println!("作者: {:?}", book.author);
    println!("URL: {}", book.url);
}
```

#### 获取书籍详情

```rust
let detail = core.detail("https://example.com/book/123".to_string()).unwrap();
println!("书名: {}", detail.name);
println!("简介: {:?}", detail.description);
println!("分类: {:?}", detail.category);
```

#### 获取目录

```rust
let catalog = core.catalog("https://example.com/book/123".to_string()).unwrap();

for volume in catalog {
    println!("卷名: {}", volume.name);
    for chapter in volume.chapters {
        println!("  章节: {} - {}", chapter.name, chapter.url);
    }
}
```

#### 获取章节内容

```rust
let chapter = core.chapter("https://example.com/chapter/456".to_string()).unwrap();
println!("内容: {}", chapter.content);
```

### 4. 环境变量管理

```rust
// 设置单个环境变量
core.set_env("token".to_string(), "abc123".to_string()).unwrap();

// 批量设置环境变量
use serde_json::json;
let envs = json!({
    "username": "user",
    "password": "pass"
});
core.set_envs(envs.to_string()).unwrap();

// 获取单个环境变量
let token = core.get_env("token".to_string()).unwrap();

// 获取所有环境变量
let all_envs = core.get_envs().unwrap();

// 清空环境变量
core.clear_envs();
```

### 5. 表单和操作

```rust
// 获取表单配置（用于登录等）
let forms = core.get_forms().unwrap();
for form in forms {
    println!("表单标题: {}", form.title);
    for field in form.fields {
        println!("  字段: {} ({})", field.title, field.hint);
    }
}

// 获取自定义操作
let actions = core.get_actions().unwrap();
for action in actions {
    println!("操作: {} - {}", action.title, action.action);
}
```

## JavaScript 书源脚本开发

### 脚本结构

一个完整的书源脚本包含以下部分：

```javascript
// 1. 元数据定义
const metadata = {
    name: "书源名称",
    uuid: "唯一标识符",
    baseUrl: "https://example.com",
    author: "作者名",
    userAgent: "自定义 User-Agent",
    version: "1.0.0",
    enableProxy: false,
    contentType: "novel"
};

// 2. 可选：表单配置
const forms = [
    {
        title: "登录",
        fields: [
            {name: "username", title: "用户名", hint: "请输入用户名", type: "text"},
            {name: "password", title: "密码", hint: "请输入密码", type: "password"}
        ],
        action: "login"
    }
];

// 3. 可选：自定义操作
const actions = [
    {title: "刷新Token", action: "refreshToken"}
];

// 4. 必需：搜索方法
function search({key, page, count}) {
    const response = JReqwest.get(`${metadata.baseUrl}/search`, {
        query: {q: key, p: page}
    });

    // 解析 HTML
    const scraper = new JScraper(response.body);

    // 返回搜索结果数组
    return [
        {
            name: "书名",
            url: "书籍URL",
            author: "作者",
            cover: "封面URL",
            category: "分类",
            description: "简介",
            status: "连载中",
            wordCount: "100万字"
        }
    ];
}

// 5. 必需：详情方法
function detail({url}) {
    const response = JReqwest.get(url);

    return {
        name: "书名",
        author: "作者",
        cover: "封面URL",
        description: "简介",
        category: ["玄幻", "东方玄幻"],
        status: "连载中",
        wordCount: "100万字",
        updateTime: "2025-01-03",
        latestChapter: "最新章节名"
    };
}

// 6. 必需：目录方法
function catalog({url}) {
    const response = JReqwest.get(url);

    return [
        {
            name: "卷一",
            chapters: [
                {name: "第一章", url: "章节URL"},
                {name: "第二章", url: "章节URL"}
            ]
        }
    ];
}

// 7. 必需：章节方法
function chapter({url}) {
    const response = JReqwest.get(url);

    return {
        content: "章节正文内容..."
    };
}

// 8. 可选：自定义方法处理
function login(fields) {
    const response = JReqwest.post(`${metadata.baseUrl}/login`, {
        form: {
            username: fields.username,
            password: fields.password
        }
    });

    // 保存 token
    setEnv("token", response.token);

    return {success: true, message: "登录成功"};
}
```

### 内置 JavaScript API

#### HTTP 请求 (JReqwest)

```javascript
// GET 请求
const response = JReqwest.get(url, {
    headers: {
        "User-Agent": "Mozilla/5.0...",
        "Cookie": "session=..."
    },
    query: {
        page: 1,
        size: 20
    },
    timeout: 10,  // 超时时间（秒）
    gbk: false    // 是否使用 GBK 编码
});

// POST 请求（表单）
const response = JReqwest.post(url, {
    form: {
        username: "user",
        password: "pass"
    }
});

// POST 请求（JSON）
const response = JReqwest.post(url, {
    json: {
        key: "value"
    }
});

// POST 请求（原始 body）
const response = JReqwest.post(url, {
    body: "raw data",
    headers: {
        "Content-Type": "application/xml"
    }
});

// 响应对象
{
    body: "响应内容",
    status: 200,
    headers: {
        "content-type": "text/html"
    }
}
```

#### HTML 解析 (JScraper)

```javascript
const scraper = new JScraper(html);
const text = scraper.text();  // 提取所有文本
```

#### AES 加密/解密

```javascript
// 创建 AES 实例
const aes = new Aes({
    cipherMode: "cbc",      // cbc/cfb/ofb
    aesType: "aes256",      // aes128/aes192/aes256
    paddingType: "pkcs7",   // pkcs7/nopadding/zeropadding/iso10126/ansix923/iso7816
    encoding: "base64",     // base64/hex
    key: [0, 1, 2, ...],    // 密钥字节数组
    iv: [0, 1, 2, ...]      // IV 字节数组
});

// 加密
const encrypted = aes.encrypt("plaintext");

// 解密
const decrypted = aes.decrypt(encrypted);
```

#### HMAC 签名

```javascript
const hmac = new Hmac({
    hash: "sha256",         // md5/sha1/sha256/sha384/sha512
    key: "secret-key",      // 密钥
    encoding: "hex"         // base64/hex
});

const signature = hmac.update("message");
```

#### String 原型扩展

```javascript
// GBK URL 编码
const encoded = "中文".toGbk();

// Base64 编码
const base64 = "text".toBase64();

// MD5 哈希
const hash = "password".toMd5();

// 转 ASCII 码数组
const codes = "ABC".toAscii();  // [65, 66, 67]

// SHA 哈希
const sha1 = "text".toSha("1");        // SHA-1，返回字节数组
const sha256 = "text".toSha("256");    // SHA-256，返回字节数组
const sha256Str = "text".toSha("256", true);  // SHA-256，返回十六进制字符串
const sha384 = "text".toSha("384");    // SHA-384
const sha512 = "text".toSha("512");    // SHA-512
```

#### Object 原型扩展

```javascript
// 对象转查询字符串
const query = {a: 1, b: "hello"}.toQuery();  // "a=1&b=hello"
```

#### 环境变量

```javascript
// 设置环境变量
setEnv("key", "value");

// 获取环境变量
const value = getEnv("key");

// 批量设置
setEnvs({key1: "value1", key2: "value2"});

// 获取所有环境变量
const all = getEnvs();

// 清空环境变量
clearEnvs();
```

#### UUID 工具

```javascript
// 生成 UUID v4
const id = uuid();

// 验证 UUID
const valid = isUuid("550e8400-e29b-41d4-a716-446655440000");
```

#### XML 转 JSON

```javascript
const xml = '<root><item>value</item></root>';
const json = xml2Json(xml);
```

#### 随机字符串

```javascript
// 生成 16 字节的十六进制随机字符串
const random = randString(16);
```

#### 控制台输出

```javascript
console.log("调试信息");
console.warn("警告信息");
console.error("错误信息");
```

### 完整示例：某小说网书源

```javascript
const metadata = {
    name: "某小说网",
    uuid: "example-novel-site",
    baseUrl: "https://www.example.com",
    author: "Developer",
    userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
    version: "1.0.0",
    contentType: "novel"
};

function search({key, page, count}) {
    // 构建搜索 URL
    const url = `${metadata.baseUrl}/search.html`;

    // 发送请求
    const response = JReqwest.get(url, {
        query: {
            keyword: key,
            page: page
        },
        headers: {
            "User-Agent": metadata.userAgent
        },
        timeout: 10
    });

    // 解析 HTML（实际项目中需要使用 CSS 选择器）
    const scraper = new JScraper(response.body);

    // 这里简化处理，实际需要根据网站结构解析
    return [
        {
            name: "示例书籍",
            url: `${metadata.baseUrl}/book/123.html`,
            author: "示例作者",
            cover: `${metadata.baseUrl}/cover/123.jpg`,
            category: "玄幻",
            description: "这是一本示例书籍",
            status: "连载中",
            wordCount: "100万字"
        }
    ];
}

function detail({url}) {
    const response = JReqwest.get(url);

    return {
        name: "示例书籍",
        author: "示例作者",
        cover: `${metadata.baseUrl}/cover/123.jpg`,
        description: "详细简介...",
        category: ["玄幻", "东方玄幻"],
        status: "连载中",
        wordCount: "100万字",
        updateTime: "2025-01-03",
        latestChapter: "第一千章 大结局"
    };
}

function catalog({url}) {
    const response = JReqwest.get(url);

    // 如果网站使用了 GBK 编码
    // const response = JReqwest.get(url, {gbk: true});

    return [
        {
            name: "正文",
            chapters: [
                {name: "第一章 开始", url: `${metadata.baseUrl}/chapter/1.html`},
                {name: "第二章 修炼", url: `${metadata.baseUrl}/chapter/2.html`}
            ]
        }
    ];
}

function chapter({url}) {
    const response = JReqwest.get(url);

    // 如果内容被加密
    const encrypted = extractEncryptedContent(response.body);
    if (encrypted) {
        // 使用 AES 解密
        const aes = new Aes({
            cipherMode: "cbc",
            aesType: "aes128",
            paddingType: "pkcs7",
            encoding: "base64",
            key: "encryption-key".toAscii(),
            iv: "initial-vector-".toAscii()
        });

        const decrypted = aes.decrypt(encrypted);
        return {content: decrypted};
    }

    // 正常解析
    const scraper = new JScraper(response.body);
    const content = scraper.text();

    return {content: content};
}

function extractEncryptedContent(html) {
    // 提取加密内容的逻辑
    return null;
}
```

## 常见问题

### 1. 字符编码问题

**问题**: 获取的内容出现乱码

**解决方案**:
```javascript
// 使用 GBK 选项
const response = JReqwest.get(url, {
    gbk: true
});
```

### 2. 请求超时

**问题**: 网络请求超时

**解决方案**:
```javascript
// 增加超时时间
const response = JReqwest.get(url, {
    timeout: 30  // 30 秒
});
```

### 3. 需要 Cookie

**问题**: 网站需要 Cookie 才能访问

**解决方案**:
```javascript
// 方案1: 手动设置 Cookie
const response = JReqwest.get(url, {
    headers: {
        "Cookie": "session=xxx; token=yyy"
    }
});

// 方案2: 使用环境变量保存 Cookie
function login(fields) {
    const response = JReqwest.post(`${metadata.baseUrl}/login`, {
        form: fields
    });

    // 保存 Cookie
    const cookie = response.headers["set-cookie"];
    setEnv("cookie", cookie);
}

function search({key, page}) {
    const cookie = getEnv("cookie");
    const response = JReqwest.get(url, {
        headers: {
            "Cookie": cookie
        }
    });
    // ...
}
```

### 4. 内容加密

**问题**: 章节内容被 AES 加密

**解决方案**:
```javascript
function chapter({url}) {
    const response = JReqwest.get(url);

    // 提取加密的 Base64 字符串
    const encrypted = extractFromHTML(response.body);

    // 准备密钥和 IV（根据网站具体情况）
    const key = "your-key-here".toAscii();
    const iv = "your-iv-here--".toAscii();

    // 创建 AES 解密器
    const aes = new Aes({
        cipherMode: "cbc",
        aesType: "aes128",
        paddingType: "pkcs7",
        encoding: "base64",
        key: key,
        iv: iv
    });

    // 解密
    const decrypted = aes.decrypt(encrypted);

    return {content: decrypted};
}
```

### 5. 需要签名验证

**问题**: API 需要 HMAC 签名

**解决方案**:
```javascript
function search({key, page}) {
    const timestamp = Date.now();
    const message = `keyword=${key}&page=${page}&timestamp=${timestamp}`;

    // 计算 HMAC 签名
    const hmac = new Hmac({
        hash: "sha256",
        key: "secret-key",
        encoding: "hex"
    });
    const signature = hmac.update(message);

    // 发送请求
    const response = JReqwest.get(`${metadata.baseUrl}/api/search`, {
        query: {
            keyword: key,
            page: page,
            timestamp: timestamp,
            signature: signature
        }
    });

    return JSON.parse(response.body);
}
```

## 调试技巧

### 1. 使用 console.log

```javascript
function search({key, page}) {
    console.log("搜索关键词:", key);
    console.log("页码:", page);

    const response = JReqwest.get(url);
    console.log("响应状态:", response.status);
    console.log("响应内容:", response.body.substring(0, 100));

    // ...
}
```

### 2. 错误处理

```javascript
function chapter({url}) {
    try {
        const response = JReqwest.get(url);

        if (response.status !== 200) {
            console.error("请求失败，状态码:", response.status);
            return {content: "章节加载失败"};
        }

        // 处理内容
        return {content: processContent(response.body)};

    } catch (error) {
        console.error("发生错误:", error);
        return {content: "章节加载出错"};
    }
}
```

### 3. 数据验证

```javascript
function search({key, page}) {
    const response = JReqwest.get(url);
    const results = parseSearchResults(response.body);

    // 验证数据
    console.log("找到", results.length, "个结果");
    if (results.length > 0) {
        console.log("第一个结果:", JSON.stringify(results[0]));
    }

    return results;
}
```

## 性能优化

### 1. 减少不必要的请求

```javascript
// 缓存元数据
let cachedMetadata = null;

function getMetadata() {
    if (cachedMetadata) {
        return cachedMetadata;
    }

    const response = JReqwest.get(`${metadata.baseUrl}/meta`);
    cachedMetadata = JSON.parse(response.body);
    return cachedMetadata;
}
```

### 2. 使用环境变量缓存

```javascript
function detail({url}) {
    // 检查缓存
    const cacheKey = "detail_" + url.toMd5();
    const cached = getEnv(cacheKey);

    if (cached) {
        return JSON.parse(cached);
    }

    // 获取数据
    const response = JReqwest.get(url);
    const detail = parseDetail(response.body);

    // 保存缓存
    setEnv(cacheKey, JSON.stringify(detail));

    return detail;
}
```

### 3. 优化正则表达式

```javascript
// 不好的做法
for (let item of items) {
    const match = item.match(/pattern/);
}

// 好的做法
const regex = /pattern/;
for (let item of items) {
    const match = item.match(regex);
}
```

## 最佳实践

1. **错误处理**: 始终添加错误处理逻辑
2. **日志记录**: 使用 console.log 记录关键信息
3. **数据验证**: 验证返回的数据格式
4. **超时设置**: 为网络请求设置合理的超时时间
5. **编码处理**: 根据网站编码选择正确的选项
6. **版本控制**: 在 metadata 中记录版本号
7. **代码注释**: 添加必要的注释说明
8. **测试**: 充分测试各种边界情况

---

**文档版本**: 1.0.0
**最后更新**: 2025-01-03
**适用版本**: BookCore 0.1.0 (Boa 0.21.0)
