use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Cpu {
    pub name: String,
    pub attributes: HashMap<String, String>,
}
