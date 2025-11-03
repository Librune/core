# Jsoup-Inspired DOM 解析 API 设计文档

**日期**: 2025-01-04
**版本**: 1.0
**目标**: 对标 Jsoup,为 BookCore 设计强大的 DOM 解析 API

---

## 1. 现状分析

### 当前实现 (`JScraper`)

**文件**: [src/scraper/jscraper.rs](src/scraper/jscraper.rs)

**当前功能**:
```javascript
const scraper = new JScraper("<html>...</html>");
const text = scraper.text();  // 仅支持获取文本
```

**问题**:
- ❌ 仅支持 `text()` 一个方法
- ❌ 无法进行 CSS 选择器查询
- ❌ 无法遍历 DOM 树
- ❌ 无法提取属性
- ❌ 无法修改 DOM (不是核心需求,但 Jsoup 支持)
- ❌ 不支持链式调用

### Jsoup 核心功能

Jsoup 是 Java 最流行的 HTML 解析库,核心特性:

1. **CSS 选择器** - `doc.select("div.class")`
2. **DOM 遍历** - `element.parent()`, `element.children()`, `element.siblings()`
3. **属性提取** - `element.attr("href")`, `element.attributes()`
4. **文本提取** - `element.text()`, `element.ownText()`, `element.html()`
5. **链式调用** - `doc.select("a").first().attr("href")`
6. **表单操作** - `form.select("input[name=username]")`
7. **URL 解析** - `element.absUrl("href")`

---

## 2. 设计目标

### 核心原则

✅ **浏览器兼容性** - API 风格接近浏览器 DOM API 和 Jsoup
✅ **书源友好** - 针对书源解析场景优化
✅ **链式调用** - 支持流畅的方法链
✅ **类型安全** - 充分利用 Rust 类型系统
✅ **高性能** - 基于高效的 scraper crate (基于 html5ever)

### API 设计哲学

**Jsoup 风格**:
```java
// Java Jsoup
Document doc = Jsoup.parse(html);
String title = doc.select("h1.title").first().text();
String link = doc.select("a.next").attr("href");
```

**BookCore 对应**:
```javascript
// JavaScript BookCore
const doc = new JDocument(html);
const title = doc.select("h1.title").first().text();
const link = doc.select("a.next").attr("href");
```

---

## 3. API 设计

### 3.1 核心类层次结构

```
JDocument (文档根节点)
    ├─ select(selector) → JElements (元素集合)
    ├─ selectFirst(selector) → JElement | null
    ├─ getElementById(id) → JElement | null
    ├─ getElementsByTag(tag) → JElements
    ├─ getElementsByClass(className) → JElements
    └─ body() → JElement

JElement (单个元素)
    ├─ 查询方法
    │   ├─ select(selector) → JElements
    │   ├─ selectFirst(selector) → JElement | null
    │   ├─ getElementById(id) → JElement | null
    │   ├─ getElementsByTag(tag) → JElements
    │   └─ getElementsByClass(className) → JElements
    │
    ├─ 遍历方法
    │   ├─ parent() → JElement | null
    │   ├─ parents() → JElements
    │   ├─ children() → JElements
    │   ├─ child(index) → JElement | null
    │   ├─ nextElementSibling() → JElement | null
    │   ├─ previousElementSibling() → JElement | null
    │   ├─ firstElementChild() → JElement | null
    │   └─ lastElementChild() → JElement | null
    │
    ├─ 属性方法
    │   ├─ attr(name) → String
    │   ├─ hasAttr(name) → Boolean
    │   ├─ removeAttr(name) → JElement
    │   ├─ className() → String
    │   ├─ classNames() → Array<String>
    │   ├─ hasClass(className) → Boolean
    │   ├─ id() → String
    │   └─ tagName() → String
    │
    ├─ 文本/HTML 方法
    │   ├─ text() → String (所有后代文本)
    │   ├─ ownText() → String (仅直接文本)
    │   ├─ html() → String (内部 HTML)
    │   ├─ outerHtml() → String (包含自身的 HTML)
    │   └─ data() → String (script/style 标签的数据)
    │
    └─ 判断方法
        ├─ is(selector) → Boolean
        ├─ hasText() → Boolean
        └─ isEmpty() → Boolean

JElements (元素集合)
    ├─ 访问方法
    │   ├─ get(index) → JElement | null
    │   ├─ first() → JElement | null
    │   ├─ last() → JElement | null
    │   ├─ size() → Number
    │   └─ isEmpty() → Boolean
    │
    ├─ 批量查询
    │   ├─ select(selector) → JElements
    │   └─ filter(selector) → JElements
    │
    ├─ 批量提取
    │   ├─ text() → String (合并所有文本)
    │   ├─ texts() → Array<String>
    │   ├─ html() → String
    │   ├─ outerHtml() → String
    │   ├─ attr(name) → String (第一个元素的属性)
    │   └─ eachAttr(name) → Array<String>
    │
    ├─ 遍历方法
    │   ├─ parents() → JElements
    │   └─ forEach(callback) → void
    │
    └─ 转换方法
        ├─ toArray() → Array<JElement>
        └─ map(callback) → Array
```

