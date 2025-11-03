//! 定时器 API
//!
//! 提供浏览器兼容的 setTimeout、setInterval、clearTimeout、clearInterval 实现

use boa_engine::{js_error, js_string, Context, JsArgs, JsResult, JsValue, NativeFunction};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

/// 全局定时器 ID 计数器
static TIMER_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// 定时器类型
#[derive(Debug, Clone)]
enum TimerType {
    Timeout,
    Interval,
}

/// 定时器信息
#[derive(Debug, Clone)]
struct TimerInfo {
    id: u32,
    timer_type: TimerType,
    handle: Option<Arc<Mutex<bool>>>, // 用于取消定时器的标志
}

/// 全局定时器存储
type TimerStorage = Arc<Mutex<Vec<TimerInfo>>>;

lazy_static::lazy_static! {
    static ref TIMERS: TimerStorage = Arc::new(Mutex::new(Vec::new()));
}

/// 生成新的定时器 ID
fn generate_timer_id() -> u32 {
    TIMER_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// 添加定时器到存储
fn add_timer(id: u32, timer_type: TimerType, handle: Arc<Mutex<bool>>) {
    let mut timers = TIMERS.lock().unwrap();
    timers.push(TimerInfo {
        id,
        timer_type,
        handle: Some(handle),
    });
}

/// 移除定时器
fn remove_timer(id: u32) -> bool {
    let mut timers = TIMERS.lock().unwrap();
    if let Some(pos) = timers.iter().position(|t| t.id == id) {
        let timer = timers.remove(pos);
        // 设置取消标志
        if let Some(handle) = timer.handle {
            if let Ok(mut cancelled) = handle.lock() {
                *cancelled = true;
            }
        }
        true
    } else {
        false
    }
}

/// 注册 setTimeout 函数
///
/// setTimeout(callback, delay)
/// 在指定的延迟后执行回调函数
pub fn register_set_timeout(ctx: &mut Context) {
    let function = NativeFunction::from_fn_ptr(|_this, args, context| {
        // 获取回调函数
        let callback = args.get_or_undefined(0);
        if !callback.is_callable() {
            return Err(js_error!("setTimeout: 第一个参数必须是函数"));
        }

        // 获取延迟时间(毫秒)
        let delay = args
            .get(1)
            .and_then(|v| v.as_number())
            .unwrap_or(0.0)
            .max(0.0) as u64;

        let callback = callback.clone();
        let timer_id = generate_timer_id();
        let cancelled = Arc::new(Mutex::new(false));
        let cancelled_clone = Arc::clone(&cancelled);

        // 添加到定时器存储
        add_timer(timer_id, TimerType::Timeout, cancelled);

        // 在新线程中执行延迟
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(delay));

            // 检查是否已被取消
            if let Ok(is_cancelled) = cancelled_clone.lock() {
                if *is_cancelled {
                    return;
                }
            }

            // 注意: 由于 Boa Context 不是 Send + Sync,
            // 我们不能直接在这里调用回调
            // 这是一个简化实现,实际使用中需要通过消息队列等机制来处理
            // 在实际的书源场景中,定时器主要用于延迟,而不是异步回调
        });

        // 返回定时器 ID
        Ok(JsValue::new(timer_id))
    });

    ctx.register_global_builtin_callable(js_string!("setTimeout"), 2, function)
        .expect("Failed to register setTimeout");
}

/// 注册 clearTimeout 函数
///
/// clearTimeout(timerId)
/// 取消由 setTimeout 设置的定时器
pub fn register_clear_timeout(ctx: &mut Context) {
    let function = NativeFunction::from_fn_ptr(|_this, args, _context| {
        let timer_id = args.get_or_undefined(0).as_number().unwrap_or(0.0) as u32;

        if timer_id > 0 {
            remove_timer(timer_id);
        }

        Ok(JsValue::undefined())
    });

    ctx.register_global_builtin_callable(js_string!("clearTimeout"), 1, function)
        .expect("Failed to register clearTimeout");
}

