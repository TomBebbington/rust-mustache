#![crate_name = "mustache"]

#![crate_type = "dylib"]
#![crate_type = "rlib"]

extern crate serialize;
extern crate unicode;

#[macro_use]
extern crate log;

pub use builder::{MapBuilder, VecBuilder};
pub use context::Context;
pub use data::Data;
pub use encoder::{Encoder, EncoderResult};
pub use error::Error;
pub use template::Template;

use std::path::Path;

pub mod builder;
mod data;
mod encoder;
mod error;
mod parser;
mod context;
mod compiler;
mod template;

/// Compiles a template from an `Iterator<char>`.
pub fn compile_iter<T: Iterator<Item=char>>(iter: T) -> Template {
    Context::new(".").compile(iter)
}

/// Compiles a template from a path.
/// returns None if the file cannot be read OR the file is not UTF-8 encoded
pub fn compile_path(path: &Path) -> Result<Template, Error> {
    Context::new(".").compile_path(path)
}

/// Compiles a template from a string.
pub fn compile_str(template: &str) -> Template {
    compile_iter(template.chars())
}