---

## 4. 详细 API 规范

### 4.1 JDocument 类

#### 构造函数

```javascript
/**
 * 解析 HTML 文档
 * @param {string} html - HTML 字符串
 * @param {string} [baseUri] - 基础 URI,用于解析相对 URL
 */
const doc = new JDocument(html);
const doc = new JDocument(html, "https://example.com");
```

#### 静态方法

```javascript
/**
 * 解析 HTML 字符串
 */
JDocument.parse(html)
JDocument.parse(html, baseUri)

/**
 * 解析 HTML 片段
 */
JDocument.parseBodyFragment(html)
```

#### 实例方法

```javascript
// 查询方法
doc.select("div.content")          // CSS 选择器查询
doc.selectFirst("h1")               // 查询第一个匹配元素
doc.getElementById("header")        // 通过 ID 查询
doc.getElementsByTag("a")           // 通过标签名查询
doc.getElementsByClass("link")      // 通过类名查询

// 特殊元素访问
doc.head()                          // <head> 元素
doc.body()                          // <body> 元素
doc.title()                         // <title> 文本内容

// 文档属性
doc.location()                      // 获取 baseUri
doc.setLocation(uri)                // 设置 baseUri

// 全文本
doc.text()                          // 整个文档的文本
doc.html()                          // 整个文档的 HTML
```

---

### 4.2 JElement 类

#### 查询方法

```javascript
const element = doc.selectFirst("div.container");

// CSS 选择器查询 (相对于当前元素)
element.select("a.link")            // 所有子孙链接
element.selectFirst("img")          // 第一个图片
element.getElementById("id")        // 子孙元素中查找 ID
element.getElementsByTag("p")       // 所有段落
element.getElementsByClass("item")  // 所有 class="item"
```

#### 遍历方法

```javascript
// 父元素
element.parent()                    // 直接父元素
element.parents()                   // 所有祖先元素
element.parents("div")              // 匹配选择器的祖先

// 子元素
element.children()                  // 所有直接子元素
element.children("a")               // 匹配选择器的子元素
element.child(0)                    // 第一个子元素
element.firstElementChild()         // 第一个子元素
element.lastElementChild()          // 最后一个子元素

// 兄弟元素
element.nextElementSibling()        // 下一个兄弟
element.previousElementSibling()    // 上一个兄弟
element.siblingElements()           // 所有兄弟元素
```

#### 属性方法

```javascript
// 获取/设置属性
element.attr("href")                // 获取 href 属性
element.attr("href", "newUrl")      // 设置 href 属性 (返回 this)
element.hasAttr("src")              // 是否有 src 属性
element.removeAttr("disabled")      // 移除属性
element.attributes()                // 所有属性 {key: value}

// URL 处理
element.absUrl("href")              // 获取绝对 URL

// Class 操作
element.className()                 // class 属性字符串
element.classNames()                // class 数组
element.hasClass("active")          // 是否包含某个 class
element.addClass("new")             // 添加 class
element.removeClass("old")          // 移除 class
element.toggleClass("hidden")       // 切换 class

// 其他属性
element.id()                        // id 属性
element.tagName()                   // 标签名 (小写)
```

#### 文本/HTML 方法

