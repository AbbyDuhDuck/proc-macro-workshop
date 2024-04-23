use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let builder = syn::Ident::new(&format!("{}Builder", name), name.span());

    let fields =
    if let syn::Data::Struct(data) = &input.data {
        &data.fields
    } else { panic!() };

    let attr: Vec<syn::Ident> = fields.iter()
        .map(|f| f.ident.to_owned().unwrap() )
        .collect();

    let attr_seg: Vec<syn::PathSegment> = fields.iter()
        .map(|f| if let syn::Type::Path(ty) = &f.ty {
            ty.path.segments[0].to_owned()
        } else { panic!() })
        .collect();

    let attr_str: Vec<String> = attr.iter()
        .map(|ident| ident.to_string() )
        .collect();

    let block_impl = quote! {
        impl #name {
            fn builder() -> #builder {
                #builder {
                    #( #attr : None ),*
                }
            }
        }
    };

    let block_builder = quote! {
        pub struct #builder {                
            #( #attr: Option<#attr_seg> ),*
        }
    };

    let block_impl_builder = quote! {
        impl #builder {
            #(
                fn #attr(&mut self, #attr: #attr_seg) -> &mut Self {
                    self.#attr = Some(#attr);
                    self
                }
            )*

            pub fn build(&self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name{
                    #(
                        #attr : self.#attr
                            .clone()
                            .expect(&format!("Expected {} but found nothing.", #attr_str))
                    ),*
                })
            }
        }
    };

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        #block_impl  
        #block_builder
        #block_impl_builder
    };

    // Hand the output tokens back to the compiler
    expanded.into()
}
