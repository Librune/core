use boa_engine::{js_string, Context, JsValue, NativeFunction};
use rand::Rng;

pub fn regist_rand_str(ctx: &mut Context) {
    let function = NativeFunction::from_fn_ptr(|_this, _args, _context| {
        let length = _args.get(0).unwrap().as_number().unwrap() as usize;
        let mut rng = rand::rng();
        let bytes: Vec<u8> = (0..length).map(|_| rng.random_range(0..255)).collect();
        Ok(JsValue::String(js_string!(hex::encode(bytes))))
    });
    ctx.register_global_builtin_callable(js_string!("randString"), 1, function)
        .expect("Failed to register randString");
}
