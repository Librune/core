use boa_engine::{js_string, Context, JsArgs, JsValue, NativeFunction};
use uuid::Uuid;

pub fn regist_uuid(ctx: &mut Context) {
    let function = NativeFunction::from_fn_ptr(|_this, _args, _context| {
        let uuid = Uuid::new_v4();
        Ok(JsValue::String(js_string!(uuid.to_string())))
    });
    ctx.register_global_builtin_callable(js_string!("uuid"), 0, function)
        .expect("Failed to register uuid");
}

pub fn regist_is_uuid(ctx: &mut Context) {
    let function = NativeFunction::from_fn_ptr(|_this, args, _context| {
        let uuid_str = args.get_or_undefined(0);
        if uuid_str.is_null_or_undefined() {
            return Ok(JsValue::Boolean(false));
        }
        let uuid_str = uuid_str
            .to_string(_context)
            .unwrap()
            .to_std_string_escaped();
        let is_valid = Uuid::parse_str(&uuid_str).is_ok();
        Ok(JsValue::Boolean(is_valid))
    });
    ctx.register_global_builtin_callable(js_string!("isUuid"), 1, function)
        .expect("Failed to register isUuid");
}
