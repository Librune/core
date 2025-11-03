pub mod console;
pub mod encoding;
pub mod rand_str;
pub mod uuid;
pub mod xml2json;

// 导出新的编码 API
pub use encoding::{register_atob, register_btoa, register_encoding_apis};
