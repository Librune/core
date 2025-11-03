use boa_engine::{js_string, property::Attribute};
use boa_runtime::Console;

use crate::{
    core::string_pool::common::init_common_strings,
    crypto::{aes::define_aes_crypto, hmac::define_hmac},
    env::env::regist_envs,
    global::{
        encoding::register_encoding_apis,
        rand_str::register_rand_str,
        uuid::{register_is_uuid, register_uuid},
        xml2json::register_xml_to_json,
    },
    prototype::{object::extend_object, string::extend_string},
    request::jreqwest::define_request,
    scraper::jscraper::define_scraper,
    BookCore,
};

pub fn init_runtime(core: &mut BookCore) {
    // 初始化常用字符串池
    init_common_strings();
    let context = &mut core.context;
    let console = Console::init(context);
    context
        .register_global_property(
            js_string!("console"),
            console,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .expect("Failed to register console");
    // 注册自定义类
    define_request(context);
    define_scraper(context);
    define_aes_crypto(context);
    define_hmac(context);

    // 注册环境变量方法
    regist_envs(context);

    // 注册全局工具函数
    register_uuid(context);
    register_is_uuid(context);
    register_xml_to_json(context);
    register_rand_str(context);

    // 注册浏览器兼容 API
    register_encoding_apis(context);

    // 扩展 JavaScript 原型
    extend_string(context);
    extend_object(context);
}
