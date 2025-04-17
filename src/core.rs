use std::collections::HashMap;

use boa_engine::{Context, Source};
use reqwest::Client;
use tokio::runtime::Runtime;

use crate::runtime::init_runtime;

pub struct BookCore {
    pub code: String,
    pub context: Context,
    pub client: Client,
    pub env: HashMap<String, String>,
}

impl BookCore {
    pub fn new(code: String, env: Option<HashMap<String, String>>) -> Self {
        let mut core = Self {
            code,
            env: env.unwrap_or_default(),
            context: Context::default(),
            client: Client::builder()
                .danger_accept_invalid_certs(true)
                .use_rustls_tls()
                .build()
                .expect("Failed to create reqwest client"),
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
}
