use boa_engine::{js_string, Context, JsArgs, JsValue, NativeFunction};
use uuid::Uuid;

/// 注册 uuid 函数到 JavaScript 全局对象
///
/// 生成一个 UUID v4 字符串
pub fn register_uuid(ctx: &mut Context) {
    let function = NativeFunction::from_fn_ptr(|_this, _args, _context| {
        let uuid = Uuid::new_v4();
        Ok(JsValue::new(js_string!(uuid.to_string())))
    });
    ctx.register_global_builtin_callable(js_string!("uuid"), 0, function)
        .expect("Failed to register uuid");
}

/// 注册 isUuid 函数到 JavaScript 全局对象
///
/// 验证一个字符串是否为有效的 UUID
pub fn register_is_uuid(ctx: &mut Context) {
    let function = NativeFunction::from_fn_ptr(|_this, args, _context| {
        let uuid_str = args.get_or_undefined(0);
        if uuid_str.is_null_or_undefined() {
            return Ok(JsValue::new(false));
        }
        let uuid_str = uuid_str
            .to_string(_context)
            .unwrap()
            .to_std_string_escaped();
        let is_valid = Uuid::parse_str(&uuid_str).is_ok();
        Ok(JsValue::new(is_valid))
    });
    ctx.register_global_builtin_callable(js_string!("isUuid"), 1, function)
        .expect("Failed to register isUuid");
}

// 向后兼容的别名
#[deprecated(since = "0.2.0", note = "Use `register_uuid` instead")]
pub fn regist_uuid(ctx: &mut Context) {
    register_uuid(ctx);
}

#[deprecated(since = "0.2.0", note = "Use `register_is_uuid` instead")]
pub fn regist_is_uuid(ctx: &mut Context) {
    register_is_uuid(ctx);
}
