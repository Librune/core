use std::collections::HashMap;

use boa_engine::{Context, Source};
use serde_json::Value;
use tokio::runtime::Runtime;

use crate::runtime::init_runtime;

pub struct BookCore {
    pub code: String,
    pub context: Context,
    pub env: HashMap<String, String>,
}

impl BookCore {
    pub fn new(code: String, env: Option<HashMap<String, String>>) -> Self {
        let mut core = Self {
            code,
            env: env.unwrap_or_default(),
            context: Context::default(),
        };
        init_runtime(&mut core);
        core
    }

    pub fn eval(&mut self, code: String) -> Result<String, String> {
        let rt = Runtime::new().unwrap();
        let code = format!("{}\n{}", self.code, code);
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

    pub fn set_envs(&mut self, env: HashMap<String, String>) {
        self.env.extend(env);
    }

    pub fn get_envs(&self) -> HashMap<String, String> {
        self.env.clone()
    }

    pub fn clear_envs(&mut self) {
        self.env.clear();
    }

    pub fn get_metadata(&mut self) -> Result<String, String> {
        self.eval("metadata;".to_string())
    }

    pub fn get_form(&mut self) -> Result<String, String> {
        self.eval("form;".to_string())
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
