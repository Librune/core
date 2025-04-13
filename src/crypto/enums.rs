use boa_gc::{Finalize, Trace};

// 加密模式
#[derive(Debug, Clone, Trace, Finalize)]
pub enum CipherMode {
    Cbc,
}

// AES 类型
#[derive(Debug, Clone, Trace, Finalize)]
pub enum AesType {
    Aes256,
}

// 填充模式
#[derive(Debug, Clone, Trace, Finalize)]
pub enum PaddingType {
    NoPadding,
    Pkcs5,
    Pkcs7,
}

// 编码方式
#[derive(Debug, Clone, Trace, Finalize)]
pub enum Encoding {
    Hex,
    Base64,
}

// 编码方式
#[derive(Debug, Clone, Trace, Finalize)]
pub enum KeyDerivation {
    Raw,
    Sha256,
}
