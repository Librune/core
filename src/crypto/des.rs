//! DES/3DES 加密模块
//!
//! 提供 DES 和 3DES (Triple DES) 对称加密算法支持

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use boa_engine::{
    class::Class, js_error, js_string, Context, JsData, JsNativeError, JsResult, JsValue,
    NativeFunction,
};
use boa_gc::{Finalize, Trace};
use des::cipher::block_padding::{Pkcs7, ZeroPadding};
use des::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};

use super::enums::{CipherMode, Encoding, PaddingType};

type DesCbcEnc = cbc::Encryptor<des::Des>;
type DesCbcDec = cbc::Decryptor<des::Des>;
type Des3CbcEnc = cbc::Encryptor<des::TdesEde3>;
type Des3CbcDec = cbc::Decryptor<des::TdesEde3>;

/// DES/3DES 加密类型
#[derive(Debug, Clone, Trace, Finalize)]
pub enum DesType {
    /// DES (56-bit 密钥)
    Des,
    /// 3DES/TripleDES (168-bit 密钥)
    Des3,
}

#[derive(Debug, Trace, Finalize, JsData)]
struct Des {
    cipher_mode: CipherMode,
    des_type: DesType,
    padding_type: PaddingType,
    encoding: Encoding,
    key: Vec<u8>,
    iv: Vec<u8>,
}

impl Des {
    pub fn from_js_value(
        _this: &JsValue,
        args: &[JsValue],
        ctx: &mut Context,
    ) -> JsResult<Self> {
        let options = args.get(0).unwrap();
        if options.is_null_or_undefined() {
            return Err(js_error!("options 不能为空"));
        }
        let options = options.as_object().unwrap();

        // 读取 cipher_mode
        let cipher_mode = if let Ok(mode) = options.get(js_string!("cipherMode"), ctx) {
            let val_str = mode.to_string(ctx)?.to_std_string_escaped();
            match val_str.as_str() {
                "cbc" => CipherMode::Cbc,
                _ => return Err(js_error!("DES 仅支持 CBC 模式")),
            }
        } else {
            CipherMode::Cbc // 默认 CBC
        };

        // 读取 des_type
        let des_type = if let Ok(des_type) = options.get(js_string!("desType"), ctx) {
            let val_str = des_type.to_string(ctx)?.to_std_string_escaped();
            match val_str.as_str() {
                "des" => DesType::Des,
                "3des" | "tripledes" => DesType::Des3,
                _ => return Err(js_error!("不支持的 desType,支持: des, 3des")),
            }
        } else {
            DesType::Des // 默认 DES
        };

        // 读取 padding_type
        let padding_type = if let Ok(padding_type) = options.get(js_string!("paddingType"), ctx) {
            let val_str = padding_type.to_string(ctx)?.to_std_string_escaped();
            match val_str.as_str() {
                "pkcs7" => PaddingType::Pkcs7,
                "zeropadding" => PaddingType::ZeroPadding,
                _ => return Err(js_error!("DES 仅支持 pkcs7 和 zeropadding")),
            }
        } else {
            PaddingType::Pkcs7 // 默认 PKCS7
        };

        // 读取 encoding
        let encoding = if let Ok(encoding) = options.get(js_string!("encoding"), ctx) {
            let val_str = encoding.to_string(ctx)?.to_std_string_escaped();
            match val_str.as_str() {
                "base64" => Encoding::Base64,
                "hex" => Encoding::Hex,
                _ => return Err(js_error!("不支持的 encoding")),
            }
        } else {
            Encoding::Base64 // 默认 Base64
        };

        // 读取 key
        let key = if let Ok(key) = options.get(js_string!("key"), ctx) {
            let key_str = key.to_string(ctx)?.to_std_string_escaped();
            key_str.into_bytes()
        } else {
            return Err(js_error!("key 不能为空"));
        };

        // 验证密钥长度
        match des_type {
            DesType::Des => {
                if key.len() != 8 {
                    return Err(js_error!("DES 密钥长度必须为 8 字节"));
                }
            }
            DesType::Des3 => {
                if key.len() != 24 {
                    return Err(js_error!("3DES 密钥长度必须为 24 字节"));
                }
            }
        }

        // 读取 iv
        let iv = if let Ok(iv) = options.get(js_string!("iv"), ctx) {
            let iv_str = iv.to_string(ctx)?.to_std_string_escaped();
            iv_str.into_bytes()
        } else {
            return Err(js_error!("iv 不能为空"));
        };

        // 验证 IV 长度 (DES/3DES 都是 8 字节)
        if iv.len() != 8 {
            return Err(js_error!("IV 长度必须为 8 字节"));
        }

        Ok(Des {
            cipher_mode,
            des_type,
            padding_type,
            encoding,
            key,
            iv,
        })
    }

