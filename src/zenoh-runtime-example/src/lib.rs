use serde::Deserialize;
use std::borrow::Borrow;
use zenoh_macros::{GenericRuntimeParam, RegisterParam};

pub const ZENOH_RUNTIME_ENV: &str = "ZENOH_RUNTIME";

#[derive(Deserialize, Debug, Clone, Copy, GenericRuntimeParam)]
#[serde(deny_unknown_fields, default)]
pub struct MyParams {
    pub threads: usize,
}

impl Default for MyParams {
    fn default() -> Self {
        Self { threads: 1 }
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, RegisterParam, Deserialize)]
#[param(MyParams)]
pub enum MyRuntime {
    #[serde(rename = "app")]
    #[param(threads = 1)]
    Application,

    #[serde(rename = "net")]
    #[param(threads = 2)]
    Network,
}
