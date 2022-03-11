use quote::quote;

use crate::analyze;

pub fn generate(model: analyze::Model) -> proc_macro::TokenStream {
    let mut structs = Vec::new();

    for m in model.items {
        let ident = m.ident;

        structs.push(quote! {
            struct #ident;

            impl #ident {
                pub fn new() -> Self {
                    #ident
                }
            }
        });
    }

    quote! {
        #(#structs)*
    }
    .into()
}