    /// 加密方法
    fn encrypt(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object".to_string())
        })?;
        let options = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Des object".to_string())
        })?;

        let plaintext = args.get(0).unwrap().to_string(ctx)?.to_std_string_escaped();
        let plaintext_bytes = plaintext.as_bytes();
        let pt_len = plaintext_bytes.len();

        // 创建缓冲区,大小为数据长度 + 一个块大小 (8字节用于DES)
        let mut buf = vec![0u8; pt_len + 8];
        buf[..pt_len].copy_from_slice(plaintext_bytes);

        let encrypted_bytes = match (&options.des_type, &options.padding_type) {
            (DesType::Des, PaddingType::Pkcs7) => {
                let cipher = DesCbcEnc::new_from_slices(&options.key, &options.iv)
                    .map_err(|e| js_error!("加密失败: {}", e))?;
                cipher
                    .encrypt_padded_mut::<Pkcs7>(&mut buf, pt_len)
                    .map_err(|e| js_error!("加密失败: {:?}", e))?
            }
            (DesType::Des, PaddingType::ZeroPadding) => {
                let cipher = DesCbcEnc::new_from_slices(&options.key, &options.iv)
                    .map_err(|e| js_error!("加密失败: {}", e))?;
                cipher
                    .encrypt_padded_mut::<ZeroPadding>(&mut buf, pt_len)
                    .map_err(|e| js_error!("加密失败: {:?}", e))?
            }
            (DesType::Des3, PaddingType::Pkcs7) => {
                let cipher = Des3CbcEnc::new_from_slices(&options.key, &options.iv)
                    .map_err(|e| js_error!("加密失败: {}", e))?;
                cipher
                    .encrypt_padded_mut::<Pkcs7>(&mut buf, pt_len)
                    .map_err(|e| js_error!("加密失败: {:?}", e))?
            }
            (DesType::Des3, PaddingType::ZeroPadding) => {
                let cipher = Des3CbcEnc::new_from_slices(&options.key, &options.iv)
                    .map_err(|e| js_error!("加密失败: {}", e))?;
                cipher
                    .encrypt_padded_mut::<ZeroPadding>(&mut buf, pt_len)
                    .map_err(|e| js_error!("加密失败: {:?}", e))?
            }
            _ => return Err(js_error!("不支持的加密配置")),
        };

        let result = match options.encoding {
            Encoding::Base64 => BASE64.encode(encrypted_bytes),
            Encoding::Hex => hex::encode(encrypted_bytes),
        };

        Ok(JsValue::new(js_string!(result)))
    }

    /// 解密方法
    fn decrypt(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        let obj = _this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an object".to_string())
        })?;
        let options = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Des object".to_string())
        })?;

        let ciphertext = args.get(0).unwrap().to_string(ctx)?.to_std_string_escaped();

        // 解码密文
        let mut ciphertext_bytes = match options.encoding {
            Encoding::Base64 => BASE64
                .decode(ciphertext)
                .map_err(|e| js_error!("Base64 解码失败: {}", e))?,
            Encoding::Hex => hex::decode(ciphertext).map_err(|e| js_error!("Hex 解码失败: {}", e))?,
        };

        let decrypted_bytes = match (&options.des_type, &options.padding_type) {
            (DesType::Des, PaddingType::Pkcs7) => {
                let cipher = DesCbcDec::new_from_slices(&options.key, &options.iv)
                    .map_err(|e| js_error!("解密失败: {}", e))?;
                cipher
                    .decrypt_padded_mut::<Pkcs7>(&mut ciphertext_bytes)
                    .map_err(|e| js_error!("解密失败: {:?}", e))?
            }
            (DesType::Des, PaddingType::ZeroPadding) => {
                let cipher = DesCbcDec::new_from_slices(&options.key, &options.iv)
                    .map_err(|e| js_error!("解密失败: {}", e))?;
                cipher
                    .decrypt_padded_mut::<ZeroPadding>(&mut ciphertext_bytes)
                    .map_err(|e| js_error!("解密失败: {:?}", e))?
            }
            (DesType::Des3, PaddingType::Pkcs7) => {
                let cipher = Des3CbcDec::new_from_slices(&options.key, &options.iv)
                    .map_err(|e| js_error!("解密失败: {}", e))?;
                cipher
                    .decrypt_padded_mut::<Pkcs7>(&mut ciphertext_bytes)
                    .map_err(|e| js_error!("解密失败: {:?}", e))?
            }
            (DesType::Des3, PaddingType::ZeroPadding) => {
                let cipher = Des3CbcDec::new_from_slices(&options.key, &options.iv)
                    .map_err(|e| js_error!("解密失败: {}", e))?;
                cipher
                    .decrypt_padded_mut::<ZeroPadding>(&mut ciphertext_bytes)
                    .map_err(|e| js_error!("解密失败: {:?}", e))?
            }
            _ => return Err(js_error!("不支持的解密配置")),
        };

        let result =
            String::from_utf8(decrypted_bytes.to_vec()).map_err(|e| js_error!("UTF-8 解码失败: {}", e))?;

        Ok(JsValue::new(js_string!(result)))
    }
}

