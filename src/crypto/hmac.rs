use base64::prelude::BASE64_STANDARD;
use base64::prelude::*;
use boa_engine::{
    class::Class, js_error, js_string, Context, JsData, JsNativeError, JsObject, JsResult, JsValue,
    NativeFunction,
};
use boa_gc::{Finalize, Trace};
use hmac::Mac;
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

type Hmac384 = hmac::Hmac<Sha384>;
type HmacSha256 = hmac::Hmac<Sha256>;
type Hmac512 = hmac::Hmac<Sha512>;
type HmacSha1 = hmac::Hmac<Sha1>;

#[derive(Debug, Trace, Finalize, JsData)]
enum HmacHash {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

#[derive(Debug, Trace, Finalize, JsData)]
enum HmacEncoding {
    Base64,
    Hex,
}

#[derive(Debug, Trace, Finalize, JsData)]
struct Hmac {
    hash: HmacHash,
    key: String,
    encoding: HmacEncoding,
}

impl Hmac {
    fn from_js_value(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<Self> {
        let options = args.get(0).unwrap();
        if options.is_null_or_undefined() {
            return Err(js_error!("options 不能为空"));
        }
        let options = options.as_object().unwrap();
        let hash = options.get(js_string!("hash"), ctx)?;
        let hash = if hash.is_string() {
            let hash = hash.to_string(ctx)?.to_std_string_escaped();
            match hash.as_str() {
                "sha1" => HmacHash::Sha1,
                "sha256" => HmacHash::Sha256,
                "sha384" => HmacHash::Sha384,
                "sha512" => HmacHash::Sha512,
                _ => return Err(js_error!("hash 不支持")),
            }
        } else {
            return Err(js_error!("hash 不是字符串"));
        };
        let encoding = options.get(js_string!("encoding"), ctx)?;
        let encoding = if encoding.is_string() {
            let encoding = encoding.to_string(ctx)?.to_std_string_escaped();
            match encoding.as_str() {
                "base64" => HmacEncoding::Base64,
                "hex" => HmacEncoding::Hex,
                _ => return Err(js_error!("encoding 不支持")),
            }
        } else {
            return Err(js_error!("encoding 不是字符串"));
        };
        let key = options
            .get(js_string!("key"), ctx)?
            .to_string(ctx)?
            .to_std_string_escaped();
        Ok(Hmac {
            hash,
            key,
            encoding,
        })
    }

    fn update(this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let options = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("get Hmac.prototype.encrypt called with invalid `this`")
            })?;
        let origin_text = args.get(0).unwrap().to_string(ctx)?.to_std_string_escaped();
        let plaintext = origin_text.as_bytes();
        let hmac_key = options.key.as_bytes();
        let encoding = &options.encoding;
        let result = match options.hash {
            HmacHash::Sha256 => {
                let mut hmac = HmacSha256::new_from_slice(hmac_key).unwrap();
                hmac.update(plaintext);
                hmac.finalize().into_bytes().to_vec()
            }
            HmacHash::Sha384 => {
                let mut hmac = Hmac384::new_from_slice(hmac_key).unwrap();
                hmac.update(plaintext);
                hmac.finalize().into_bytes().to_vec()
            }
            HmacHash::Sha512 => {
                let mut hmac = Hmac512::new_from_slice(hmac_key).unwrap();
                hmac.update(plaintext);
                hmac.finalize().into_bytes().to_vec()
            }
            HmacHash::Sha1 => {
                let mut hmac = HmacSha1::new_from_slice(hmac_key).unwrap();
                hmac.update(plaintext);
                hmac.finalize().into_bytes().to_vec()
            }
        };
        let result = match encoding {
            HmacEncoding::Base64 => BASE64_STANDARD.encode(result),
            HmacEncoding::Hex => hex::encode(result),
        };
        Ok(js_string!(result).into())
    }
}

impl Class for Hmac {
    const NAME: &'static str = "Hmac";
    const LENGTH: usize = 1;
    fn data_constructor(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<Self> {
        let hmac = Self::from_js_value(_this, args, ctx)?;
        Ok(hmac)
    }
    fn init(class: &mut boa_engine::class::ClassBuilder<'_>) -> boa_engine::JsResult<()> {
        class.method(
            js_string!("update"),
            1,
            NativeFunction::from_fn_ptr(Self::update),
        );
        Ok(())
    }
}

pub fn define_hmac(context: &mut Context) {
    context
        .register_global_class::<Hmac>()
        .expect("the Hmac builtin shouldn't exist");
}
