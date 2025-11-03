//! URL 类
//!
//! 提供浏览器兼容的 URL 解析和操作 API

use boa_engine::{
    class::Class, js_error, js_string, Context, JsData, JsNativeError, JsResult, JsValue,
    NativeFunction,
};
use boa_gc::{Finalize, Trace};
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};

/// URL 编码的 ASCII 字符集
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
const PATH: &AsciiSet = &FRAGMENT.add(b'#').add(b'?').add(b'{').add(b'}');
const USERINFO: &AsciiSet = &PATH
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'|');

#[derive(Debug, Clone, Trace, Finalize, JsData)]
struct Url {
    href: String,
    protocol: String,
    host: String,
    hostname: String,
    port: String,
    pathname: String,
    search: String,
    hash: String,
    origin: String,
}

impl Url {
    /// 从字符串解析 URL
    fn parse(url_str: &str) -> Result<Self, String> {
        // 简化的 URL 解析实现
        let url_str = url_str.trim();

        // 解析协议
        let (protocol, rest) = if let Some(pos) = url_str.find("://") {
            let proto = &url_str[..pos];
            (format!("{}:", proto), &url_str[pos + 3..])
        } else {
            return Err("Invalid URL: missing protocol".to_string());
        };

        // 解析 hash (#fragment)
        let (rest, hash) = if let Some(pos) = rest.rfind('#') {
            (&rest[..pos], rest[pos..].to_string())
        } else {
            (rest, String::new())
        };

        // 解析 search (?query)
        let (rest, search) = if let Some(pos) = rest.find('?') {
            (&rest[..pos], rest[pos..].to_string())
        } else {
            (rest, String::new())
        };

        // 解析 host 和 pathname
        let (host_part, pathname) = if let Some(pos) = rest.find('/') {
            (&rest[..pos], rest[pos..].to_string())
        } else {
            (rest, "/".to_string())
        };

        // 解析 host 和 port
        let (hostname, port) = if let Some(pos) = host_part.rfind(':') {
            let h = &host_part[..pos];
            let p = &host_part[pos + 1..];
            (h.to_string(), p.to_string())
        } else {
            (host_part.to_string(), String::new())
        };

        let host = if port.is_empty() {
            hostname.clone()
        } else {
            format!("{}:{}", hostname, port)
        };

        // 构建 origin
        let origin = if port.is_empty() {
            format!("{}//{}", protocol, hostname)
        } else {
            format!("{}//{}", protocol, host)
        };

        // 构建完整的 href
        let href = format!(
            "{}//{}{}{}{}",
            protocol, host, pathname, search, hash
        );

        Ok(Url {
            href,
            protocol,
            host,
            hostname,
            port,
            pathname,
            search,
            hash,
            origin,
        })
    }

    /// 构造函数
    fn constructor(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<Self> {
        let url_str = args
            .get(0)
            .and_then(|v| v.as_string())
            .ok_or_else(|| js_error!("URL 构造函数需要一个字符串参数"))?
            .to_std_string_escaped();

        Self::parse(&url_str)
            .map_err(|e| JsNativeError::typ().with_message(e).into())
    }

    /// 获取 href
    fn get_href(_this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object")
        })?;
        let url = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a URL object")
        })?;
        Ok(JsValue::new(js_string!(url.href.clone())))
    }

    /// 获取 protocol
    fn get_protocol(_this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object")
        })?;
        let url = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a URL object")
        })?;
        Ok(JsValue::new(js_string!(url.protocol.clone())))
    }

    /// 获取 host
    fn get_host(_this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object")
        })?;
        let url = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a URL object")
        })?;
        Ok(JsValue::new(js_string!(url.host.clone())))
    }

    /// 获取 hostname
    fn get_hostname(_this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object")
        })?;
        let url = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a URL object")
        })?;
        Ok(JsValue::new(js_string!(url.hostname.clone())))
    }

    /// 获取 port
    fn get_port(_this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object")
        })?;
        let url = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a URL object")
        })?;
        Ok(JsValue::new(js_string!(url.port.clone())))
    }

    /// 获取 pathname
    fn get_pathname(_this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object")
        })?;
        let url = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a URL object")
        })?;
        Ok(JsValue::new(js_string!(url.pathname.clone())))
    }

    /// 获取 search
    fn get_search(_this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object")
        })?;
        let url = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a URL object")
        })?;
        Ok(JsValue::new(js_string!(url.search.clone())))
    }

    /// 获取 hash
    fn get_hash(_this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object")
        })?;
        let url = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a URL object")
        })?;
        Ok(JsValue::new(js_string!(url.hash.clone())))
    }

    /// 获取 origin
    fn get_origin(_this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object")
        })?;
        let url = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a URL object")
        })?;
        Ok(JsValue::new(js_string!(url.origin.clone())))
    }

    /// toString 方法
    fn to_string(_this: &JsValue, _args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        Self::get_href(_this, _args, ctx)
    }
}

