use quote::quote;
use syn::*;

mod analyze;
mod parse;

#[proc_macro]
pub fn umlstate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as parse::UmlState);
    let model = analyze::analyze(ast);
    if let Err(err) = model {
        return err.into_compile_error().into();
    }

    quote! {}.into()
}
