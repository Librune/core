//! 字符串池
//!
//! 提供字符串池化功能,减少重复字符串的内存占用。
//! 对于经常使用的字符串(如 JavaScript 属性名、常量等),使用字符串池可以显著减少内存使用。

use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;
use std::sync::RwLock;

/// 字符串池
///
/// 使用内部可变性和 Arc 实现线程安全的字符串池。
/// 相同的字符串只会存储一份,减少内存占用。
pub struct StringPool {
    pool: RwLock<FxHashMap<String, &'static str>>,
}

impl StringPool {
    /// 创建新的字符串池
    fn new() -> Self {
        Self {
            pool: RwLock::new(FxHashMap::default()),
        }
    }

    /// 从池中获取或插入字符串
    ///
    /// 如果字符串已经在池中,返回池中的引用。
    /// 如果字符串不在池中,将其添加到池中并返回引用。
    ///
    /// # 参数
    ///
    /// * `s` - 要池化的字符串
    ///
    /// # 返回
    ///
    /// 返回池中字符串的静态引用
    ///
    /// # 注意
    ///
    /// 这个方法使用 Box::leak 将字符串转换为静态生命周期。
    /// 字符串一旦进入池中就不会被释放,直到程序结束。
    /// 因此应该只对经常使用的字符串使用池化。
    pub fn get_or_insert(&self, s: &str) -> &'static str {
        // 先尝试读锁获取
        {
            let pool = self.pool.read().unwrap();
            if let Some(&pooled) = pool.get(s) {
                return pooled;
            }
        }

        // 如果不存在,获取写锁插入
        let mut pool = self.pool.write().unwrap();
        // 双重检查,防止在获取写锁期间被其他线程插入
        if let Some(&pooled) = pool.get(s) {
            return pooled;
        }

        // 创建静态字符串并插入池中
        let owned = s.to_string();
        let leaked: &'static str = Box::leak(owned.into_boxed_str());
        pool.insert(leaked.to_string(), leaked);
        leaked
    }

    /// 检查字符串是否在池中
    ///
    /// # 参数
    ///
    /// * `s` - 要检查的字符串
    ///
    /// # 返回
    ///
    /// 如果字符串在池中返回 true,否则返回 false
    pub fn contains(&self, s: &str) -> bool {
        let pool = self.pool.read().unwrap();
        pool.contains_key(s)
    }

    /// 获取池中字符串的数量
    pub fn len(&self) -> usize {
        let pool = self.pool.read().unwrap();
        pool.len()
    }

    /// 检查池是否为空
    pub fn is_empty(&self) -> bool {
        let pool = self.pool.read().unwrap();
        pool.is_empty()
    }
}

/// 全局字符串池实例
pub static STRING_POOL: Lazy<StringPool> = Lazy::new(StringPool::new);

/// 获取全局字符串池实例
///
/// # 示例
///
/// ```rust
/// use book_core::core::string_pool::get_pool;
///
/// let pool = get_pool();
/// let s1 = pool.get_or_insert("hello");
/// let s2 = pool.get_or_insert("hello");
/// assert_eq!(s1.as_ptr(), s2.as_ptr()); // 指向同一内存地址
/// ```
#[inline]
pub fn get_pool() -> &'static StringPool {
    &STRING_POOL
}

/// 将字符串池化
///
/// 这是 `get_pool().get_or_insert(s)` 的便捷方法
///
/// # 参数
///
/// * `s` - 要池化的字符串
///
/// # 返回
///
/// 返回池中字符串的静态引用
///
/// # 示例
///
/// ```rust
/// use book_core::core::string_pool::intern;
///
/// let s1 = intern("world");
/// let s2 = intern("world");
/// assert_eq!(s1.as_ptr(), s2.as_ptr());
/// ```
#[inline]
pub fn intern(s: &str) -> &'static str {
    get_pool().get_or_insert(s)
}

/// 常用字符串常量池
///
/// 为常用的 JavaScript 属性名和常量提供预池化的字符串
pub mod common {
    use super::intern;

    /// 初始化常用字符串到池中
    ///
    /// 应该在程序启动时调用一次
    pub fn init_common_strings() {
        // HTTP 相关
        intern("GET");
        intern("POST");
        intern("PUT");
        intern("DELETE");
        intern("PATCH");
        intern("HEAD");
        intern("OPTIONS");

        // 常用 HTTP 头
        intern("Content-Type");
        intern("User-Agent");
        intern("Authorization");
        intern("Accept");
        intern("Cookie");

        // 常用 JavaScript 属性
        intern("length");
        intern("prototype");
        intern("constructor");
        intern("name");
        intern("value");
        intern("type");
        intern("id");
        intern("class");
        intern("data");
        intern("status");
        intern("message");
        intern("error");
        intern("result");

        // 书源相关常用字段
        intern("metadata");
        intern("search");
        intern("detail");
        intern("catalog");
        intern("chapter");
        intern("forms");
        intern("actions");
        intern("title");
        intern("author");
        intern("cover");
        intern("description");
        intern("tags");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_pool_basic() {
        let pool = StringPool::new();
        let s1 = pool.get_or_insert("test");
        let s2 = pool.get_or_insert("test");

        // 应该返回相同的指针
        assert_eq!(s1.as_ptr(), s2.as_ptr());
        assert_eq!(s1, "test");
    }

    #[test]
    fn test_string_pool_different_strings() {
        let pool = StringPool::new();
        let s1 = pool.get_or_insert("hello");
        let s2 = pool.get_or_insert("world");

        // 不同的字符串应该有不同的指针
        assert_ne!(s1.as_ptr(), s2.as_ptr());
        assert_eq!(s1, "hello");
        assert_eq!(s2, "world");
    }

    #[test]
    fn test_string_pool_contains() {
        let pool = StringPool::new();
        pool.get_or_insert("exists");

        assert!(pool.contains("exists"));
        assert!(!pool.contains("not_exists"));
    }

    #[test]
    fn test_string_pool_len() {
        let pool = StringPool::new();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());

        pool.get_or_insert("one");
        assert_eq!(pool.len(), 1);
        assert!(!pool.is_empty());

        pool.get_or_insert("two");
        assert_eq!(pool.len(), 2);

        // 重复插入不应该增加大小
        pool.get_or_insert("one");
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_global_pool() {
        let s1 = get_pool().get_or_insert("global");
        let s2 = get_pool().get_or_insert("global");

        assert_eq!(s1.as_ptr(), s2.as_ptr());
    }

    #[test]
    fn test_intern_function() {
        let s1 = intern("convenience");
        let s2 = intern("convenience");

        assert_eq!(s1.as_ptr(), s2.as_ptr());
        assert_eq!(s1, "convenience");
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;

        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let s = intern(&format!("thread_{}", i % 3));
                    s
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // 相同值应该有相同的指针
        assert_eq!(results[0].as_ptr(), results[3].as_ptr()); // thread_0
        assert_eq!(results[1].as_ptr(), results[4].as_ptr()); // thread_1
        assert_eq!(results[2].as_ptr(), results[5].as_ptr()); // thread_2
    }

    #[test]
    fn test_common_strings_init() {
        common::init_common_strings();

        // 验证常用字符串已经在池中
        assert!(get_pool().contains("GET"));
        assert!(get_pool().contains("POST"));
        assert!(get_pool().contains("Content-Type"));
        assert!(get_pool().contains("length"));
        assert!(get_pool().contains("metadata"));
    }
}