```javascript
// 文本提取
element.text()                      // 所有后代文本 (含子元素)
element.ownText()                   // 仅当前元素的直接文本
element.wholeText()                 // 完整文本 (保留空格)
element.hasText()                   // 是否包含文本

// HTML 提取
element.html()                      // 内部 HTML
element.outerHtml()                 // 包含自身的 HTML

// 数据提取 (script/style 标签)
element.data()                      // <script> 或 <style> 的内容

// 设置内容
element.text("new text")            // 设置文本
element.html("<p>new html</p>")    // 设置 HTML
```

#### 判断方法

```javascript
element.is("div")                   // 是否匹配选择器
element.hasText()                   // 是否有文本内容
element.isEmpty()                   // 是否为空元素
```

#### 表单相关

```javascript
// 针对 <input>, <textarea>, <select> 等
element.val()                       // 获取表单值
element.val("new value")            // 设置表单值
```

---

### 4.3 JElements 类 (元素集合)

```javascript
const links = doc.select("a");

// 访问元素
links.size()                        // 元素数量
links.isEmpty()                     // 是否为空
links.get(0)                        // 通过索引获取
links.first()                       // 第一个元素
links.last()                        // 最后一个元素

// 批量查询
links.select("img")                 // 所有链接下的图片
links.filter("[href]")              // 过滤有 href 的元素

// 批量提取文本
links.text()                        // 合并所有文本 (空格分隔)
links.texts()                       // 文本数组

// 批量提取属性
links.attr("href")                  // 第一个元素的 href
links.eachAttr("href")              // 所有元素的 href 数组

// 批量 HTML
links.html()                        // 第一个元素的 HTML
links.outerHtml()                   // 所有元素的 outerHtml 拼接

// 遍历
links.forEach(function(elem, index) {
    console.log(elem.text());
});

// 转换
links.toArray()                     // 转为 JavaScript 数组
links.map(function(elem) {
    return elem.attr("href");
});
```

---

## 5. 使用示例

### 5.1 基础查询

```javascript
// 解析 HTML
const html = `
<html>
  <head><title>示例页面</title></head>
  <body>
    <div class="container">
      <h1 class="title">书籍列表</h1>
      <ul class="book-list">
        <li class="book">
          <a href="/book/1" class="link">《书名1》</a>
          <span class="author">作者1</span>
        </li>
        <li class="book">
          <a href="/book/2" class="link">《书名2》</a>
          <span class="author">作者2</span>
        </li>
      </ul>
    </div>
  </body>
</html>
`;

const doc = new JDocument(html, "https://example.com");

// 获取标题
const title = doc.selectFirst("h1.title").text();  // "书籍列表"

// 获取所有书名
const books = doc.select("li.book a.link");
const bookNames = books.texts();  // ["《书名1》", "《书名2》"]

// 获取所有链接
const links = books.eachAttr("href");  // ["/book/1", "/book/2"]

// 获取绝对 URL
const absUrl = doc.selectFirst("a.link").absUrl("href");
// "https://example.com/book/1"
```

### 5.2 书源场景示例

#### 搜索结果解析

```javascript
function parseSearchResults(html) {
    const doc = new JDocument(html);
    const results = [];

    doc.select("div.search-result").forEach(function(item) {
        results.push({
            name: item.selectFirst("h3.book-name").text(),
            author: item.selectFirst("span.author").text(),
            cover: item.selectFirst("img").absUrl("src"),
            url: item.selectFirst("a.detail-link").absUrl("href"),
            latestChapter: item.selectFirst("span.latest").text()
        });
    });

    return results;
}
```

#### 章节列表解析

```javascript
function parseChapterList(html) {
    const doc = new JDocument(html);
    const chapters = doc.select("ul.chapter-list li a");

    return chapters.map(function(a) {
        return {
            name: a.text(),
            url: a.absUrl("href")
        };
    });
}
```

#### 章节内容解析

```javascript
function parseChapterContent(html) {
    const doc = new JDocument(html);
    const content = doc.selectFirst("div.content");

    // 移除广告
    content.select("div.ad").forEach(function(ad) {
        ad.remove();
    });

    // 获取纯文本
    return content.text();
}
```

### 5.3 复杂遍历

