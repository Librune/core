mod crypto;
mod env;
mod global;
mod prototype;
mod request;
mod runtime;
mod scraper;
use boa_engine::{js_string, Context, JsNativeError, JsResult, JsValue, Source};
use boa_runtime::{Console, Logger};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::runtime::Runtime;

use crate::runtime::init_runtime;

#[derive(Debug)]
pub struct BookCore {
    pub context: Context,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ProxyType {
    #[serde(rename = "http")]
    Http,
    #[serde(rename = "https")]
    Https,
    #[serde(rename = "socks4")]
    Socks4,
    #[serde(rename = "socks5")]
    Socks5,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Proxy {
    pub host: String,
    pub port: u8,
    pub r#type: Option<ProxyType>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaData {
    pub name: String,
    pub uuid: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub author: String,
    #[serde(rename = "userAgent")]
    pub user_agent: String,
    pub proxy: Option<Proxy>,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FormFieldType {
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "select")]
    Select,
    #[serde(rename = "checkbox")]
    Checkbox,
    #[serde(rename = "button")]
    Button,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormField {
    pub r#type: FormFieldType,
    pub field: String,
    pub label: String,
    pub placeholder: Option<String>,
    pub password: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Form {
    pub title: String,
    pub description: Option<String>,
    pub fields: Vec<FormField>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Action {
    pub label: String,
    pub action: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BookStatus {
    #[serde(rename = "0")]
    连载中,
    #[serde(rename = "1")]
    已完结,
    #[serde(rename = "2")]
    已下架,
    #[serde(rename = "3")]
    已断更,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchBook {
    pub id: String,
    pub name: String,
    pub author: Option<String>,
    pub cover: Option<String>,
    pub description: Option<String>,
    pub status: Option<BookStatus>,
    pub tags: Option<Vec<String>>,
    pub last_update_time: Option<String>,
    pub lastest_chapter: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BookLatestChapter {
    pub id: String,
    pub name: String,
    #[serde(rename = "updateTime")]
    pub update_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BookExtraData {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BookDetail {
    pub id: String,
    pub name: String,
    pub author: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "wordCount")]
    pub word_count: Option<u64>,
    pub cover: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<BookStatus>,
    #[serde(rename = "copyRight")]
    pub copy_right: Option<String>,
    #[serde(rename = "latestChapter")]
    pub latest_chapter: Option<BookLatestChapter>,
    #[serde(rename = "extraDatas")]
    pub extra_datas: Option<Vec<BookExtraData>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CatalogChapter {
    pub id: String,
    pub name: String,
    #[serde(rename = "isVip")]
    pub is_vip: Option<bool>,
    #[serde(rename = "canRead")]
    pub can_read: Option<bool>,
    #[serde(rename = "updateTime")]
    pub update_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CatalogVolume {
    pub id: String,
    pub name: String,
    pub chapters: Vec<CatalogChapter>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Chapter {
    pub id: String,
    pub content: String,
    pub name: Option<String>,
    #[serde(rename = "isVip")]
    pub is_vip: Option<bool>,
    #[serde(rename = "canRead")]
    pub can_read: Option<bool>,
    #[serde(rename = "updateTime")]
    pub update_time: Option<String>,
    #[serde(rename = "wordCount")]
    pub word_count: Option<u64>,
    #[serde(rename = "commentCount")]
    pub comment_count: Option<u64>,
    #[serde(rename = "commentBeginAtTitle")]
    pub comment_begin_at_title: Option<bool>,
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

    pub fn eval<T>(&mut self, code: String) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let rt = Runtime::new().unwrap();
        let code = format!("{}", code);
        rt.block_on(async {
            let ctx = &mut self.context;
            ctx.eval(Source::from_bytes(code.as_bytes()))
                .map(|value| {
                    if value.is_null_or_undefined() {
                        serde_json::from_value::<T>(serde_json::Value::Null).unwrap()
                    } else {
                        let value = value.to_json(ctx).unwrap();
                        serde_json::from_value::<T>(value).unwrap()
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

    pub fn set_env(&mut self, key: String, value: Value) -> Result<Value, String> {
        self.eval::<Value>(format!("setEnv('{}', {:?})", key, value))
    }

    pub fn get_envs(&mut self) -> Result<Value, String> {
        self.eval::<Value>(format!("getEnvs()"))
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

    pub fn get_metadata(&mut self) -> Result<MetaData, String> {
        self.eval::<MetaData>("metadata".to_string())
    }

    pub fn get_forms(&mut self) -> Result<Vec<Form>, String> {
        self.eval::<Vec<Form>>("forms".to_string())
    }

    pub fn get_actions(&mut self) -> Result<Vec<Action>, String> {
        self.eval::<Vec<Action>>("actions".to_string())
    }

    pub fn run_action(&mut self, action: String) -> Result<Value, String> {
        self.eval(format!("{}()", action).to_string())
    }

    pub fn search_books(
        &mut self,
        keyword: String,
        page: u8,
        count: u8,
    ) -> Result<Vec<SearchBook>, String> {
        self.eval::<Vec<SearchBook>>(format!(
            "search({{key: '{}', page: {}, count: {}}});",
            keyword, page, count
        ))
    }

    pub fn get_book_detail(&mut self, bid: String) -> Result<BookDetail, String> {
        self.eval::<BookDetail>(format!("detail({{bid: '{}'}});", bid))
    }

    pub fn get_catalog(&mut self, bid: String) -> Result<Vec<CatalogVolume>, String> {
        self.eval::<Vec<CatalogVolume>>(format!("catalog({{bid: '{}'}});", bid))
    }

    pub fn get_chapter(&mut self, bid: String, cid: String) -> Result<Chapter, String> {
        self.eval::<Chapter>(format!("chapter({{bid: '{}', cid: '{}'}});", bid, cid))
    }
}
