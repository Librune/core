use boa_engine::{js_string, object::ObjectInitializer, Context, JsObject, JsResult, JsValue};
use chardet::detect;
use encoding_rs::{Encoding, UTF_8};
use regex::Regex;
use reqwest::Response;

pub async fn decode_response(response: Response, ctx: &mut Context) -> JsResult<JsObject> {
    let obj = ObjectInitializer::new(ctx).build();
    obj.set(js_string!("ok"), response.status().is_success(), true, ctx)?;
    obj.set(js_string!("status"), response.status().as_u16(), true, ctx)?;
    obj.set(
        js_string!("statusText"),
        js_string!(response.status().canonical_reason().unwrap_or("")),
        true,
        ctx,
    )?;
    let headers = response.headers().clone();
    let headers_obj = ObjectInitializer::new(ctx).build();
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            headers_obj.set(js_string!(name.as_str()), js_string!(value_str), true, ctx)?;
        }
    }
    // 1. 首先尝试从 Content-Type 头获取编码
    let mut encoding = headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .and_then(|content_type| {
            content_type
                .split(';')
                .find(|part| part.trim().starts_with("charset="))
                .and_then(|charset| {
                    let charset = charset.trim()[8..].trim();
                    Encoding::for_label(charset.as_bytes())
                })
        });
    // 2. 如果没有在 header 中找到编码，尝试从 meta 标签获取
    let bytes = response.bytes().await.unwrap_or_default();
    if encoding.is_none() {
        encoding = extract_charset_from_meta(&bytes);
    }
    // 3. 如果还是没找到，使用 chardet 进行检测
    let encoding = encoding
        .or_else(|| detect_encoding(&bytes))
        .unwrap_or(UTF_8); // 如果都失败了，默认使用 UTF8

    // 解码内容
    let (text, _encoding_used, had_errors) = encoding.decode(&bytes);

    if had_errors {
        println!(
            "Warning: Some characters couldn't be decoded properly using {:?}",
            encoding.name()
        );
    }

    obj.set(
        js_string!("body"),
        JsValue::new(js_string!(text.into_owned())),
        true,
        ctx,
    )?;

    obj.set(
        js_string!("headers"),
        JsValue::Object(headers_obj),
        true,
        ctx,
    )?;

    Ok(obj)
}

/// 从 HTML meta 标签中提取字符集
fn extract_charset_from_meta(html_content: &[u8]) -> Option<&'static Encoding> {
    // 先尝试用 UTF-8 解码前 1000 个字节来查找 meta 标签
    // 这通常足够找到 head 部分了
    let head_content = String::from_utf8_lossy(&html_content[..html_content.len().min(1000)]);

    // 匹配 <meta> 标签中的字符集
    // 支持以下格式:
    // <meta charset="gbk">
    // <meta http-equiv="Content-Type" content="text/html; charset=gbk">
    let charset_pattern = Regex::new(r#"(?i)<meta[^>]+charset=[\s'"]*([^\s'">;]+)"#).unwrap();
    let http_equiv_pattern = Regex::new(r#"(?i)<meta[^>]+http-equiv=[\s'"]*content-type[\s'"]*[^>]+content=[\s'"]*[^;]+;\s*charset=[\s'"]*([^\s'">;]+)"#).unwrap();

    // 优先检查 charset 属性
    if let Some(cap) = charset_pattern.captures(&head_content) {
        if let Some(charset) = cap.get(1) {
            if let Some(encoding) = Encoding::for_label(charset.as_str().as_bytes()) {
                return Some(encoding);
            }
        }
    }

    // 然后检查 http-equiv
    if let Some(cap) = http_equiv_pattern.captures(&head_content) {
        if let Some(charset) = cap.get(1) {
            if let Some(encoding) = Encoding::for_label(charset.as_str().as_bytes()) {
                return Some(encoding);
            }
        }
    }

    None
}

/// 使用 chardet 检测编码
fn detect_encoding(content: &[u8]) -> Option<&'static Encoding> {
    let (charset, confidence, _language) = detect(content);

    // 只有当检测可信度超过 0.6 时才采用检测结果
    if confidence < 0.6 {
        return None;
    }

    // chardet 返回的编码名称转换为 encoding_rs 支持的编码
    match charset.to_ascii_lowercase().as_str() {
        "utf-8" => Some(encoding_rs::UTF_8),
        "gb2312" | "gbk" | "gb18030" => Some(encoding_rs::GB18030),
        "big5" => Some(encoding_rs::BIG5),
        "euc-jp" => Some(encoding_rs::EUC_JP),
        "euc-kr" => Some(encoding_rs::EUC_KR),
        "shift-jis" | "shift_jis" => Some(encoding_rs::SHIFT_JIS),
        "windows-1252" | "ascii" => Some(encoding_rs::WINDOWS_1252),
        _ => None,
    }
}
