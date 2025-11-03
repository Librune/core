//! 核心模块
//!
//! 包含 BookCore 的核心功能和类型定义。

pub mod error;
pub mod string_pool;

pub use error::{BookCoreError, BookResult, CryptoError, NetworkError, ParseError};
pub use string_pool::{get_pool, intern, StringPool};
