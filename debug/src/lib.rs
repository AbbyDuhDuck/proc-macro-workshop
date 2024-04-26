use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as syn::DeriveInput);

    // ident names for structs
    let struct_name = &input.ident;
    // let builder = syn::Ident::new(&format!("{}Builder", name), name.span());

    // get the structs fields to operate on.
    let fields =
    if let syn::Data::Struct(data) = &input.data {
        &data.fields
    } else { panic!() };

    let debug_fields = fields.iter().map(|field| {
        let name = &field.ident;

        quote! { .field(stringify!(#name), &self.#name) }
    });

    let output_tokens = quote! {
        impl std::fmt::Debug for #struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#struct_name))
                    #( #debug_fields )*
                    .finish()
            }
        }
    };
    output_tokens.into()
}
