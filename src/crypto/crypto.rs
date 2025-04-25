use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use boa_engine::{
    class::Class, js_error, js_string, Context, JsData, JsNativeError, JsObject, JsResult, JsValue,
    NativeFunction,
};
use boa_gc::{Finalize, Trace};

use super::enums::{AesType, CipherMode, Encoding, PaddingType};

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes192CbcDec = cbc::Decryptor<aes::Aes192>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes192CbcEnc = cbc::Encryptor<aes::Aes192>;
type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;

#[derive(Debug, Trace, Finalize, JsData)]

struct AesCrypto {
    cipher_mode: CipherMode,
    aes_type: AesType,
    padding_type: PaddingType,
    encoding: Encoding,
    key: Vec<u8>,
    iv: Vec<u8>,
}

impl AesCrypto {
    pub fn form_js_value(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<Self> {
        let options = args.get(0).unwrap();
        if options.is_null_or_undefined() {
            return Err(js_error!("options 不能为空"));
        }
        let options = options.as_object().unwrap();
        let cipher_mode = if let Ok(mode) = options.get(js_string!("cipher_mode"), ctx) {
            let val_str = mode.to_string(ctx)?.to_std_string_escaped();
            match val_str.as_str() {
                "cbc" => CipherMode::Cbc,
                _ => return Err(js_error!("不支持的 cipher_mode")),
            }
        } else {
            return Err(js_error!("cipher_mode 不能为空"));
        };
        // 读取 aes_type
        let aes_type = if let Ok(aes_type) = options.get(js_string!("aes_type"), ctx) {
            let val_str = aes_type.to_string(ctx).unwrap().to_std_string_escaped();
            match val_str.as_str() {
                "aes128" => AesType::Aes128,
                "aes192" => AesType::Aes192,
                "aes256" => AesType::Aes256,
                _ => return Err(js_error!("不支持的 aes_type")),
            }
        } else {
            return Err(js_error!("aes_type 不能为空"));
        };
        // 读取 padding_type
        let padding_type = if let Ok(padding_type) = options.get(js_string!("padding_type"), ctx) {
            let val_str = padding_type.to_string(ctx).unwrap().to_std_string_escaped();
            match val_str.as_str() {
                "nopadding" => PaddingType::NoPadding,
                "pkcs5" => PaddingType::Pkcs5,
                "pkcs7" => PaddingType::Pkcs7,
                _ => return Err(js_error!("不支持的 padding_type")),
            }
        } else {
            return Err(js_error!("padding_type 不能为空"));
        };
        // 读取 encoding
        let encoding = if let Ok(encoding) = options.get(js_string!("encoding"), ctx) {
            let val_str = encoding.to_string(ctx).unwrap().to_std_string_escaped();
            match val_str.as_str() {
                "base64" => Encoding::Base64,
                "hex" => Encoding::Hex,
                _ => return Err(js_error!("不支持的 encoding")),
            }
        } else {
            return Err(js_error!("encoding 不能为空"));
        };
        // 读取 key
        let key = if let Ok(key) = options.get(js_string!("key"), ctx) {
            let val_str = key.to_string(ctx).unwrap().to_std_string_escaped();
            val_str.into_bytes()
        } else {
            return Err(js_error!("key 不能为空"));
        };
        // 读取 iv
        let iv: Vec<u8> = if let Ok(_iv) = options.get(js_string!("iv"), ctx) {
            if _iv.is_string() {
                let _iv_str = _iv.to_string(ctx)?.to_std_string_escaped();
                // 转 hex
                _iv_str.as_str().bytes().collect::<Vec<u8>>()
            } else if _iv.is_object() && _iv.as_object().unwrap().is_array() {
                let len = _iv
                    .as_object()
                    .unwrap()
                    .get(js_string!("length"), ctx)?
                    .to_number(ctx)? as usize;
                let mut iv = vec![0u8; len];
                let obj = _iv.as_object().unwrap();
                for i in 0..len {
                    iv[i] = obj
                        .get(i, ctx)
                        .map_err(|_| js_error!("iv 读取失败"))?
                        .to_number(ctx)? as u8;
                }
                iv
            } else {
                vec![0u8; 16]
            }
        } else {
            vec![0u8; 16]
        };
        // 如果 iv 长度不够 16 位，补齐，如果超出，截断
        let iv = if iv.len() < 16 {
            let mut iv = iv.clone();
            iv.resize(16, 0);
            iv
        } else if iv.len() > 16 {
            iv[..16].to_vec()
        } else {
            iv
        };
        Ok(Self {
            cipher_mode,
            aes_type,
            padding_type,
            encoding,
            key,
            iv,
        })
    }

    fn encrypt(this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let options = this
            .as_object()
            .and_then(JsObject::downcast_ref::<Self>)
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("get Decrypt.prototype.decrypt called with invalid `this`")
            })?;
        let origin_text = args.get(0).unwrap().to_string(ctx)?.to_std_string_escaped();
        let key = &options.key;
        let iv = &options.iv;
        let plaintext = origin_text.as_bytes();
        let mut buf = [0u8; 48];
        let pt_len = plaintext.len();
        buf[..pt_len].copy_from_slice(plaintext);
        let encrypted = match options.cipher_mode {
            CipherMode::Cbc => match options.aes_type {
                AesType::Aes128 => {
                    let key = &key[..16];
                    let cipher = Aes128CbcEnc::new_from_slices(key, &iv)
                        .map_err(|_| js_error!("aes128 cbc new error"))?;
                    cipher
                        .encrypt_padded_mut::<Pkcs7>(&mut buf, pt_len)
                        .map_err(|_| js_error!("aes128 cbc encrypt error"))?
                        .to_vec()
                }
                AesType::Aes192 => {
                    let key = &key[..24];
                    let cipher = Aes192CbcEnc::new_from_slices(key, &iv)
                        .map_err(|_| js_error!("aes192 cbc new error"))?;
                    cipher
                        .encrypt_padded_mut::<Pkcs7>(&mut buf, pt_len)
                        .map_err(|_| js_error!("aes192 cbc encrypt error"))?
                        .to_vec()
                }
                AesType::Aes256 => {
                    let key = &key[..32];
                    let cipher = Aes256CbcEnc::new_from_slices(key, &iv)
                        .map_err(|_| js_error!("aes256 cbc new error"))?;
                    cipher
                        .encrypt_padded_mut::<Pkcs7>(&mut buf, pt_len)
                        .map_err(|_| js_error!("aes256 cbc encrypt error"))?
                        .to_vec()
                }
            },
        };
        let encrypted_str = match options.encoding {
            Encoding::Base64 => BASE64.encode(&encrypted),
            Encoding::Hex => hex::encode(&encrypted),
        };
        Ok(js_string!(encrypted_str).into())
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
        let key = &options.key;
        let iv = &options.iv;
        let decrypted = match options.cipher_mode {
            CipherMode::Cbc => match options.aes_type {
                AesType::Aes128 => {
                    let key = &key[..16];
                    let cipher = Aes128CbcDec::new_from_slices(key, &iv)
                        .map_err(|_| js_error!("aes128 cbc new error"))?;
                    let mut buffer = encrypted_bytes.clone();
                    cipher
                        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
                        .map_err(|_| js_error!("aes128 cbc decrypt error"))?
                        .to_vec()
                }
                AesType::Aes192 => {
                    let key = &key[..24];
                    let cipher = Aes192CbcDec::new_from_slices(key, &iv)
                        .map_err(|_| js_error!("aes192 cbc new error"))?;
                    let mut buffer = encrypted_bytes.clone();
                    cipher
                        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
                        .map_err(|_| js_error!("aes192 cbc decrypt error"))?
                        .to_vec()
                }
                AesType::Aes256 => {
                    let key = &key[..32];
                    let cipher = Aes256CbcDec::new_from_slices(key, &iv)
                        .map_err(|_| js_error!("aes256 cbc new error"))?;
                    let mut buffer = encrypted_bytes.clone();
                    cipher
                        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
                        .map_err(|_| js_error!("aes256 cbc decrypt error"))?
                        .to_vec()
                }
            },
        };
        let decrypted_str =
            String::from_utf8(decrypted).map_err(|_| js_error!("utf8 decode error"))?;
        Ok(js_string!(decrypted_str).into())
    }
}

impl Class for AesCrypto {
    const NAME: &'static str = "AesCrypto";
    const LENGTH: usize = 1;
    fn data_constructor(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<Self> {
        let crypto = Self::form_js_value(_this, args, ctx)?;
        Ok(crypto)
    }
    fn init(class: &mut boa_engine::class::ClassBuilder<'_>) -> boa_engine::JsResult<()> {
        class.method(
            js_string!("encrypt"),
            1,
            NativeFunction::from_fn_ptr(Self::encrypt),
        );
        class.method(
            js_string!("decrypt"),
            1,
            NativeFunction::from_fn_ptr(Self::decrypt),
        );
        Ok(())
    }
}

pub fn define_aes_crypto(context: &mut Context) {
    context
        .register_global_class::<AesCrypto>()
        .expect("the AesCrypto builtin shouldn't exist");
}
