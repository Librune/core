use std::{str::FromStr, time::Duration};

use boa_engine::{js_string, Context, JsResult, JsValue};
use reqwest::header::{HeaderMap, HeaderName};
use serde_json::Value;

#[derive(Debug)]
pub struct Options {
    pub headers: HeaderMap,
    pub timeout: Duration,
    pub body: Option<String>,
    pub query: Option<Value>,
    pub form: Option<Value>,
    pub json: Option<Value>,
    pub gbk: bool,
}

// 设置 options 的默认值
impl Default for Options {
    fn default() -> Self {
        Options {
            headers: HeaderMap::new(),
            timeout: Duration::from_secs(5),
            body: None,
            query: None,
            form: None,
            json: None,
            gbk: false,
        }
    }
}

impl Options {
    pub fn from_js_value(value: &JsValue, ctx: &mut Context) -> JsResult<Self> {
        let obj = value.as_object().unwrap();
        // 生成 headers
        let mut headers = HeaderMap::new();
        let headers_value = obj.get(js_string!("headers"), ctx)?;
        if !headers_value.is_null_or_undefined() {
            let js_headers = headers_value.as_object().unwrap();
            for key in js_headers.own_property_keys(ctx)? {
                let value = js_headers
                    .get(key.clone(), ctx)?
                    .to_string(ctx)?
                    .to_std_string_escaped();
                if let Ok(key) = HeaderName::from_str(&key.to_string()) {
                    headers.insert(key, value.parse().unwrap());
                }
            }
        }
        // 处理编码
        let mut gbk = false;
        let gbk_value = obj.get(js_string!("gbk"), ctx)?;
        if gbk_value.is_boolean() {
            gbk = gbk_value.as_boolean().unwrap();
        }
        // 生成超时
        let mut timeout = Duration::from_secs(5);
        let timeout_value = obj.get(js_string!("timeout"), ctx)?;
        if timeout_value.is_number() {
            timeout = Duration::from_secs(timeout_value.as_number().unwrap() as u64);
        }
        // 生成请求body
        let mut body = None;
        let body_value = obj.get(js_string!("body"), ctx)?;
        if !body_value.is_null_or_undefined() {
            body = body_value.to_string(ctx)?.to_std_string().ok();
        }
        // 生成请求json
        let mut json = None;
        let json_value = obj.get(js_string!("json"), ctx)?;
        if !json_value.is_null_or_undefined() {
            json = json_value.to_json(ctx).ok();
        }
        // 生成请求form
        let mut form = None;
        let form_value = obj.get(js_string!("form"), ctx)?;
        if !form_value.is_null_or_undefined() {
            form = form_value.to_json(ctx).ok();
        }
        // 生成请求query
        let mut query = None;
        let query_value = obj.get(js_string!("query"), ctx)?;
        if !query_value.is_null_or_undefined() {
            query = query_value.to_json(ctx).ok();
        }

        Ok(Options {
            headers: headers,
            timeout: timeout,
            body: body,
            query: query,
            form: form,
            json: json,
            gbk: gbk,
        })
    }
}