```javascript
// 查找包含特定文本的元素的父元素
const parent = doc.select("span").filter(function(elem) {
    return elem.text().includes("关键词");
}).first().parent();

// 获取表格数据
const table = doc.selectFirst("table");
const rows = table.select("tr");

const data = rows.map(function(row) {
    const cells = row.select("td");
    return cells.texts();  // 每行的所有单元格文本
});
```

### 5.4 表单处理

```javascript
// 获取表单数据
const form = doc.selectFirst("form#login");
const username = form.selectFirst("input[name=username]").val();
const password = form.selectFirst("input[name=password]").val();

// 获取所有选中的 checkbox
const checked = form.select("input[type=checkbox]:checked");
const values = checked.eachAttr("value");
```

---

## 6. 技术实现细节

### 6.1 底层依赖

使用 Rust `scraper` crate (基于 html5ever):
```toml
[dependencies]
scraper = "0.22.0"  # ✅ 已有依赖
```

**scraper 核心类型**:
- `scraper::Html` - HTML 文档
- `scraper::Selector` - CSS 选择器
- `scraper::ElementRef` - 元素引用
- `scraper::element_ref::Select` - 选择器迭代器

### 6.2 数据结构设计

```rust
use scraper::{Html, Selector, ElementRef};
use boa_engine::{JsData, JsValue, Context, JsResult};
use boa_gc::{Trace, Finalize};

/// 文档对象
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct JDocument {
    // 存储原始 HTML 和解析后的文档
    html: String,
    base_uri: Option<String>,
    // 注意: Html 不能直接存储,因为不是 Trace/Finalize
    // 解决方案: 每次方法调用时重新解析,或使用 lazy 缓存
}

/// 元素对象
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct JElement {
    // 存储元素的 HTML 片段和上下文
    html: String,              // 元素的 outerHtml
    base_uri: Option<String>,

    // 元素信息 (避免重复解析)
    tag_name: String,
    id: Option<String>,
    class_names: Vec<String>,
    attributes: HashMap<String, String>,
}

/// 元素集合
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct JElements {
    elements: Vec<JElement>,
    base_uri: Option<String>,
}
```

### 6.3 关键实现挑战

#### 挑战 1: scraper::Html 不实现 Trace/Finalize

**问题**: `scraper::Html` 和 `scraper::ElementRef` 不实现 Boa 要求的 `Trace` 和 `Finalize` 特征。

**解决方案**:

**选项 A: 存储 HTML 字符串,每次重新解析**
```rust
impl JDocument {
    fn select(&self, selector: &str) -> JsResult<JElements> {
        let doc = Html::parse_document(&self.html);  // 重新解析
        let selector = Selector::parse(selector).unwrap();
        // ...
    }
}
```
- ✅ 简单实现
- ✅ 符合 Boa GC 要求
- ❌ 性能开销 (每次查询都重新解析)

**选项 B: 缓存解析结果 (使用 RefCell)**
```rust
use std::cell::RefCell;

#[derive(Debug, Trace, Finalize, JsData)]
pub struct JDocument {
    html: String,
    #[unsafe_ignore_trace]  // 告诉 GC 忽略此字段
    cached_doc: RefCell<Option<Html>>,
}
```
- ✅ 高性能 (仅解析一次)
- ⚠️ 需要使用 `unsafe_ignore_trace`
- ⚠️ 需要确保 GC 安全

**推荐**: **选项 B (缓存 + unsafe_ignore_trace)**,性能更好,风险可控。

#### 挑战 2: 元素引用生命周期

**问题**: `scraper::ElementRef<'a>` 有生命周期参数,无法直接存储在 `JElement` 中。

**解决方案**: 存储元素的必要信息 (HTML 片段、属性等),而非引用。

```rust
impl JElement {
    fn from_element_ref(elem: ElementRef<'_>, base_uri: Option<String>) -> Self {
        // 提取所有需要的信息
        Self {
            html: elem.html(),
            tag_name: elem.value().name().to_string(),
            id: elem.value().id().map(String::from),
            class_names: elem.value().classes().map(String::from).collect(),
            attributes: elem.value()
                .attrs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            base_uri,
        }
    }
}
```

#### 挑战 3: CSS 选择器性能

**优化**: 缓存编译后的 `Selector`

