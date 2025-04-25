use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use boa_engine::object::builtins::JsArray;
use boa_engine::{
    class::Class, js_error, js_string, Context, JsData, JsNativeError, JsObject, JsResult, JsValue,
    NativeFunction,
};
use boa_gc::{Finalize, Trace};
use sha2::{Digest, Sha256};

use super::enums::{AesType, CipherMode, Encoding, KeyDerivation, PaddingType};

type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;

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
        let iv: Option<Vec<u8>> = if let Ok(_iv) = options.get(js_string!("iv"), ctx) {
            if _iv.is_string() {
                let _iv_str = _iv.to_string(ctx)?.to_std_string_escaped();
                // 转 hex
                let vec = _iv_str
                    .as_str()
                    .chars()
                    .map(|c| c.to_digit(16).unwrap() as u8)
                    .collect::<Vec<u8>>();
                Some(vec)
            } else if _iv.is_object() && _iv.as_object().unwrap().is_array() {
                let len = _iv.to_length(ctx)? as usize;
                let mut iv = vec![0u8; len];
                let arr = JsArray::from_object(_iv.as_object().unwrap().clone())?;
                for i in 0..len {
                    let item = arr.get(i, ctx)?;
                    iv[i] = item.to_number(ctx)? as u8;
                }
                Some(iv)
            } else {
                None
            }
        } else {
            None
        };

        println!(
            "cipher_mode: {:?}, aes_type: {:?}, padding_type: {:?}, encoding: {:?}, key: {:?}, key_derivation: {:?}, iv: {:?}",
            cipher_mode, aes_type, padding_type, encoding, key, key_derivation, iv
        );

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

#[derive(Debug, Trace, Finalize, JsData)]

struct Encrypt {
    cipher_mode: CipherMode,
    aes_type: AesType,
    padding_type: PaddingType,
    encoding: Encoding,
    key: Vec<u8>,
    iv: Option<Vec<u8>>,
}

impl Encrypt {
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
        // // 读取 key_derivation
        // let key_derivation = if let Ok(key_derivation) = options.get(js_string!("key_derivation"), ctx) {
        //     let val_str = key_derivation
        //         .to_string(ctx)
        //         .unwrap()
        //         .to_std_string_escaped();
        //     match val_str.as_str() {
        //         "raw" => KeyDerivation::Raw,
        //         "sha256" => KeyDerivation::Sha256,
        //         _ => return Err(js_error!("不支持的 key_derivation")),
        //     }
        // } else {
        //     KeyDerivation::Raw
        // };
        // 读取 iv
        let iv: Option<Vec<u8>> = if let Ok(_iv) = options.get(js_string!("iv"), ctx) {
            if _iv.is_string() {
                let _iv_str = _iv.to_string(ctx)?.to_std_string_escaped();
                // 转 hex
                let vec = _iv_str.as_str().bytes().collect::<Vec<u8>>();
                Some(vec)
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
                Some(iv)
            } else {
                None
            }
        } else {
            None
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

        let encrypted = match options.cipher_mode {
            CipherMode::Cbc => match options.aes_type {
                AesType::Aes256 => {
                    let cipher = Aes256CbcEnc::new_from_slices(key, &iv)
                        .map_err(|_| js_error!("aes256 cbc new error"))?;
                    let plaintext = origin_text.as_bytes();
                    let mut buf = [0u8; 48];
                    let pt_len = plaintext.len();
                    buf[..pt_len].copy_from_slice(plaintext);
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
}

impl Class for Encrypt {
    const NAME: &'static str = "Encrypt";
    const LENGTH: usize = 1;
    fn data_constructor(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<Self> {
        let encrypt = Self::form_js_value(_this, args, ctx)?;
        Ok(encrypt)
    }
    fn init(class: &mut boa_engine::class::ClassBuilder<'_>) -> boa_engine::JsResult<()> {
        class.method(
            js_string!("value"),
            1,
            NativeFunction::from_fn_ptr(Self::encrypt),
        );
        Ok(())
    }
}

pub fn define_encrypt(context: &mut Context) {
    context
        .register_global_class::<Encrypt>()
        .expect("the Encrypt builtin shouldn't exist");
}

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
        let encrypted = match options.cipher_mode {
            CipherMode::Cbc => match options.aes_type {
                AesType::Aes256 => {
                    let cipher = Aes256CbcEnc::new_from_slices(key, &iv)
                        .map_err(|_| js_error!("aes256 cbc new error"))?;
                    let plaintext = origin_text.as_bytes();
                    let mut buf = [0u8; 48];
                    let pt_len = plaintext.len();
                    buf[..pt_len].copy_from_slice(plaintext);
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
        // class.method(
        //     js_string!("decrypt"),
        //     1,
        //     NativeFunction::from_fn_ptr(Self::decrypt),
        // );
        Ok(())
    }
}

pub fn define_aes_crypto(context: &mut Context) {
    context
        .register_global_class::<AesCrypto>()
        .expect("the AesCrypto builtin shouldn't exist");
}
