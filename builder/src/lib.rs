use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AngleBracketedGenericArguments, DeriveInput, PathSegment};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let builder = syn::Ident::new(&format!("{}Builder", name), name.span());

    
    let get_inner_type = |ty: &syn::Type, sub_ty: usize| {
        if let syn::Type::Path(ref p) = ty {
            let PathSegment {arguments, ..} = p.path.segments.last().unwrap();
            if let syn::GenericArgument::Type(ty) =
            if let syn::PathArguments::AngleBracketed(AngleBracketedGenericArguments {args, ..}) = arguments {
                &args[sub_ty]
            } else { panic!() } {
                ty.to_owned()
            } else { panic!() }
        } else { panic!() }
    };

    let _ty_is_type = |ty: &syn::Type, s: &str| {
        if let syn::Type::Path(ref p) = ty {
            p.path.segments.len() == 1 &&
            p.path.segments[0].ident == s
        } else {
            false
        }
    };
    let ty_is_option = |ty: &syn::Type| {
        if let syn::Type::Path(ref p) = ty {
            if (
                p.path.segments.len() == 1 &&
                p.path.segments[0].ident == "Option"
            ) || (
                p.path.segments.len() == 2 &&
                p.path.segments[0].ident == "option" &&
                p.path.segments[1].ident == "Option"
            ) || (
                p.path.segments.len() == 3 &&
                p.path.segments[0].ident == "std" &&
                p.path.segments[1].ident == "option" &&
                p.path.segments[2].ident == "Option"
            ) {
                Some(get_inner_type(ty, 0))
            } else { None }
        } else { None }
    };

    let fields =
    if let syn::Data::Struct(data) = &input.data {
        &data.fields
    } else { panic!() };

    // -=-=- Impl struct -=-=-

    let impl_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! { #name: None }
    });
    let block_impl = quote! {
        impl #name {
            fn builder() -> #builder {
                #builder { #( #impl_fields ),* }
            }
        }
    };

    // -=-=- define Builder struct -=-=- //

    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if let Some(ty) = ty_is_option(ty) {
            quote! { #name: std::option::Option<#ty> }
        } else {
            quote! { #name: std::option::Option<#ty> }
        }
    });
    let block_builder = quote! {
        pub struct #builder {                
            #( #builder_fields ),*
        }
    };

    // -=-=- impl Builder -=-=- //

    let impl_builder_attrs = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = match ty_is_option(&f.ty) {
            Some(inner_ty) => inner_ty,
            None => f.ty.to_owned()
        };

        quote! {
            fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });
    let impl_builder_build_attrs = fields.iter().map(|f| {
        let name = &f.ident;
        if ty_is_option(&f.ty).is_some() {
            quote! {
                #name : self.#name.clone()
            }
        } else {
            quote! {
                #name : self.#name.clone().ok_or(concat!(stringify!(#name), " is not set."))?
            }
        }
    });
    let impl_builder_build = quote! {
        pub fn build(&self) -> Result<#name, Box<dyn std::error::Error>> {
            Ok(#name{ #( #impl_builder_build_attrs ),* })
    }
    };
    let block_impl_builder = quote! {
        impl #builder {
            #( #impl_builder_attrs )*

            #impl_builder_build
        }
    };

    // -=-=- Output -=-=- //

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        #block_impl  
        #block_builder
        #block_impl_builder
    };

    // Hand the output tokens back to the compiler
    expanded.into()
}

// fn get_sub_path(path: syn::Path, seg: usize, arg: usize) -> syn::Path {
//     let PathSegment {arguments, ..} = &path.segments[seg];
//     match arguments {
//          => {
//             if let syn::Type::Path(ty) =
//             if let syn::GenericArgument::Type(ty) =
//             &args[arg] {
//                 ty
//             } else { panic!() } {
//                 ty.path.to_owned()
//             } else { panic!() }
//         },
//         _ => panic!(),
//     }
// }