```rust
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref SELECTOR_CACHE: Mutex<HashMap<String, Selector>> =
        Mutex::new(HashMap::new());
}

fn get_selector(selector_str: &str) -> Result<Selector, String> {
    let mut cache = SELECTOR_CACHE.lock().unwrap();

    if let Some(selector) = cache.get(selector_str) {
        return Ok(selector.clone());
    }

    let selector = Selector::parse(selector_str)
        .map_err(|e| format!("Invalid selector: {:?}", e))?;

    cache.insert(selector_str.to_string(), selector.clone());
    Ok(selector)
}
```

---

## 7. 实现计划

### Phase 1: 核心基础 (1-2 天)

**任务**:
1. ✅ 创建 `src/scraper/jdocument.rs`
2. ✅ 实现 `JDocument` 类
   - 构造函数 `new(html, baseUri)`
   - `select(selector)` → `JElements`
   - `selectFirst(selector)` → `JElement | null`
3. ✅ 实现 `JElement` 类 (基础)
   - `text()`, `html()`, `outerHtml()`
   - `attr(name)`, `hasAttr(name)`
   - `tagName()`, `id()`, `className()`
4. ✅ 实现 `JElements` 类 (基础)
   - `size()`, `isEmpty()`, `get(index)`
   - `first()`, `last()`
   - `texts()`, `eachAttr(name)`

**测试**:
```javascript
const doc = new JDocument("<div class='test'>Hello</div>");
const elem = doc.selectFirst("div.test");
assert(elem.text() === "Hello");
assert(elem.attr("class") === "test");
```

### Phase 2: 遍历方法 (1 天)

**任务**:
1. `JElement` 遍历方法
   - `parent()`, `children()`, `child(index)`
   - `nextElementSibling()`, `previousElementSibling()`
   - `firstElementChild()`, `lastElementChild()`
2. `JElements` 批量遍历
   - `parents()`
   - `forEach(callback)`

**测试**:
```javascript
const doc = new JDocument("<div><p>A</p><p>B</p></div>");
const p = doc.selectFirst("p");
assert(p.parent().tagName() === "div");
assert(p.nextElementSibling().text() === "B");
```

### Phase 3: 高级查询 (1 天)

**任务**:
1. `getElementById(id)`
2. `getElementsByTag(tag)`
3. `getElementsByClass(className)`
4. `JElement.select()` (相对查询)
5. `JElements.select()`, `filter()`

### Phase 4: URL 处理 (0.5 天)

**任务**:
1. `JDocument.location()`, `setLocation()`
2. `JElement.absUrl(attrName)`

**依赖**: 使用已实现的 `URL` 类

### Phase 5: 文本优化 (0.5 天)

**任务**:
1. `JElement.ownText()` (仅直接文本)
2. `JElement.wholeText()` (保留空格)
3. `JElement.data()` (script/style 内容)

### Phase 6: 完善与优化 (1 天)

**任务**:
1. 添加选择器缓存
2. 性能基准测试
3. 完善错误处理
4. 文档和示例
5. 集成到 `BookCore`

---

## 8. 测试策略