impl Class for Des {
    const NAME: &'static str = "JDes";
    const LENGTH: usize = 0;

    fn init(class: &mut boa_engine::class::ClassBuilder<'_>) -> JsResult<()> {
        class
            .method(
                js_string!("encrypt"),
                1,
                NativeFunction::from_fn_ptr(Self::encrypt),
            )
            .method(
                js_string!("decrypt"),
                1,
                NativeFunction::from_fn_ptr(Self::decrypt),
            );
        Ok(())
    }

    fn data_constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Self> {
        Self::from_js_value(new_target, args, context)
    }
}

/// 注册 DES 类到 JavaScript 上下文
pub fn define_des_crypto(context: &mut Context) {
    context
        .register_global_class::<Des>()
        .expect("the JDes builtin shouldn't exist");
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Context;

    #[test]
    fn test_des_encrypt_decrypt() {
        let mut context = Context::default();
        define_des_crypto(&mut context);

        let code = r#"
            const des = new JDes({
                desType: 'des',
                cipherMode: 'cbc',
                paddingType: 'pkcs7',
                encoding: 'base64',
                key: '12345678',  // 8 字节
                iv: 'abcdefgh'    // 8 字节
            });

            const plaintext = "Hello, World!";
            const encrypted = des.encrypt(plaintext);
            const decrypted = des.decrypt(encrypted);

            decrypted === plaintext;
        "#;

        let result = context.eval(boa_engine::Source::from_bytes(code));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_boolean(), Some(true));
    }

    #[test]
    fn test_3des_encrypt_decrypt() {
        let mut context = Context::default();
        define_des_crypto(&mut context);

        let code = r#"
            const des3 = new JDes({
                desType: '3des',
                cipherMode: 'cbc',
                paddingType: 'pkcs7',
                encoding: 'base64',
                key: '123456781234567812345678',  // 24 字节
                iv: 'abcdefgh'                    // 8 字节
            });

            const plaintext = "Hello, 3DES!";
            const encrypted = des3.encrypt(plaintext);
            const decrypted = des3.decrypt(encrypted);

            decrypted === plaintext;
        "#;

        let result = context.eval(boa_engine::Source::from_bytes(code));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_boolean(), Some(true));
    }

    #[test]
    fn test_des_hex_encoding() {
        let mut context = Context::default();
        define_des_crypto(&mut context);

        let code = r#"
            const des = new JDes({
                desType: 'des',
                cipherMode: 'cbc',  // 需要明确指定 cipherMode
                paddingType: 'pkcs7',
                encoding: 'hex',
                key: '12345678',
                iv: 'abcdefgh'
            });

            const plaintext = "Test";
            const encrypted = des.encrypt(plaintext);
            const decrypted = des.decrypt(encrypted);

            decrypted === plaintext;
        "#;

        let result = context.eval(boa_engine::Source::from_bytes(code));
        if result.is_err() {
            eprintln!("Error: {:?}", result.as_ref().err());
        }
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_boolean(), Some(true));
    }
}
