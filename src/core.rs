use boa_engine::{js_string, Context, JsNativeError, JsResult, JsValue, Source};
use boa_runtime::{Console, Logger};
use serde_json::{json, Value};
use tokio::runtime::Runtime;

use crate::runtime::init_runtime;

#[derive(Debug)]
pub struct BookCore {
    pub context: Context,
}

impl BookCore {
    pub fn init(code: String) -> Self {
        let mut core = Self {
            context: Context::default(),
        };
        init_runtime(&mut core);
        core.context
            .eval(Source::from_bytes(format!("setEnvs()").as_bytes()))
            .expect("Failed to eval __ENVS__");
        core.context
            .eval(Source::from_bytes(code.as_str()))
            .unwrap();
        core
    }

    pub fn regist_cust_logger(&mut self, logger: impl Logger + 'static) {
        let context = &mut self.context;
        context
            .global_object()
            .delete_property_or_throw(js_string!("console"), context)
            .expect("Failed to delete console");
        Console::register_with_logger(context, logger).expect("Failed to register custom logger");
        context
        .eval(Source::from_bytes("const console_log = console.log;console.log = function(...args) { const string_args =  args.map(arg => typeof arg === 'object' ? JSON.stringify(arg) : arg); console_log(...string_args);  };"))
        .expect("Failed to eval console");
    }

    pub fn eval(&mut self, code: String) -> Result<String, String> {
        let rt = Runtime::new().unwrap();
        let code = format!("{}", code);
        rt.block_on(async {
            let ctx = &mut self.context;
            ctx.eval(Source::from_bytes(code.as_bytes()))
                .map(|value| {
                    if value.is_object() {
                        value.to_json(ctx).unwrap().to_string()
                    } else {
                        value.to_string(ctx).unwrap().to_std_string_escaped()
                    }
                })
                .map_err(|err| err.to_string())
        })
    }

    pub fn call_func(&mut self, func: String, args: Vec<Value>) -> JsResult<JsValue> {
        let context = &mut self.context;
        let global = context.global_object();
        let func = global.get(js_string!(func), context)?;
        if !func.is_callable() {
            Err(JsNativeError::typ()
                .with_message(format!("{:?} is not callable", func))
                .into())
        } else {
            let func = func.as_callable().unwrap();
            let args: Vec<JsValue> = args
                .into_iter()
                .map(|arg| JsValue::from_json(&arg, context).unwrap())
                .collect();
            func.call(&JsValue::undefined(), &args, context)
        }
    }

    pub fn set_envs(&mut self, envs: Value) -> Result<(), String> {
        self.call_func("setEnvs".to_string(), vec![envs])
            .map(|_| ())
            .map_err(|err| err.to_string())
    }

    pub fn set_env(&mut self, key: String, value: Value) -> Result<(), String> {
        self.eval(format!("setEnv('{}', {:?})", key, value))?;
        Ok(())
    }

    pub fn get_envs(&mut self) -> Result<Value, String> {
        let res = self.eval(format!("getEnvs()"))?;
        let res = serde_json::from_str::<Value>(&res)
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
        Ok(res)
    }

    pub fn get_env(&mut self, key: String) -> Result<Value, String> {
        self.call_func("getEnv".to_string(), vec![json!(key)])
            .map(|value| value.to_json(&mut self.context).unwrap())
            .map_err(|err| err.to_string())
    }

    pub fn clear_envs(&mut self) {
        self.call_func("clearEnvs".to_string(), vec![])
            .expect("Failed to clear envs");
    }

    pub fn get_metadata(&mut self) -> Result<String, String> {
        self.eval("metadata".to_string())
    }

    pub fn get_forms(&mut self) -> Result<String, String> {
        self.eval("forms".to_string())
    }

    pub fn get_actions(&mut self) -> Result<String, String> {
        self.eval("actions".to_string())
    }

    pub fn run_action(&mut self, action: String) -> Result<String, String> {
        self.eval(format!("{}()", action).to_string())
    }

    pub fn search_books(&mut self, keyword: String, page: u8, count: u8) -> Result<Value, String> {
        let res = self.eval(format!(
            "search({{key: '{}', page: {}, count: {}}});",
            keyword, page, count
        ))?;
        let res = serde_json::from_str::<Value>(&res)
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
        Ok(res)
    }

    pub fn get_book_detail(&mut self, bid: String) -> Result<Value, String> {
        let res = self.eval(format!("detail({{bid: '{}'}});", bid))?;
        let res = serde_json::from_str::<Value>(&res)
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
        Ok(res)
    }

    pub fn get_catalog(&mut self, bid: String) -> Result<Value, String> {
        let res = self.eval(format!("catalog({{bid: '{}'}});", bid))?;
        let res = serde_json::from_str::<Value>(&res)
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
        Ok(res)
    }

    pub fn get_chapter(&mut self, bid: String, cid: String) -> Result<Value, String> {
        let res = self.eval(format!("chapter({{bid: '{}', cid: '{}'}});", bid, cid))?;
        let res = serde_json::from_str::<Value>(&res)
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
        Ok(res)
    }
}
