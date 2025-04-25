use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use boa_engine::{
    js_string,
    object::{builtins::JsArray, FunctionObjectBuilder},
    property::{PropertyDescriptor, PropertyKey},
    Context, JsError, JsValue, NativeFunction,
};
use encoding_rs::GBK; 
use percent_encoding::percent_encode;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
 

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

fn register_to_ascii(context: &mut Context) -> Result<bool, JsError> {
    let func = NativeFunction::from_fn_ptr(|this, _args, context| {
        // 将调用对象转换为字符串
        let this_str = this.to_string(context)?.to_std_string_escaped();
        let digest = this_str.as_str().bytes().collect::<Vec<u8>>();
        Ok(JsValue::from(JsArray::from_iter(
            digest.iter().map(|b| JsValue::Integer(*b as i32)),
            context,
        )))
    });
    register(context, "toAscii", func)
}

fn register_to_hex(context: &mut Context) -> Result<bool, JsError> {
    let func = NativeFunction::from_fn_ptr(|this, _args, context| {
        // 将调用对象转换为字符串
        let this_str = this.to_string(context)?.to_std_string_escaped();
        // 使用 md5 包计算 MD5 值，并格式化为 16 进制字符串
        let digest = format!("{:x}", md5::compute(this_str));
        Ok(JsValue::String(digest.into()))
    });
    register(context, "toHex", func)
}

fn register_to_sha224(context: &mut Context) -> Result<bool, JsError> {
    let func = NativeFunction::from_fn_ptr(|this, _args, context| {
        let this_str = this.to_string(context)?.to_std_string_escaped();
        let mut hasher = Sha224::new();
        hasher.update(this_str);
        let result = hasher.finalize();
        let digest = result
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        Ok(JsValue::String(digest.into()))
    });
    register(context, "toSha224", func)
}

fn register_to_sha256(context: &mut Context) -> Result<bool, JsError> {
    let func = NativeFunction::from_fn_ptr(|this, _args, context| {
        let this_str = this.to_string(context)?.to_std_string_escaped();
        let mut hasher = Sha256::new();
        hasher.update(this_str);
        let result = hasher.finalize();
        let digest = result
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        Ok(JsValue::String(digest.into()))
    });
    register(context, "toSha256", func)
}

fn register_to_sha384(context: &mut Context) -> Result<bool, JsError> {
    let func = NativeFunction::from_fn_ptr(|this, _args, context| {
        let this_str = this.to_string(context)?.to_std_string_escaped();
        let mut hasher = Sha384::new();
        hasher.update(this_str);
        let result = hasher.finalize();
        let digest = result
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        Ok(JsValue::String(digest.into()))
    });
    register(context, "toSha384", func)
}

fn register_to_sha512(context: &mut Context) -> Result<bool, JsError> {
    let func = NativeFunction::from_fn_ptr(|this, _args, context| {
        let this_str = this.to_string(context)?.to_std_string_escaped();
        let mut hasher = Sha512::new();
        hasher.update(this_str);
        let result = hasher.finalize();
        let digest = result
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        Ok(JsValue::String(digest.into()))
    });
    register(context, "toSha512", func)
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
    // Register the toHex function to the String prototype
    register_to_ascii(ctx).expect("Failed to register toAscii function");
    register_to_hex(ctx).expect("Failed to register toHex function");
    register_to_sha224(ctx).expect("Failed to register toSha224 function");
    register_to_sha256(ctx).expect("Failed to register toSha256 function");
    register_to_sha384(ctx).expect("Failed to register toSha384 function");
    register_to_sha512(ctx).expect("Failed to register toSha512 function");
}
