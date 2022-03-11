use syn::{Error, Result};

use crate::parse;

pub struct Model {}

pub fn analyze(ast: parse::UmlState) -> Result<Model> {
    Err(Error::new_spanned(ast, "unimplemented"))
}
