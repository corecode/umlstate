use quote::quote;

mod parse;

#[proc_macro]
pub fn umlstate(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote! {}.into()
}