### 8.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Context;

    #[test]
    fn test_basic_select() {
        let mut ctx = Context::default();
        register_jdocument(&mut ctx);

        let code = r#"
            const html = '<div class="test"><p>Hello</p></div>';
            const doc = new JDocument(html);
            const elem = doc.selectFirst("div.test p");
            elem.text();
        "#;

        let result = ctx.eval(Source::from_bytes(code)).unwrap();
        assert_eq!(result.to_string(&mut ctx).unwrap().to_std_string_escaped(), "Hello");
    }

    #[test]
    fn test_attr_extraction() {
        let mut ctx = Context::default();
        register_jdocument(&mut ctx);

        let code = r#"
            const html = '<a href="/path" class="link">Text</a>';
            const doc = new JDocument(html, "https://example.com");
            const link = doc.selectFirst("a");
            [
                link.attr("href"),
                link.absUrl("href"),
                link.className(),
                link.text()
            ];
        "#;

        let result = ctx.eval(Source::from_bytes(code)).unwrap();
        let arr = result.as_object().unwrap();
        // 验证结果...
    }
}
```

### 8.2 集成测试

创建真实书源场景测试:

```rust
#[test]
fn test_real_book_source_parsing() {
    let html = std::fs::read_to_string("tests/fixtures/search_result.html").unwrap();

    let mut ctx = Context::default();
    register_jdocument(&mut ctx);

    let code = format!(r#"
        const doc = new JDocument(`{}`);
        const books = doc.select("div.book-item");

        books.map(function(book) {{
            return {{
                name: book.selectFirst("h3.name").text(),
                author: book.selectFirst("span.author").text(),
                cover: book.selectFirst("img").attr("src")
            }};
        }});
    "#, html);

    let result = ctx.eval(Source::from_bytes(&code)).unwrap();
    // 验证解析结果...
}
```

---

## 9. API 对比表

| 功能 | Jsoup (Java) | BookCore (JavaScript) | 优先级 |
|------|-------------|----------------------|--------|
| **文档解析** |
| 解析 HTML | `Jsoup.parse(html)` | `new JDocument(html)` | P0 |
| 设置 baseUri | `Jsoup.parse(html, baseUri)` | `new JDocument(html, baseUri)` | P0 |
| **查询** |
| CSS 选择器 | `doc.select("div")` | `doc.select("div")` | P0 |
| 查询第一个 | `doc.selectFirst("a")` | `doc.selectFirst("a")` | P0 |
| 通过 ID | `doc.getElementById("id")` | `doc.getElementById("id")` | P1 |
| 通过标签 | `doc.getElementsByTag("p")` | `doc.getElementsByTag("p")` | P1 |
| 通过 Class | `doc.getElementsByClass("c")` | `doc.getElementsByClass("c")` | P1 |
| **遍历** |
| 父元素 | `elem.parent()` | `elem.parent()` | P0 |
| 子元素 | `elem.children()` | `elem.children()` | P0 |
| 兄弟元素 | `elem.nextElementSibling()` | `elem.nextElementSibling()` | P1 |
| **属性** |
| 获取属性 | `elem.attr("href")` | `elem.attr("href")` | P0 |
| 判断属性 | `elem.hasAttr("src")` | `elem.hasAttr("src")` | P1 |
| 绝对 URL | `elem.absUrl("href")` | `elem.absUrl("href")` | P0 |
| Class 操作 | `elem.hasClass("active")` | `elem.hasClass("active")` | P1 |
| **文本/HTML** |
| 获取文本 | `elem.text()` | `elem.text()` | P0 |
| 获取 HTML | `elem.html()` | `elem.html()` | P0 |
| 外部 HTML | `elem.outerHtml()` | `elem.outerHtml()` | P1 |
| 直接文本 | `elem.ownText()` | `elem.ownText()` | P2 |
| **集合操作** |
| 大小 | `elems.size()` | `elems.size()` | P0 |
| 获取元素 | `elems.get(0)` | `elems.get(0)` | P0 |
| 第一个 | `elems.first()` | `elems.first()` | P0 |
| 批量文本 | `elems.eachText()` | `elems.texts()` | P0 |
| 批量属性 | `elems.eachAttr("href")` | `elems.eachAttr("href")` | P0 |

**优先级说明**:
- **P0** (高): 书源核心功能,必须实现
- **P1** (中): 常用功能,应该实现
- **P2** (低): 辅助功能,可选实现

---

## 10. 性能优化建议

### 10.1 选择器缓存

```rust
// 全局选择器缓存,避免重复编译
lazy_static! {
    static ref SELECTOR_CACHE: Mutex<LruCache<String, Selector>> =
        Mutex::new(LruCache::new(100));
}
```

### 10.2 文档解析缓存

```rust
impl JDocument {
    // 仅解析一次,后续复用
    fn get_parsed_doc(&self) -> Html {
        let mut cache = self.cached_doc.borrow_mut();
        if cache.is_none() {
            *cache = Some(Html::parse_document(&self.html));
        }
        cache.as_ref().unwrap().clone()
    }
}
```

### 10.3 延迟属性提取

```rust
impl JElement {
    // 仅在首次访问时提取属性
    fn ensure_attributes(&mut self) {
        if self.attributes.is_none() {
            // 解析并缓存属性
        }
    }
}
```

---

## 11. 向后兼容性

**当前 `JScraper` 类**:
```javascript
const scraper = new JScraper(html);
const text = scraper.text();
```

**兼容策略**:
1. ✅ 保留 `JScraper` 类 (标记为 deprecated)
2. ✅ 添加 `#[deprecated]` 提示
3. ✅ 文档中引导用户迁移到 `JDocument`

**迁移示例**:
```javascript
// 旧 API
const scraper = new JScraper(html);
const text = scraper.text();

// 新 API (更强大)
const doc = new JDocument(html);
const text = doc.body().text();
```

---

## 12. 文档和示例

### 12.1 README 示例

```markdown
### DOM 解析 (Jsoup 风格)

BookCore 提供强大的 DOM 解析 API,灵感来自 Java Jsoup 库:

\`\`\`javascript
const html = await fetch("https://example.com/books");
const doc = new JDocument(html, "https://example.com");

// CSS 选择器查询
const books = doc.select("div.book-item");

// 提取数据
const bookList = books.map(function(book) {
    return {
        name: book.selectFirst("h3.title").text(),
        author: book.selectFirst("span.author").text(),
        cover: book.selectFirst("img").absUrl("src"),
        url: book.selectFirst("a").absUrl("href")
    };
});
\`\`\`

支持的功能:
- ✅ CSS 选择器 (`select`, `selectFirst`)
- ✅ DOM 遍历 (`parent`, `children`, `nextElementSibling`)
- ✅ 属性提取 (`attr`, `absUrl`, `className`)
- ✅ 文本提取 (`text`, `html`, `outerHtml`)
- ✅ 批量操作 (`texts`, `eachAttr`, `forEach`)
```

---

## 13. 成功标准

### 功能完整性
- ✅ 支持 Jsoup 80%+ 常用 API
- ✅ 通过 100+ 单元测试
- ✅ 通过 10+ 真实书源场景测试

### 性能目标
- ✅ 解析 1MB HTML < 100ms
- ✅ 复杂选择器查询 < 10ms
- ✅ 选择器缓存命中率 > 80%

### 用户体验
- ✅ API 直观易用 (类似 Jsoup/jQuery)
- ✅ 完整的错误提示
- ✅ 详细的文档和示例

---

## 14. 时间线

| 阶段 | 任务 | 预计时间 | 累计 |
|------|------|---------|------|
| Phase 1 | 核心基础 (JDocument, JElement, JElements) | 1-2 天 | 2 天 |
| Phase 2 | 遍历方法 | 1 天 | 3 天 |
| Phase 3 | 高级查询 | 1 天 | 4 天 |
| Phase 4 | URL 处理 | 0.5 天 | 4.5 天 |
| Phase 5 | 文本优化 | 0.5 天 | 5 天 |
| Phase 6 | 完善与优化 | 1 天 | 6 天 |

**总计**: 约 1 周 (6 个工作日)

---

## 15. 风险和挑战

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| `scraper` crate 不支持某些 CSS 选择器 | 中 | 文档中列出支持的选择器,提供替代方案 |
| GC 追踪问题 (`unsafe_ignore_trace`) | 低 | 仔细测试,确保无内存泄漏 |
| 性能不如原生 Jsoup | 低 | 添加缓存优化,满足书源场景需求 |
| API 设计与 Jsoup 差异 | 低 | 遵循 Jsoup 命名,JavaScript 风格调整 |

---

## 16. 后续扩展

### 短期 (完成后 1 个月)
- [ ] XPath 选择器支持
- [ ] CSS 伪类选择器 (`:nth-child`, `:not()` 等)
- [ ] 元素修改 API (`remove()`, `append()`, `prepend()`)

### 中期 (3-6 个月)
- [ ] JSON 解析器 (类似 Jsoup 的 JsonPath)
- [ ] XML 解析支持
- [ ] 流式解析 (处理大型 HTML)

### 长期 (6-12 个月)
- [ ] 浏览器兼容的 `querySelector` / `querySelectorAll`
- [ ] 完整的 DOM Level 3 支持

---

**文档状态**: Ready for Implementation
**审核者**: @zsakvo
**开始日期**: 2025-01-04
**目标完成**: 2025-01-10
