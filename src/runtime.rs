use boa_engine::{js_string, property::Attribute};
use boa_runtime::Console;

use crate::{
    crypto::{aes::define_aes_crypto, hmac::define_hmac},
    env::env::regist_envs,
    global::{
        rand_str::regist_rand_str,
        uuid::{regist_is_uuid, regist_uuid},
        xml2json::regist_xml_to_json,
    },
    prototype::{object::extend_object, string::extend_string},
    request::jreqwest::define_request,
    scraper::jscraper::define_scraper,
    BookCore,
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
    // define_envs(context);
    regist_envs(context);
    define_request(context);
    define_scraper(context);
    define_aes_crypto(context);
    define_hmac(context);
    regist_xml_to_json(context);
    regist_rand_str(context);
    regist_uuid(context);
    regist_is_uuid(context);
    extend_string(context);
    extend_object(context);
}
