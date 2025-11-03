//! 浏览器编码 API
//!
//! 提供 atob 和 btoa 函数，与浏览器环境兼容

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use boa_engine::{js_error, js_string, Context, JsArgs, JsValue, NativeFunction};

/// 注册 atob 函数到 JavaScript 全局对象
///
/// 解码 Base64 编码的字符串
///
/// # JavaScript 示例
///
/// ```javascript
/// const decoded = atob("SGVsbG8sIFdvcmxkIQ==");
/// console.log(decoded); // "Hello, World!"
/// ```
pub fn register_atob(ctx: &mut Context) {
    let function = NativeFunction::from_fn_ptr(|_this, args, context| {
        let input = args.get_or_undefined(0);

        if input.is_null_or_undefined() {
            return Err(js_error!("atob: Argument is required"));
        }

        let input_str = input.to_string(context)?.to_std_string_escaped();

        // 解码 Base64
        let decoded_bytes = BASE64
            .decode(input_str.trim())
            .map_err(|e| js_error!("atob: Invalid base64 string: {}", e))?;

        // 转换为字符串（latin1 编码，与浏览器行为一致）
        let result: String = decoded_bytes.iter().map(|&b| b as char).collect();

        Ok(JsValue::new(js_string!(result)))
    });

    ctx.register_global_builtin_callable(js_string!("atob"), 1, function)
        .expect("Failed to register atob");
}

/// 注册 btoa 函数到 JavaScript 全局对象
///
/// 将字符串编码为 Base64
///
/// # JavaScript 示例
///
/// ```javascript
/// const encoded = btoa("Hello, World!");
/// console.log(encoded); // "SGVsbG8sIFdvcmxkIQ=="
/// ```
pub fn register_btoa(ctx: &mut Context) {
    let function = NativeFunction::from_fn_ptr(|_this, args, context| {
        let input = args.get_or_undefined(0);

        if input.is_null_or_undefined() {
            return Err(js_error!("btoa: Argument is required"));
        }

        let input_str = input.to_string(context)?.to_std_string_escaped();

        // 检查是否包含非 latin1 字符
        if input_str.chars().any(|c| c as u32 > 0xFF) {
            return Err(js_error!(
                "btoa: String contains characters outside the Latin1 range"
            ));
        }

        // 转换为字节（latin1 编码）
        let bytes: Vec<u8> = input_str.chars().map(|c| c as u8).collect();

        // 编码为 Base64
        let encoded = BASE64.encode(&bytes);

        Ok(JsValue::new(js_string!(encoded)))
    });

    ctx.register_global_builtin_callable(js_string!("btoa"), 1, function)
        .expect("Failed to register btoa");
}

/// 注册所有编码 API
pub fn register_encoding_apis(ctx: &mut Context) {
    register_atob(ctx);
    register_btoa(ctx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Context;

    #[test]
    fn test_atob_btoa() {
        let mut context = Context::default();
        register_encoding_apis(&mut context);

        // 测试 btoa
        let result = context
            .eval(boa_engine::Source::from_bytes(
                r#"btoa("Hello, World!")"#,
            ))
            .unwrap();
        let encoded = result.to_string(&mut context).unwrap().to_std_string_escaped();
        assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");

        // 测试 atob
        let result = context
            .eval(boa_engine::Source::from_bytes(
                r#"atob("SGVsbG8sIFdvcmxkIQ==")"#,
            ))
            .unwrap();
        let decoded = result.to_string(&mut context).unwrap().to_std_string_escaped();
        assert_eq!(decoded, "Hello, World!");
    }

    #[test]
    fn test_atob_invalid() {
        let mut context = Context::default();
        register_atob(&mut context);

        let result = context.eval(boa_engine::Source::from_bytes(r#"atob("Invalid!")"#));
        assert!(result.is_err());
    }
}
