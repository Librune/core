use boa_gc::{Finalize, Trace};

// 加密模式
#[derive(Debug, Clone, Trace, Finalize)]
pub enum CipherMode {
    // Ecb,
    Cbc,
    // Cfb,
    // Ofb,
}

// AES 类型
#[derive(Debug, Clone, Trace, Finalize)]
pub enum AesType {
    Aes128,
    Aes192,
    Aes256,
}

// 填充模式
#[derive(Debug, Clone, Trace, Finalize)]
pub enum PaddingType {
    Pkcs5,
    Pkcs7,
    NoPadding,
    // Zero,
    // Iso10126,
    // AnsiX923,
}

// 编码方式
#[derive(Debug, Clone, Trace, Finalize)]
pub enum Encoding {
    Hex,
    Base64,
}
