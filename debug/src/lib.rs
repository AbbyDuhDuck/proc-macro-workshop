use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;

    let output_tokens = quote! {
        /* ... */
    };
    output_tokens.into()
}
