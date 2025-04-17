mod core;
mod crypto;
mod global;
mod prototype;
mod request;
mod runtime;
mod scraper;
use std::{collections::HashMap, sync::Mutex};

use once_cell::sync::Lazy;
use serde_json::Value;

pub use crate::core::BookCore;

type BCResult = (Value, Value);

static SCRIPTS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static ENVS: Lazy<Mutex<HashMap<String, HashMap<String, String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }

// /// 向脚本映射中添加一个新的脚本
// ///
// /// # 参数
// ///
// /// * `code` - 对应的脚本内容
// /// * `uuid` - 脚本的唯一标识符
// pub fn add_script(code: String) -> String {
//     let mut map = SCRIPTS.lock().unwrap();
//     let metadata = get_metadata(None, Some(code.clone())).unwrap();
//     let uuid = serde_json::from_str::<HashMap<String, String>>(&metadata)
//         .unwrap()
//         .get("uuid")
//         .unwrap()
//         .to_string();
//     map.insert(uuid.clone(), code);
//     uuid
// }

// pub fn get_script(uuid: &str) -> Option<String> {
//     let map = SCRIPTS.lock().unwrap();
//     let code = map.get(uuid).cloned();
//     let envs = get_env(uuid);
//     code.map(|code| {
//         let code = code.clone();
//         format!(
//             "const __ENVS__ = {};\n{}",
//             serde_json::to_string(&envs).unwrap(),
//             code
//         )
//     })
// }

// pub fn remove_ecma_script(uuid: &str) -> Option<String> {
//     let mut map = SCRIPTS.lock().unwrap();
//     map.remove(uuid)
// }

// pub fn clear_ecma_script() {
//     let mut map = SCRIPTS.lock().unwrap();
//     map.clear();
// }

// pub fn set_env(uuid: String, value: HashMap<String, String>) {
//     let mut map = ENVS.lock().unwrap();
//     map.insert(uuid, value);
// }

// pub fn get_env(uuid: &str) -> HashMap<String, String> {
//     let map = ENVS.lock().unwrap();
//     map.get(uuid).cloned().unwrap_or_default()
// }

// pub fn remove_env(uuid: &str) -> Option<HashMap<String, String>> {
//     let mut map = ENVS.lock().unwrap();
//     map.remove(uuid)
// }

// pub fn clear_env() {
//     let mut map = ENVS.lock().unwrap();
//     map.clear();
// }

// pub fn js_eval(code: String) -> Result<BCResult, String> {
//     let mut core = {
//         let mut core = BookCore {
//             code,
//             context: Context::default(),
//             request: Client::builder()
//                 .danger_accept_invalid_certs(true)
//                 .use_rustls_tls()
//                 .build()
//                 .expect("Failed to create reqwest client"),
//         };
//         init_runtime(&mut core.context);
//         core
//     };
//     core.eval(code)
// }

// fn get_code(uuid: String) -> Result<String, String> {
//     let mut code = SCRIPTS
//         .lock()
//         .unwrap()
//         .get(&uuid)
//         .unwrap_or_else(|| {
//             panic!("uuid {} 不存在", uuid);
//         })
//         .clone();
//     let envs = get_env(&uuid);
//     code = format!(
//         "const __ENVS__ = {};\n{}",
//         serde_json::to_string(&envs).unwrap(),
//         code
//     );
//     Ok(code)
// }

// pub fn get_metadata(uuid: Option<String>, code: Option<String>) -> Result<String, String> {
//     if uuid.is_none() && code.is_none() {
//         return Err("uuid 和 code 不能同时为空".to_string());
//     }
//     let mut code = if let Some(uuid_value) = uuid {
//         SCRIPTS
//             .lock()
//             .unwrap()
//             .get(&uuid_value)
//             .cloned()
//             .ok_or_else(|| "uuid 不存在".to_string())
//             .unwrap()
//     } else {
//         code.unwrap()
//     };
//     code = format!("{}\nmetadata;", code);
//     js_eval(code)
// }

// pub fn get_form(uuid: Option<String>, code: Option<String>) -> Result<String, String> {
//     if uuid.is_none() && code.is_none() {
//         return Err("uuid 和 code 不能同时为空".to_string());
//     }
//     let mut code = if let Some(uuid_value) = uuid {
//         SCRIPTS
//             .lock()
//             .unwrap()
//             .get(&uuid_value)
//             .cloned()
//             .ok_or_else(|| "uuid 不存在".to_string())
//             .unwrap()
//     } else {
//         code.unwrap()
//     };
//     code = format!("{}\nform;", code);
//     js_eval(code)
// }

// pub fn search_books(uuid: String, key: String, page: u8, count: u8) -> Result<Value, String> {
//     let mut code = get_code(uuid)?;
//     code = format!(
//         "{}\nsearch({{key: '{}', page: {}, count: {}}});",
//         code, key, page, count
//     );
//     let res = js_eval(code)?;
//     let res = serde_json::from_str::<Value>(&res)
//         .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
//     Ok(res)
// }

// pub fn run_code(uuid: String, code: String) -> Result<Value, String> {
//     let mut _code = get_code(uuid).unwrap();
//     _code = format!("{}\n{}", _code, code);
//     let res = js_eval(_code)?;
//     let res = serde_json::from_str::<Value>(&res).unwrap();
//     if res.is_object() {
//         let obj = res.as_object().unwrap();
//         if obj.contains_key("error") {
//             return Err(obj.get("error").unwrap().to_string());
//         }
//     }
//     Ok(res)
// }

// pub fn run_action(uuid: String, action: String) -> Result<Value, String> {
//     let mut code = get_code(uuid).unwrap();
//     code = format!("{}\n{}()", code, action);
//     let res = js_eval(code)?;
//     let res = serde_json::from_str::<Value>(&res).unwrap();
//     if res.is_object() {
//         let obj = res.as_object().unwrap();
//         if obj.contains_key("error") {
//             return Err(obj.get("error").unwrap().to_string());
//         }
//     }
//     Ok(res)
// }

// pub fn get_book_detail(uuid: String, bid: String) -> Result<Value, String> {
//     let mut code = get_code(uuid)?;
//     code = format!("{}\ndetail({{bid: '{}'}});", code, bid);
//     let res = js_eval(code)?;
//     let res = serde_json::from_str::<Value>(&res)
//         .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
//     Ok(res)
// }

// pub fn get_catalog(uuid: String, bid: String) -> Result<Value, String> {
//     let mut code = get_code(uuid)?;
//     code = format!("{}\ncatalog({{bid: '{}'}});", code, bid);
//     let res = js_eval(code)?;
//     let res = serde_json::from_str::<Value>(&res)
//         .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
//     Ok(res)
// }

// pub fn get_chapter(uuid: String, bid: String, cid: String) -> Result<Value, String> {
//     let mut code = get_code(uuid)?;
//     code = format!("{}\nchapter({{bid: '{}',cid:'{}'}});", code, bid, cid);
//     let res = js_eval(code)?;
//     let res = serde_json::from_str::<Value>(&res)
//         .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
//     Ok(res)
// }
