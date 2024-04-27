
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {

    // -=-=- Helper Functions -=-=- //

    let has_attr_err = |attrs: &Vec<syn::Attribute>| {
        let err = |t: &dyn quote::ToTokens| {
            syn::Error::new_spanned(t, "expected `debug = \"...\"`").to_compile_error()
        };

        for attr in attrs {
            if let syn::Meta::NameValue(ref name_value) = attr.meta {
                // if name_value.path.segments.len() != 1 && name_value.path.segments[0].ident != "debug" {
                //     return Some(err(&name_value.path));
                // } // assert debug

                // if name_value.eq_token != syn::token::Eq::default() {
                //     return Some(err(&name_value.eq_token));
                // } // assert =
                // eprintln!("{:#?}", name_value.eq_token);

                if let syn::Lit::Str(_) = 
                if let syn::Expr::Lit(syn::ExprLit { ref lit, .. }) = name_value.value
                { lit } else { return Some(err(&name_value.value)) }
                { return None; } else { return Some(err(&name_value.value)) }
            }
            return Some(err(attr));
        }
        None
    };
    let has_debug_attr = |attrs: &Vec<syn::Attribute>| {
        for attr in attrs {
            if let syn::Meta::NameValue(ref name_value) = attr.meta {
                if let syn::Lit::Str(string) = 
                if let syn::Expr::Lit(syn::ExprLit { ref lit, .. }) = name_value.value
                { lit } else { return None }
                { return Some(string.to_owned()); } else { return None; }
            }
            return None;
        }
        None
    };

    // -=-=- THING -=-=- //

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
    
    // -=-=- Err Check -=-=- //

    for field in fields {
        // if attr parsing has error => raise the error
        if let Some(err) = has_attr_err(&field.attrs) {
            return err.into();
        }
    }

    // -=-=- Generate Output -=-=- //

    let debug_fields = fields.iter().enumerate()
    .map(|(i, field)| {
        let name = &field.ident;

        let mut sep = None;
        if i > 0 { sep = Some(quote! { write!(f, ", ")?; }) }

        let fmt = 
        if let Some(fmt) = has_debug_attr(&field.attrs) { fmt }
        else { syn::LitStr::new("{:?}", proc_macro2::Span::call_site()) };
        
        quote! { 
            #sep
            write!(f, concat!("{}: ", #fmt), stringify!(#name), &self.#name)?;
        }
    });


    let output_tokens = quote! {
        impl std::fmt::Debug for #struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{} {{ ", stringify!(#struct_name))?;
                #( #debug_fields )*
                write!(f, " }}")
            }
        }
    };
    output_tokens.into()
}
