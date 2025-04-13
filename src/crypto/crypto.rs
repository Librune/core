use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, KeyIvInit};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use boa_engine::{
    class::Class, js_error, js_string, Context, JsData, JsNativeError, JsObject, JsResult, JsValue,
    NativeFunction,
};
use boa_gc::{Finalize, Trace};
use sha2::{Digest, Sha256};

use super::enums::{AesType, CipherMode, Encoding, KeyDerivation, PaddingType};

type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

#[derive(Debug, Trace, Finalize, JsData)]
struct Decrypt {
    cipher_mode: CipherMode,
    aes_type: AesType,
    padding_type: PaddingType,
    encoding: Encoding,
    key: Vec<u8>,
    key_derivation: KeyDerivation,
    iv: Option<Vec<u8>>,
}

impl Default for Decrypt {
    fn default() -> Self {
        Self {
            cipher_mode: CipherMode::Cbc,
            aes_type: AesType::Aes256,
            padding_type: PaddingType::Pkcs7,
            encoding: Encoding::Base64,
            key: vec![],
            key_derivation: KeyDerivation::Raw,
            iv: None,
        }
    }
}

impl Decrypt {
    pub fn form_js_value(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<Self> {
        let options = args.get(0).unwrap().as_object().unwrap();
        let mut cipher_mode = CipherMode::Cbc;
        // 读取 cipher_mode
        if let Ok(key) = options.get(js_string!("cipher_mode"), ctx) {
            let val_str = key.to_string(ctx)?.to_std_string_escaped();
            cipher_mode = match val_str.as_str() {
                "cbc" => CipherMode::Cbc,
                _ => return Err(js_error!("cipher_mode 只支持 cbc")),
            };
        }
        // 读取 aes_type
        let mut aes_type = AesType::Aes256;
        if let Ok(key) = options.get(js_string!("aes_type"), ctx) {
            let val_str = key.to_string(ctx)?.to_std_string_escaped();
            aes_type = match val_str.as_str() {
                "aes256" => AesType::Aes256,
                _ => return Err(js_error!("aes_type 只支持 aes256")),
            };
        }
        // 读取 padding_type
        let mut padding_type = PaddingType::Pkcs7;
        if let Ok(key) = options.get(js_string!("padding_type"), ctx) {
            let val_str = key.to_string(ctx)?.to_std_string_escaped();
            padding_type = match val_str.as_str() {
                "nopadding" => PaddingType::NoPadding,
                "pkcs5" => PaddingType::Pkcs5,
                "pkcs7" => PaddingType::Pkcs7,
                _ => return Err(js_error!("padding_type 只支持 nopadding, pkcs5, pkcs7")),
            };
        }
        // 读取 encoding
        let mut encoding = Encoding::Base64;
        if let Ok(key) = options.get(js_string!("encoding"), ctx) {
            let val_str = key.to_string(ctx)?.to_std_string_escaped();
            encoding = match val_str.as_str() {
                "base64" => Encoding::Base64,
                "hex" => Encoding::Hex,
                _ => return Err(js_error!("encoding 只支持 base64, hex")),
            };
        }
        // 读取 key
        let key = options
            .get(js_string!("key"), ctx)?
            .to_string(ctx)?
            .to_std_string_escaped()
            .into_bytes();
        // 读取 key_derivation
        let mut key_derivation = KeyDerivation::Raw;
        if let Ok(key) = options.get(js_string!("key_derivation"), ctx) {
            let val_str = key.to_string(ctx)?.to_std_string_escaped();
            key_derivation = match val_str.as_str() {
                "raw" => KeyDerivation::Raw,
                "sha256" => KeyDerivation::Sha256,
                _ => return Err(js_error!("key_derivation 只支持 raw")),
            };
        }
        // 读取 iv
        let mut iv: Option<Vec<u8>> = None;
        if let Ok(_iv) = options.get(js_string!("iv"), ctx) {
            let iv_str = _iv.to_string(ctx)?.to_std_string_escaped();
            iv = Some(
                iv_str
                    .chars()
                    .map(|c| c.to_digit(10).unwrap() as u8)
                    .collect(),
            );
        }
        Ok(Self {
            cipher_mode: cipher_mode,
            aes_type: aes_type,
            padding_type: padding_type,
            encoding: encoding,
            key: key,
            key_derivation: key_derivation,
            iv: iv,
        })
    }

    // 密钥派生函数
    fn derive_key(
        key: &[u8],
        method: KeyDerivation,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(match method {
            KeyDerivation::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(key);
                hasher.finalize().to_vec()
            }
            KeyDerivation::Raw => key.to_vec(),
        })
    }

    fn decrypt(this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let options = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("get Decrypt.prototype.decrypt called with invalid `this`")
            })?;
        let encrypted_data = args.get(0).unwrap().to_string(ctx)?.to_std_string_escaped();
        let encrypted_bytes = match options.encoding {
            Encoding::Base64 => BASE64
                .decode(&encrypted_data)
                .map_err(|_| js_error!("base64 decode error"))?,
            Encoding::Hex => {
                hex::decode(encrypted_data).map_err(|_| js_error!("hex decode error"))?
            }
        };
        let key = Self::derive_key(&options.key, options.key_derivation.clone())
            .map_err(|_| js_error!("key_derivation error"))?;
        let decrypted = match options.cipher_mode {
            CipherMode::Cbc => {
                let iv = match options.iv {
                    Some(ref v) => {
                        if v.len() >= 16 {
                            v[..16].to_vec()
                        } else {
                            vec![0u8; 16]
                        }
                    }
                    None => vec![0u8; 16],
                };
                match options.aes_type {
                    AesType::Aes256 => {
                        let cipher = Aes256CbcDec::new_from_slices(&key, &iv)
                            .map_err(|_| js_error!("aes256 cbc new error"))?;
                        let mut buffer = encrypted_bytes.clone();
                        cipher
                            .decrypt_padded_mut::<Pkcs7>(&mut buffer)
                            .map_err(|_| js_error!("aes256 cbc decrypt error"))?
                            .to_vec()
                    }
                }
            }
        };
        let decrypted_str =
            String::from_utf8(decrypted).map_err(|_| js_error!("utf8 decode error"))?;
        Ok(js_string!(decrypted_str).into())
    }
}

impl Class for Decrypt {
    const NAME: &'static str = "Decrypt";
    const LENGTH: usize = 1;

    fn data_constructor(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<Self> {
        let decrypt = Self::form_js_value(_this, args, ctx).unwrap_or_default();
        Ok(decrypt)
    }
    fn init(class: &mut boa_engine::class::ClassBuilder<'_>) -> boa_engine::JsResult<()> {
        class.method(
            js_string!("decrypt"),
            1,
            NativeFunction::from_fn_ptr(Self::decrypt),
        );
        Ok(())
    }
}

pub fn define_decrypt(context: &mut Context) {
    context
        .register_global_class::<Decrypt>()
        .expect("the Decrypt builtin shouldn't exist");
}
