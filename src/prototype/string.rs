use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use boa_engine::{
    js_error, js_string, object::{builtins::JsArray, FunctionObjectBuilder}, property::{PropertyDescriptor, PropertyKey}, Context, JsArgs, JsError, JsValue, NativeFunction
};
use encoding_rs::GBK;
use md5::Md5;
use percent_encoding::percent_encode;
use sha1::Sha1;
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
        let mut hasher = Md5::new();
        hasher.update(this_str.to_std_string_escaped());
        let digest = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
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
 
fn register_to_sha(context: &mut Context) -> Result<bool, JsError> {
    let func = NativeFunction::from_fn_ptr(|this, args, context| {
        let this_str = this.to_string(context)?.to_std_string_escaped();
        let hash = args.get(0).unwrap().to_string(context)?.to_std_string_escaped();
        let to_string = args.get_or_undefined(1);
        // let mut hasher = Sha256::new();
        // hasher.update(this_str);
        // let result = hasher.finalize();
        let result = match hash.as_str() {
            "1" =>{
                let mut hasher = Sha1::new();
                hasher.update(this_str);
                hasher.finalize().to_vec()
            }
            "224" => {
                let mut hasher = Sha224::new();
                hasher.update(this_str);
                hasher.finalize().to_vec()
            }
            "256" => {
                let mut hasher = Sha256::new();
                hasher.update(this_str);
                hasher.finalize().to_vec()
            }
            "384" => {
                let mut hasher = Sha384::new();
                hasher.update(this_str);
                hasher.finalize().to_vec()
            }
            "512" => {
                let mut hasher = Sha512::new();
                hasher.update(this_str);
                hasher.finalize().to_vec()
            }
            _ => {
                return Err(js_error!("Unsupported hash algorithm"));
            }
        };
        if to_string.is_null_or_undefined() || !to_string.to_boolean() {
            Ok(JsValue::new(JsArray::from_iter(
                result.iter().map(|e| JsValue::Integer(*e as i32)),
                context,
            )))
        } else {
            let digest = result
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        Ok(JsValue::String(digest.into()))
        }
    });
    register(context, "toSha", func)
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
    register_to_sha(ctx).expect("Failed to register toSha512 function");
}
