use syn::*;

mod analyze;
mod codegen;
mod parse;

#[proc_macro]
pub fn umlstate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as parse::UmlState);
    let model = analyze::analyze(ast);
    let model = match model {
        Err(err) => return err.into_compile_error().into(),
        Ok(model) => model,
    };

    codegen::generate(model).into()
}
