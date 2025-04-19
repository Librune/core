use boa_engine::{js_string, property::Attribute};
use boa_runtime::Console;

use crate::{
    core::BookCore,
    crypto::crypto::define_decrypt,
    env::env::define_envs,
    global::{
        uuid::{regist_is_uuid, regist_uuid},
        xml2json::regist_xml_to_json,
    },
    prototype::{object::extend_object, string::extend_string},
    request::jreqwest::define_request,
    scraper::jscraper::define_scraper,
};

pub fn init_runtime(core: &mut BookCore) {
    let context = &mut core.context;
    let console = Console::init(context);
    context
        .register_global_property(
            js_string!("console"),
            console,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .expect("Failed to register console");
    define_envs(context);
    define_request(context);
    define_scraper(context);
    define_decrypt(context);
    regist_xml_to_json(context);
    regist_uuid(context);
    regist_is_uuid(context);
    extend_string(context);
    extend_object(context);
}
