use aes::cipher::block_padding::{AnsiX923, Iso10126, Iso7816, NoPadding, Pkcs7, ZeroPadding};
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, StreamCipher};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use boa_engine::{
    class::Class, js_error, js_string, Context, JsData, JsNativeError, JsObject, JsResult, JsValue,
    NativeFunction,
};
use boa_gc::{Finalize, Trace};

use super::enums::{AesType, CipherMode, Encoding, PaddingType};

type Aes128Ofb = ofb::Ofb<aes::Aes128>;
type Aes192Ofb = ofb::Ofb<aes::Aes192>;
type Aes256Ofb = ofb::Ofb<aes::Aes256>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes128CfbDec = cfb_mode::Decryptor<aes::Aes128>;
type Aes192CbcDec = cbc::Decryptor<aes::Aes192>;
type Aes192CfbDec = cfb_mode::Decryptor<aes::Aes192>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
type Aes256CfbDec = cfb_mode::Decryptor<aes::Aes256>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CfbEnc = cfb_mode::Encryptor<aes::Aes128>;
type Aes192CbcEnc = cbc::Encryptor<aes::Aes192>;
type Aes192CfbEnc = cfb_mode::Encryptor<aes::Aes192>;
type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CfbEnc = cfb_mode::Encryptor<aes::Aes256>;

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
                "cfb" => CipherMode::Cfb,
                "ofb" => CipherMode::Ofb,
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
                "pkcs7" => PaddingType::Pkcs7,
                "zeropadding" => PaddingType::ZeroPadding,
                "iso10126" => PaddingType::Iso10126,
                "ansix923" => PaddingType::AnsiX923,
                "iso7816" => PaddingType::Iso7816,
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

    fn encrypt_cbc(
        key: &[u8],
        iv: &[u8],
        buf: &mut [u8],
        pt_len: usize,
        aes_type: &AesType,
        padding_type: &PaddingType,
    ) -> JsResult<Vec<u8>> {
        match aes_type {
            AesType::Aes128 => {
                let key = &key[..16];
                let cipher = Aes128CbcEnc::new_from_slices(key, iv)
                    .map_err(|_| js_error!("aes128 cbc new error"))?;
                Self::encrypt_with_padding(cipher, buf, pt_len, padding_type)
            }
            AesType::Aes192 => {
                let key = &key[..24];
                let cipher = Aes192CbcEnc::new_from_slices(key, iv)
                    .map_err(|_| js_error!("aes192 cbc new error"))?;
                Self::encrypt_with_padding(cipher, buf, pt_len, padding_type)
            }
            AesType::Aes256 => {
                let key = &key[..32];
                let cipher = Aes256CbcEnc::new_from_slices(key, iv)
                    .map_err(|_| js_error!("aes256 cbc new error"))?;
                Self::encrypt_with_padding(cipher, buf, pt_len, padding_type)
            }
        }
    }

    fn encrypt_cfb(
        key: &[u8],
        iv: &[u8],
        buf: &mut [u8],
        aes_type: &AesType,
        padding_type: &PaddingType,
    ) -> JsResult<Vec<u8>> {
        match aes_type {
            AesType::Aes128 => {
                let key = &key[..16];
                let cipher = Aes128CfbEnc::new_from_slices(key, iv)
                    .map_err(|_| js_error!("aes128 cfb new error"))?;
                Self::encrypt_with_padding(cipher, buf, 0, padding_type)
            }
            AesType::Aes192 => {
                let key = &key[..24];
                let cipher = Aes192CfbEnc::new_from_slices(key, iv)
                    .map_err(|_| js_error!("aes192 cfb new error"))?;
                Self::encrypt_with_padding(cipher, buf, 0, padding_type)
            }
            AesType::Aes256 => {
                let key = &key[..32];
                let cipher = Aes256CfbEnc::new_from_slices(key, iv)
                    .map_err(|_| js_error!("aes256 cfb new error"))?;
                Self::encrypt_with_padding(cipher, buf, 0, padding_type)
            }
        }
    }

    fn encrypt_ofb(key: &[u8], iv: &[u8], buf: &mut [u8], aes_type: &AesType) -> JsResult<Vec<u8>> {
        match aes_type {
            AesType::Aes128 => {
                let key = &key[..16];
                let mut buf1 = vec![0u8; buf.len()];
                let mut cipher = Aes128Ofb::new(key.into(), iv.into());
                cipher.apply_keystream_b2b(buf, &mut buf1).unwrap();
                Ok(buf1)
            }
            AesType::Aes192 => {
                let key = &key[..24];
                let mut buf1 = vec![0u8; buf.len()];
                let mut cipher = Aes192Ofb::new(key.into(), iv.into());
                cipher.apply_keystream_b2b(buf, &mut buf1).unwrap();
                Ok(buf1)
            }
            AesType::Aes256 => {
                let key = &key[..32];
                let mut buf1 = vec![0u8; buf.len()];
                let mut cipher = Aes256Ofb::new(key.into(), iv.into());
                cipher.apply_keystream_b2b(buf, &mut buf1).unwrap();
                Ok(buf1)
            }
        }
    }

    // 使用泛型处理不同的填充模式和加密器
    fn encrypt_with_padding<C>(
        cipher: C,
        buf: &mut [u8],
        pt_len: usize,
        padding_type: &PaddingType,
    ) -> JsResult<Vec<u8>>
    where
        C: BlockEncryptMut,
    {
        Ok(match padding_type {
            PaddingType::Pkcs7 => cipher
                .encrypt_padded_mut::<Pkcs7>(buf, pt_len)
                .map_err(|_| js_error!("aes cbc encrypt error"))?,
            PaddingType::NoPadding => cipher
                .encrypt_padded_mut::<NoPadding>(buf, pt_len)
                .map_err(|_| js_error!("aes cbc encrypt error"))?,
            PaddingType::ZeroPadding => cipher
                .encrypt_padded_mut::<ZeroPadding>(buf, pt_len)
                .map_err(|_| js_error!("aes cbc encrypt error"))?,
            PaddingType::Iso10126 => cipher
                .encrypt_padded_mut::<Iso10126>(buf, pt_len)
                .map_err(|_| js_error!("aes cbc encrypt error"))?,
            PaddingType::AnsiX923 => cipher
                .encrypt_padded_mut::<AnsiX923>(buf, pt_len)
                .map_err(|_| js_error!("aes cbc encrypt error"))?,
            PaddingType::Iso7816 => cipher
                .encrypt_padded_mut::<Iso7816>(buf, pt_len)
                .map_err(|_| js_error!("aes cbc encrypt error"))?,
        }
        .to_vec())
    }

    // 处理结果编码
    fn encode_result(data: &[u8], encoding: &Encoding) -> String {
        match encoding {
            Encoding::Base64 => BASE64.encode(data),
            Encoding::Hex => hex::encode(data),
        }
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
        let plaintext = origin_text.as_bytes();
        let pt_len = plaintext.len();
        let mut buf = vec![0u8; pt_len + 16];
        buf[..pt_len].copy_from_slice(plaintext);
        let encrypted = match options.cipher_mode {
            CipherMode::Cbc => Self::encrypt_cbc(
                &options.key,
                &options.iv,
                &mut buf,
                pt_len,
                &options.aes_type,
                &options.padding_type,
            )?,
            CipherMode::Cfb => Self::encrypt_cfb(
                &options.key,
                &options.iv,
                &mut buf,
                &options.aes_type,
                &options.padding_type,
            )?,
            CipherMode::Ofb => {
                Self::encrypt_ofb(&options.key, &options.iv, &mut buf, &options.aes_type)?
            }
        };
        let encrypted_str = Self::encode_result(&encrypted, &options.encoding);
        Ok(js_string!(encrypted_str).into())
    }

    fn decrypt_cbc(
        key: &[u8],
        iv: &[u8],
        buf: &mut [u8],
        aes_type: &AesType,
        padding_type: &PaddingType,
    ) -> JsResult<Vec<u8>> {
        match aes_type {
            AesType::Aes128 => {
                let key = &key[..16];
                let cipher = Aes128CbcDec::new_from_slices(key, iv)
                    .map_err(|_| js_error!("aes128 cbc new error"))?;
                Self::decrypt_with_padding(cipher, buf, padding_type)
            }
            AesType::Aes192 => {
                let key = &key[..24];
                let cipher = Aes192CbcDec::new_from_slices(key, iv)
                    .map_err(|_| js_error!("aes192 cbc new error"))?;
                Self::decrypt_with_padding(cipher, buf, padding_type)
            }
            AesType::Aes256 => {
                let key = &key[..32];
                let cipher = Aes256CbcDec::new_from_slices(key, iv)
                    .map_err(|_| js_error!("aes256 cbc new error"))?;
                Self::decrypt_with_padding(cipher, buf, padding_type)
            }
        }
    }

    fn decrypt_cfb(
        key: &[u8],
        iv: &[u8],
        buf: &mut [u8],
        aes_type: &AesType,
        padding_type: &PaddingType,
    ) -> JsResult<Vec<u8>> {
        match aes_type {
            AesType::Aes128 => {
                let key = &key[..16];
                let cipher = Aes128CfbDec::new_from_slices(key, iv)
                    .map_err(|_| js_error!("aes128 cfb new error"))?;
                Self::decrypt_with_padding(cipher, buf, padding_type)
            }
            AesType::Aes192 => {
                let key = &key[..24];
                let cipher = Aes192CfbDec::new_from_slices(key, iv)
                    .map_err(|_| js_error!("aes192 cfb new error"))?;
                Self::decrypt_with_padding(cipher, buf, padding_type)
            }
            AesType::Aes256 => {
                let key = &key[..32];
                let cipher = Aes256CfbDec::new_from_slices(key, iv)
                    .map_err(|_| js_error!("aes256 cfb new error"))?;
                Self::decrypt_with_padding(cipher, buf, padding_type)
            }
        }
    }

    fn decrypt_ofb(key: &[u8], iv: &[u8], buf: &mut [u8], aes_type: &AesType) -> JsResult<Vec<u8>> {
        match aes_type {
            AesType::Aes128 => {
                let key = &key[..16];
                let mut buf1 = vec![0u8; buf.len()];
                let mut cipher = Aes128Ofb::new(key.into(), iv.into());
                cipher.apply_keystream_b2b(buf, &mut buf1).unwrap();
                Ok(buf1)
            }
            AesType::Aes192 => {
                let key = &key[..24];
                let mut buf1 = vec![0u8; buf.len()];
                let mut cipher = Aes192Ofb::new(key.into(), iv.into());
                cipher.apply_keystream_b2b(buf, &mut buf1).unwrap();
                Ok(buf1)
            }
            AesType::Aes256 => {
                let key = &key[..32];
                let mut buf1 = vec![0u8; buf.len()];
                let mut cipher = Aes256Ofb::new(key.into(), iv.into());
                cipher.apply_keystream_b2b(buf, &mut buf1).unwrap();
                Ok(buf1)
            }
        }
    }

    // 使用泛型处理不同的填充模式和解密器
    fn decrypt_with_padding<C>(
        cipher: C,
        buf: &mut [u8],
        padding_type: &PaddingType,
    ) -> JsResult<Vec<u8>>
    where
        C: BlockDecryptMut,
    {
        Ok(match padding_type {
            PaddingType::Pkcs7 => cipher
                .decrypt_padded_mut::<Pkcs7>(buf)
                .map_err(|_| js_error!("aes cbc decrypt error"))?,
            PaddingType::NoPadding => cipher
                .decrypt_padded_mut::<NoPadding>(buf)
                .map_err(|_| js_error!("aes cbc decrypt error"))?,
            PaddingType::ZeroPadding => cipher
                .decrypt_padded_mut::<ZeroPadding>(buf)
                .map_err(|_| js_error!("aes cbc decrypt error"))?,
            PaddingType::Iso10126 => cipher
                .decrypt_padded_mut::<Iso10126>(buf)
                .map_err(|_| js_error!("aes cbc decrypt error"))?,
            PaddingType::AnsiX923 => cipher
                .decrypt_padded_mut::<AnsiX923>(buf)
                .map_err(|_| js_error!("aes cbc decrypt error"))?,
            PaddingType::Iso7816 => cipher
                .decrypt_padded_mut::<Iso7816>(buf)
                .map_err(|_| js_error!("aes cbc decrypt error"))?,
        }
        .to_vec())
    }
    // 处理结果编码
    fn decode_result(data: &[u8], encoding: &Encoding) -> JsResult<Vec<u8>> {
        match encoding {
            Encoding::Base64 => BASE64
                .decode(data)
                .map_err(|_| js_error!("base64 decode error")),
            Encoding::Hex => hex::decode(data).map_err(|_| js_error!("hex decode error")),
        }
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
        let encrypted_bytes = Self::decode_result(encrypted_data.as_bytes(), &options.encoding)?;
        let decrypted = match options.cipher_mode {
            CipherMode::Cbc => Self::decrypt_cbc(
                &options.key,
                &options.iv,
                &mut encrypted_bytes.clone(),
                &options.aes_type,
                &options.padding_type,
            )?,
            CipherMode::Cfb => Self::decrypt_cfb(
                &options.key,
                &options.iv,
                &mut encrypted_bytes.clone(),
                &options.aes_type,
                &options.padding_type,
            )?,
            CipherMode::Ofb => Self::decrypt_ofb(
                &options.key,
                &options.iv,
                &mut encrypted_bytes.clone(),
                &options.aes_type,
            )?,
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
