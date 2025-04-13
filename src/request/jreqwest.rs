use std::time::Duration;

use boa_engine::{
    class::Class, js_string, Context, JsData, JsNativeError, JsResult, JsValue, NativeFunction,
};
use boa_gc::{Finalize, Trace};
use reqwest::{Client, Method};

use super::{charset::decode_response, options::Options};

#[derive(Debug, Trace, Finalize, JsData)]
struct JReqwest {}

impl JReqwest {
    fn request(method: Method, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let mut url = args.get(0).unwrap().to_string(ctx)?.to_std_string_escaped();
        let options = args.get(1).unwrap();
        let options = Options::from_js_value(options, ctx).unwrap_or_default();
        let gbk = options.gbk;
        if let Some(query) = options.query.clone() {
            if gbk {
                let mut query_vec = Vec::new();
                for (key, value) in query.as_object().unwrap().iter() {
                    query_vec.push(format!("{}={}", key, value));
                }
                let query_str = query_vec.join("&");
                let (encoded, _, _) = encoding_rs::GBK.encode(&query_str);
                let encoded_query = String::from_utf8_lossy(&encoded);
                if url.contains('?') {
                    url = format!("{}&{}", url, encoded_query)
                } else {
                    url = format!("{}?{}", url, encoded_query)
                }
            }
        }
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .use_rustls_tls()
            .timeout(Duration::from_secs(options.timeout.as_secs()))
            .build()
            .expect("Failed to create reqwest client");
        let mut request = client.request(method, url);
        if !options.headers.is_empty() {
            request = request.headers(options.headers);
        }
        if let Some(json) = options.json {
            if gbk {
                let json_str = serde_json::to_string(&json).map_err(|e| {
                    JsNativeError::typ().with_message(format!("Failed to serialize JSON: {}", e))
                })?;
                let (encoded, _, _) = encoding_rs::GB18030.encode(&json_str);
                request = request
                    .body(encoded.into_owned())
                    .header("Content-Type", "application/json; charset=gbk");
            } else {
                request = request.json(&json);
            }
        }
        if let Some(body) = options.body {
            // request = request.body(body);
            if gbk {
                let (encoded, _, _) = encoding_rs::GB18030.encode(&body);
                request = request
                    .body(encoded.into_owned())
                    .header("Content-Type", "text/plain; charset=gbk");
            } else {
                request = request.body(body);
            }
        }
        if let Some(query) = options.query {
            if !gbk {
                request = request.query(&query);
            }
        }
        if let Some(form) = options.form {
            // request = request.form(&form);
            if gbk {
                let form_obj = form.as_object().ok_or("form 应为对象").map_err(|e| {
                    JsNativeError::typ().with_message(format!("form 应为对象: {}", e))
                })?;
                let mut form_vec = Vec::new();
                for (key, value) in form_obj {
                    form_vec.push(format!("{}={}", key, value.to_string()));
                }
                let form_str = form_vec.join("&");
                let (encoded, _, _) = encoding_rs::GB18030.encode(&form_str);
                request = request.body(encoded.into_owned()).header(
                    "Content-Type",
                    "application/x-www-form-urlencoded; charset=gbk",
                );
            } else {
                request = request.form(&form);
            }
        }

        let response = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                request.send().await.map_err(|e| {
                    JsNativeError::typ().with_message(format!("Request failed: {}", e))
                })
            })
        })?;

        let response = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                decode_response(response, ctx).await.map_err(|e| {
                    JsNativeError::typ().with_message(format!("Failed to decode response: {}", e))
                })
            })
        })?;

        Ok(response.into())
    }
    fn get(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        JReqwest::request(Method::GET, args, ctx)
    }
    fn post(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        JReqwest::request(Method::POST, args, ctx)
    }
    fn put(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        JReqwest::request(Method::PUT, args, ctx)
    }
    fn delete(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        JReqwest::request(Method::DELETE, args, ctx)
    }
}

impl Class for JReqwest {
    const NAME: &'static str = "JReqwest";
    const LENGTH: usize = 0;

    fn init(class: &mut boa_engine::class::ClassBuilder<'_>) -> boa_engine::JsResult<()> {
        class
            .static_method(js_string!("get"), 2, NativeFunction::from_fn_ptr(Self::get))
            .static_method(
                js_string!("post"),
                2,
                NativeFunction::from_fn_ptr(Self::post),
            )
            .static_method(js_string!("put"), 2, NativeFunction::from_fn_ptr(Self::put))
            .static_method(
                js_string!("delete"),
                2,
                NativeFunction::from_fn_ptr(Self::delete),
            );
        Ok(())
    }

    fn data_constructor(
        _new_target: &boa_engine::JsValue,
        _args: &[boa_engine::JsValue],
        _context: &mut Context,
    ) -> boa_engine::JsResult<Self> {
        Ok(JReqwest {})
    }
}

pub fn define_request(context: &mut Context) {
    context
        .register_global_class::<JReqwest>()
        .expect("the JReqwest builtin shouldn't exist");
}
