use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Cpu {
    /// Something like "Intel core i5-1234 processor"
    pub name: String,
    pub attributes: HashMap<String, String>,
}
