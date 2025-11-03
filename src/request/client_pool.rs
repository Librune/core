//! HTTP 客户端连接池
//!
//! 使用单例模式提供共享的 HTTP 客户端,避免每次请求都创建新的客户端实例。
//! 这可以显著提高性能,减少连接建立的开销。

use once_cell::sync::Lazy;
use reqwest::Client;
use std::time::Duration;

/// 全局 HTTP 客户端池
///
/// 使用 `once_cell::Lazy` 实现线程安全的单例模式。
/// 客户端配置:
/// - 接受无效证书(用于某些书源网站)
/// - 使用 rustls TLS 后端
/// - 连接池设置:最大空闲连接数为 10
/// - 连接池空闲超时: 90 秒
/// - 默认请求超时: 30 秒
pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .use_rustls_tls()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client pool")
});

/// 获取共享的 HTTP 客户端实例
///
/// # 示例
///
/// ```rust
/// use book_core::request::client_pool::get_client;
///
/// let client = get_client();
/// // 使用 client 发送请求
/// ```
#[inline]
pub fn get_client() -> &'static Client {
    &HTTP_CLIENT
}

/// 创建带自定义超时的客户端
///
/// 当需要不同于默认超时设置的请求时使用。
/// 注意:这会创建一个新的客户端实例,不会使用连接池。
///
/// # 参数
///
/// * `timeout` - 请求超时时间
///
/// # 示例
///
/// ```rust
/// use std::time::Duration;
/// use book_core::request::client_pool::create_client_with_timeout;
///
/// let client = create_client_with_timeout(Duration::from_secs(60));
/// ```
pub fn create_client_with_timeout(timeout: Duration) -> Client {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .use_rustls_tls()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        .timeout(timeout)
        .build()
        .expect("Failed to create HTTP client with custom timeout")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_client() {
        let client1 = get_client();
        let client2 = get_client();
        // 验证返回的是同一个客户端实例
        assert_eq!(
            client1 as *const Client,
            client2 as *const Client,
            "Should return the same client instance"
        );
    }

    #[test]
    fn test_create_client_with_timeout() {
        let timeout = Duration::from_secs(60);
        let client = create_client_with_timeout(timeout);
        // 验证客户端可以成功创建 (Client 类型本身存在就说明创建成功)
        drop(client); // 使用 client 避免未使用警告
    }
}
