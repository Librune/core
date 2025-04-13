use boa_engine::Context;

use crate::{
    crypto::crypto::define_decrypt,
    global::set_global_extra_bindings,
    prototype::{object::extend_object, string::extend_string},
    request::jreqwest::define_request,
    scraper::jscraper::define_scraper,
};

pub fn init_runtime(context: &mut Context) {
    define_request(context);
    define_scraper(context);
    define_decrypt(context);
    set_global_extra_bindings(context);
    extend_string(context);
    extend_object(context);
}
