//! HTTP 响应缓存
//!
//! 提供基于内存的 HTTP 响应缓存机制,支持 TTL (Time To Live) 过期策略。
//! 可以显著减少重复请求的开销,提高性能。

use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// 缓存项
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 缓存的响应数据
    data: String,
    /// 缓存创建时间
    created_at: Instant,
    /// 存活时间 (TTL)
    ttl: Duration,
}

impl CacheEntry {
    /// 检查缓存项是否已过期
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// HTTP 响应缓存
///
/// 使用 FxHashMap (基于 FxHash) 提供更快的哈希性能。
/// 使用 RwLock 实现线程安全的读写访问。
pub struct ResponseCache {
    cache: RwLock<FxHashMap<String, CacheEntry>>,
}

impl ResponseCache {
    /// 创建新的缓存实例
    fn new() -> Self {
        Self {
            cache: RwLock::new(FxHashMap::default()),
        }
    }

    /// 获取缓存值
    ///
    /// # 参数
    ///
    /// * `key` - 缓存键
    ///
    /// # 返回
    ///
    /// 如果缓存存在且未过期,返回 Some(data),否则返回 None
    pub fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().unwrap();
        if let Some(entry) = cache.get(key) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    /// 设置缓存值
    ///
    /// # 参数
    ///
    /// * `key` - 缓存键
    /// * `data` - 要缓存的数据
    /// * `ttl` - 存活时间
    pub fn set(&self, key: String, data: String, ttl: Duration) {
        let entry = CacheEntry {
            data,
            created_at: Instant::now(),
            ttl,
        };
        let mut cache = self.cache.write().unwrap();
        cache.insert(key, entry);
    }

    /// 删除缓存值
    ///
    /// # 参数
    ///
    /// * `key` - 缓存键
    #[allow(dead_code)]
    pub fn remove(&self, key: &str) {
        let mut cache = self.cache.write().unwrap();
        cache.remove(key);
    }

    /// 清除所有过期的缓存项
    ///
    /// 这个方法应该定期调用以释放内存
    #[allow(dead_code)]
    pub fn clean_expired(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.retain(|_, entry| !entry.is_expired());
    }

    /// 清空所有缓存
    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// 获取缓存项数量
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }

    /// 检查缓存是否为空
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        let cache = self.cache.read().unwrap();
        cache.is_empty()
    }
}

/// 全局响应缓存实例
pub static RESPONSE_CACHE: Lazy<ResponseCache> = Lazy::new(ResponseCache::new);

/// 生成缓存键
///
/// 基于 URL 和请求方法生成唯一的缓存键
///
/// # 参数
///
/// * `method` - HTTP 方法 (GET, POST 等)
/// * `url` - 请求 URL
///
/// # 示例
///
/// ```rust
/// use book_core::request::cache::generate_cache_key;
///
/// let key = generate_cache_key("GET", "https://example.com/api");
/// ```
pub fn generate_cache_key(method: &str, url: &str) -> String {
    format!("{}:{}", method, url)
}

/// 获取全局缓存实例
///
/// # 示例
///
/// ```rust
/// use book_core::request::cache::get_cache;
/// use std::time::Duration;
///
/// let cache = get_cache();
/// cache.set("key".to_string(), "value".to_string(), Duration::from_secs(60));
/// let value = cache.get("key");
/// ```
#[inline]
pub fn get_cache() -> &'static ResponseCache {
    &RESPONSE_CACHE
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_set_and_get() {
        let cache = ResponseCache::new();
        let key = "test_key";
        let data = "test_data".to_string();
        let ttl = Duration::from_secs(60);

        cache.set(key.to_string(), data.clone(), ttl);
        let result = cache.get(key);

        assert_eq!(result, Some(data));
    }

    #[test]
    fn test_cache_expiration() {
        let cache = ResponseCache::new();
        let key = "expire_key";
        let data = "expire_data".to_string();
        let ttl = Duration::from_millis(100);

        cache.set(key.to_string(), data.clone(), ttl);

        // 立即获取应该成功
        assert_eq!(cache.get(key), Some(data));

        // 等待 TTL 过期
        thread::sleep(Duration::from_millis(150));

        // 过期后应该返回 None
        assert_eq!(cache.get(key), None);
    }

    #[test]
    fn test_cache_remove() {
        let cache = ResponseCache::new();
        let key = "remove_key";
        let data = "remove_data".to_string();
        let ttl = Duration::from_secs(60);

        cache.set(key.to_string(), data.clone(), ttl);
        assert_eq!(cache.get(key), Some(data));

        cache.remove(key);
        assert_eq!(cache.get(key), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache = ResponseCache::new();
        cache.set("key1".to_string(), "data1".to_string(), Duration::from_secs(60));
        cache.set("key2".to_string(), "data2".to_string(), Duration::from_secs(60));

        assert_eq!(cache.len(), 2);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_clean_expired() {
        let cache = ResponseCache::new();

        // 添加一个短 TTL 的项
        cache.set(
            "short_ttl".to_string(),
            "data1".to_string(),
            Duration::from_millis(100),
        );

        // 添加一个长 TTL 的项
        cache.set(
            "long_ttl".to_string(),
            "data2".to_string(),
            Duration::from_secs(60),
        );

        assert_eq!(cache.len(), 2);

        // 等待短 TTL 过期
        thread::sleep(Duration::from_millis(150));

        // 清理过期项
        cache.clean_expired();

        // 只剩下长 TTL 的项
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get("long_ttl"), Some("data2".to_string()));
    }

    #[test]
    fn test_generate_cache_key() {
        let key1 = generate_cache_key("GET", "https://example.com/api");
        let key2 = generate_cache_key("POST", "https://example.com/api");
        let key3 = generate_cache_key("GET", "https://example.com/api");

        // 不同方法生成不同的键
        assert_ne!(key1, key2);

        // 相同方法和 URL 生成相同的键
        assert_eq!(key1, key3);
    }

    #[test]
    fn test_global_cache() {
        let cache = get_cache();
        let key = "global_test";
        let data = "global_data".to_string();

        cache.set(key.to_string(), data.clone(), Duration::from_secs(60));
        assert_eq!(cache.get(key), Some(data));

        // 清理测试数据
        cache.remove(key);
    }
}
