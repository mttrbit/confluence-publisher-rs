extern crate confluence;
extern crate serde_json;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

pub mod publisher;
pub mod reader;
pub mod util;
pub mod yml;
