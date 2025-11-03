use boa_engine::{js_string, Context, JsValue, NativeFunction};
use rand::Rng;

/// 注册 randString 函数到 JavaScript 全局对象
///
/// 生成指定长度的随机十六进制字符串
pub fn register_rand_str(ctx: &mut Context) {
    let function = NativeFunction::from_fn_ptr(|_this, _args, _context| {
        let length = _args.get(0).unwrap().as_number().unwrap() as usize;
        let mut rng = rand::rng();
        let bytes: Vec<u8> = (0..length).map(|_| rng.random_range(0..255)).collect();
        Ok(JsValue::new(js_string!(hex::encode(bytes))))
    });
    ctx.register_global_builtin_callable(js_string!("randString"), 1, function)
        .expect("Failed to register randString");
}

// 向后兼容的别名
#[deprecated(since = "0.2.0", note = "Use `register_rand_str` instead")]
#[allow(dead_code)]
pub fn regist_rand_str(ctx: &mut Context) {
    register_rand_str(ctx);
}
