//! 统一的错误处理系统
//!
//! 提供结构化的错误类型，替代简单的 String 错误消息。

use thiserror::Error;

/// BookCore 的主要错误类型
#[derive(Error, Debug)]
pub enum BookCoreError {
    /// JavaScript 执行错误
    #[error("JavaScript error: {0}")]
    JavaScript(#[from] boa_engine::JsError),

    /// 网络请求错误
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    /// 加密操作错误
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),

    /// 解析错误
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 自定义错误消息
    #[error("{0}")]
    Custom(String),
}

/// 网络相关错误
#[derive(Error, Debug)]
pub enum NetworkError {
    /// HTTP 请求失败
    #[error("Request failed: {0}")]
    RequestFailed(String),

    /// 无效的 URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// 连接超时
    #[error("Connection timeout")]
    Timeout,

    /// 字符集检测失败
    #[error("Charset detection failed")]
    CharsetError,

    /// Reqwest 错误
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

/// 加密相关错误
#[derive(Error, Debug)]
pub enum CryptoError {
    /// 密钥长度无效
    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    /// IV 长度无效
    #[error("Invalid IV length: expected {expected}, got {actual}")]
    InvalidIvLength { expected: usize, actual: usize },

    /// 加密失败
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// 解密失败
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// 不支持的算法
    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),

    /// Base64 解码错误
    #[error("Base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    /// Hex 解码错误
    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),
}

/// 解析相关错误
#[derive(Error, Debug)]
pub enum ParseError {
    /// HTML 解析错误
    #[error("HTML parse error: {0}")]
    Html(String),

    /// JSON 解析错误
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    /// XML 解析错误
    #[error("XML parse error: {0}")]
    Xml(String),

    /// 字符编码错误
    #[error("Encoding error: {0}")]
    Encoding(String),
}

/// 简化的 Result 类型别名
pub type BookResult<T> = std::result::Result<T, BookCoreError>;

// 实现 From<String> 以保持向后兼容
impl From<String> for BookCoreError {
    fn from(s: String) -> Self {
        BookCoreError::Custom(s)
    }
}

impl From<&str> for BookCoreError {
    fn from(s: &str) -> Self {
        BookCoreError::Custom(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = NetworkError::Timeout;
        assert_eq!(err.to_string(), "Connection timeout");

        let err = CryptoError::InvalidKeyLength {
            expected: 16,
            actual: 8,
        };
        assert_eq!(
            err.to_string(),
            "Invalid key length: expected 16, got 8"
        );
    }

    #[test]
    fn test_error_conversion() {
        let err: BookCoreError = "Custom error".into();
        assert!(matches!(err, BookCoreError::Custom(_)));
    }
}