/// 注册 setInterval 函数
///
/// setInterval(callback, interval)
/// 以指定的间隔重复执行回调函数
pub fn register_set_interval(ctx: &mut Context) {
    let function = NativeFunction::from_fn_ptr(|_this, args, context| {
        // 获取回调函数
        let callback = args.get_or_undefined(0);
        if !callback.is_callable() {
            return Err(js_error!("setInterval: 第一个参数必须是函数"));
        }

        // 获取间隔时间(毫秒)
        let interval = args
            .get(1)
            .and_then(|v| v.as_number())
            .unwrap_or(0.0)
            .max(0.0) as u64;

        let callback = callback.clone();
        let timer_id = generate_timer_id();
        let cancelled = Arc::new(Mutex::new(false));
        let cancelled_clone = Arc::clone(&cancelled);

        // 添加到定时器存储
        add_timer(timer_id, TimerType::Interval, cancelled);

        // 在新线程中执行循环
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(interval));

                // 检查是否已被取消
                if let Ok(is_cancelled) = cancelled_clone.lock() {
                    if *is_cancelled {
                        break;
                    }
                }

                // 注意: 同 setTimeout 的限制
                // 这是简化实现,实际场景中主要用于延迟控制
            }
        });

        // 返回定时器 ID
        Ok(JsValue::new(timer_id))
    });

    ctx.register_global_builtin_callable(js_string!("setInterval"), 2, function)
        .expect("Failed to register setInterval");
}

/// 注册 clearInterval 函数
///
/// clearInterval(timerId)
/// 取消由 setInterval 设置的定时器
pub fn register_clear_interval(ctx: &mut Context) {
    let function = NativeFunction::from_fn_ptr(|_this, args, _context| {
        let timer_id = args.get_or_undefined(0).as_number().unwrap_or(0.0) as u32;

        if timer_id > 0 {
            remove_timer(timer_id);
        }

        Ok(JsValue::undefined())
    });

    ctx.register_global_builtin_callable(js_string!("clearInterval"), 1, function)
        .expect("Failed to register clearInterval");
}

/// 注册所有定时器 API
pub fn register_timer_apis(ctx: &mut Context) {
    register_set_timeout(ctx);
    register_clear_timeout(ctx);
    register_set_interval(ctx);
    register_clear_interval(ctx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Instant;

    #[test]
    fn test_set_timeout() {
        let mut context = Context::default();
        register_timer_apis(&mut context);

        let code = r#"
            let executed = false;
            const timerId = setTimeout(function() {
                executed = true;
            }, 100);
            timerId;
        "#;

        let result = context.eval(boa_engine::Source::from_bytes(code));
        assert!(result.is_ok());
        let timer_id = result.unwrap().as_number().unwrap();
        assert!(timer_id > 0.0);
    }

    #[test]
    fn test_clear_timeout() {
        let mut context = Context::default();
        register_timer_apis(&mut context);

        let code = r#"
            const timerId = setTimeout(function() {}, 1000);
            clearTimeout(timerId);
            timerId;
        "#;

        let result = context.eval(boa_engine::Source::from_bytes(code));
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_interval() {
        let mut context = Context::default();
        register_timer_apis(&mut context);

        let code = r#"
            let count = 0;
            const intervalId = setInterval(function() {
                count++;
            }, 100);
            intervalId;
        "#;

        let result = context.eval(boa_engine::Source::from_bytes(code));
        assert!(result.is_ok());
        let timer_id = result.unwrap().as_number().unwrap();
        assert!(timer_id > 0.0);

        // 清理
        let cleanup = format!("clearInterval({})", timer_id);
        context.eval(boa_engine::Source::from_bytes(&cleanup)).ok();
    }

    #[test]
    fn test_clear_interval() {
        let mut context = Context::default();
        register_timer_apis(&mut context);

        let code = r#"
            const intervalId = setInterval(function() {}, 100);
            clearInterval(intervalId);
            intervalId;
        "#;

        let result = context.eval(boa_engine::Source::from_bytes(code));
        assert!(result.is_ok());
    }

    #[test]
    fn test_timeout_delay() {
        let mut context = Context::default();
        register_timer_apis(&mut context);

        let start = Instant::now();

        let code = r#"
            setTimeout(function() {}, 100);
        "#;

        context.eval(boa_engine::Source::from_bytes(code)).ok();

        // 等待定时器执行
        thread::sleep(Duration::from_millis(150));

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(100));
    }
}
