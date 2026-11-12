use std::collections::HashMap;

use crate::memory::Memory;
use crate::value::Value;

#[derive(Default)]
pub(crate) struct Context {
    pub(crate) memory: Memory,
    pub(crate) vars: HashMap<String, Value>,
    pub(crate) start_address: u64,
    pub(crate) silent: bool,
}