impl Class for Url {
    const NAME: &'static str = "URL";
    const LENGTH: usize = 1;

    fn init(class: &mut boa_engine::class::ClassBuilder<'_>) -> JsResult<()> {
        class
            .accessor(
                js_string!("href"),
                Some(NativeFunction::from_fn_ptr(Self::get_href)),
                None,
            )
            .accessor(
                js_string!("protocol"),
                Some(NativeFunction::from_fn_ptr(Self::get_protocol)),
                None,
            )
            .accessor(
                js_string!("host"),
                Some(NativeFunction::from_fn_ptr(Self::get_host)),
                None,
            )
            .accessor(
                js_string!("hostname"),
                Some(NativeFunction::from_fn_ptr(Self::get_hostname)),
                None,
            )
            .accessor(
                js_string!("port"),
                Some(NativeFunction::from_fn_ptr(Self::get_port)),
                None,
            )
            .accessor(
                js_string!("pathname"),
                Some(NativeFunction::from_fn_ptr(Self::get_pathname)),
                None,
            )
            .accessor(
                js_string!("search"),
                Some(NativeFunction::from_fn_ptr(Self::get_search)),
                None,
            )
            .accessor(
                js_string!("hash"),
                Some(NativeFunction::from_fn_ptr(Self::get_hash)),
                None,
            )
            .accessor(
                js_string!("origin"),
                Some(NativeFunction::from_fn_ptr(Self::get_origin)),
                None,
            )
            .method(
                js_string!("toString"),
                0,
                NativeFunction::from_fn_ptr(Self::to_string),
            );
        Ok(())
    }

    fn data_constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Self> {
        Self::constructor(new_target, args, context)
    }
}

/// 注册 URL 类到 JavaScript 上下文
pub fn register_url(context: &mut Context) {
    context
        .register_global_class::<Url>()
        .expect("the URL class shouldn't exist");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parse() {
        let mut context = Context::default();
        register_url(&mut context);

        let code = r#"
            const url = new URL("https://example.com:8080/path/to/page?query=value#fragment");
            url.href;
        "#;

        let result = context.eval(boa_engine::Source::from_bytes(code));
        assert!(result.is_ok());
        let href = result.unwrap().to_string(&mut context).unwrap();
        assert!(href.to_std_string_escaped().contains("example.com"));
    }

    #[test]
    fn test_url_properties() {
        let mut context = Context::default();
        register_url(&mut context);

        let code = r#"
            const url = new URL("https://example.com:8080/path?query=value#hash");
            [
                url.protocol === "https:",
                url.hostname === "example.com",
                url.port === "8080",
                url.pathname === "/path",
                url.search === "?query=value",
                url.hash === "#hash"
            ].every(x => x);
        "#;

        let result = context.eval(boa_engine::Source::from_bytes(code));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_boolean(), Some(true));
    }

    #[test]
    fn test_url_origin() {
        let mut context = Context::default();
        register_url(&mut context);

        let code = r#"
            const url = new URL("https://example.com:8080/path");
            url.origin;
        "#;

        let result = context.eval(boa_engine::Source::from_bytes(code));
        assert!(result.is_ok());
        let origin = result.unwrap().to_string(&mut context).unwrap();
        assert_eq!(
            origin.to_std_string_escaped(),
            "https://example.com:8080"
        );
    }

    #[test]
    fn test_url_to_string() {
        let mut context = Context::default();
        register_url(&mut context);

        let code = r#"
            const url = new URL("https://example.com/path");
            url.toString();
        "#;

        let result = context.eval(boa_engine::Source::from_bytes(code));
        assert!(result.is_ok());
    }
}
