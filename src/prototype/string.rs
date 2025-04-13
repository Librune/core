use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use boa_engine::{
    js_string,
    object::FunctionObjectBuilder,
    property::{PropertyDescriptor, PropertyKey},
    Context, JsError, JsValue, NativeFunction,
};
use encoding_rs::GBK;
use percent_encoding::percent_encode;

fn register(ctx: &mut Context, name: &str, func: NativeFunction) -> Result<bool, JsError> {
    let string_proto = ctx.intrinsics().constructors().string().prototype();
    let func = FunctionObjectBuilder::new(ctx.realm(), func).build();
    string_proto.define_property_or_throw(
        PropertyKey::from(js_string!(name)),
        PropertyDescriptor::builder()
            .value(func)
            .writable(true)
            .enumerable(false)
            .configurable(true),
        ctx,
    )
}

fn register_to_gbk(ctx: &mut Context) -> Result<bool, JsError> {
    let func = NativeFunction::from_fn_ptr(|this, _args, context| {
        // 将调用对象转换为字符串
        let this_str = this.to_string(context)?.to_std_string_escaped();
        let (bytes, _encoding_used, _had_errors) = GBK.encode(this_str.as_str());
        let encoded_param = percent_encode(&bytes, percent_encoding::NON_ALPHANUMERIC).to_string();
        Ok(JsValue::String(encoded_param.into()))
    });
    register(ctx, "toGbk", func)
}

fn register_to_base64(context: &mut Context) -> Result<bool, JsError> {
    let func = NativeFunction::from_fn_ptr(|this, _args, context| {
        // 将调用对象转换为字符串
        let this_str = this.to_string(context)?.to_std_string_escaped();
        // 使用 base64 编码
        let result = BASE64.encode(this_str.as_bytes());
        Ok(JsValue::String(result.into()))
    });
    register(context, "toBase64", func)
}

fn register_to_md5(context: &mut Context) -> Result<bool, JsError> {
    let func = NativeFunction::from_fn_ptr(|this, _args, context| {
        // 将调用对象转换为字符串
        let this_str = this.to_string(context)?;
        // 使用 md5 包计算 MD5 值，并格式化为 16 进制字符串
        let digest = format!("{:x}", md5::compute(this_str.to_std_string_escaped()));
        Ok(JsValue::String(digest.into()))
    });
    register(context, "toMd5", func)
}

pub fn extend_string(ctx: &mut Context) {
    // Register the toQuery function to the String prototype
    // regist_to_query(ctx).expect("Failed to register toQuery function");
    // Register the toGBK function to the String prototype
    register_to_gbk(ctx).expect("Failed to register toGBK function");
    // Register the toBase64 function to the String prototype
    register_to_base64(ctx).expect("Failed to register toBase64 function");
    // Register the toMD5 function to the String prototype
    register_to_md5(ctx).expect("Failed to register toMD5 function");
}
