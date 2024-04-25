// extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{parse_macro_input, AngleBracketedGenericArguments, DeriveInput, PathSegment};

// #[proc_macro_derive(Builder)]
#[proc_macro_derive(Builder, attributes(builder))]
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

    let ty_is_type = |ty: &syn::Type, s: &str| {
        if let syn::Type::Path(ref p) = ty {
            p.path.segments.last().unwrap().ident == s
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

    let attr_is_builder = |attr: &syn::Attribute| {
        if let syn::Meta::List(ref list) = attr.meta {
            if list.path.segments.len() == 1 && list.path.segments[0].ident == "builder" {
                // TODO from here
                let mut tokens = list.tokens.to_owned().into_iter();
                
                match tokens.next().unwrap() {
                    TokenTree::Ident(ref i) => assert_eq!(i, "each"),
                    tt => panic!("expected 'each', found '{tt}'"), 
                } // assert `each``

                match tokens.next().unwrap() {
                    TokenTree::Punct(ref p) => assert_eq!(p.as_char(), '='),
                    tt => panic!("expected 'each', found '{tt}'"),
                } // assert `=`

                let lit = match tokens.next().unwrap() {
                    TokenTree::Literal(l) => l,
                    tt => panic!("expected string, found '{tt}'"),
                }; // assert literal

                match syn::Lit::new(lit) {
                    syn::Lit::Str(s) => {
                        return Some( syn::Ident::new(&s.value(), s.span()) )
                    },
                    lit => panic!("expected string, found '{lit:?}'"),
                } // assert literal string
            }
            return None
        };
        None
    };
    let attr_has_builder = |attrs| {
        for attr in attrs {
            if let Some(i) = attr_is_builder(attr) {
                return Some(i)
            }
        }
        None
    };

    

    let fields =
    if let syn::Data::Struct(data) = &input.data {
        &data.fields
    } else { panic!() };

    // -=-=- Impl struct -=-=-

    let impl_fields = fields.iter().map(|f| {
        let name = &f.ident;
        if let Some(_) = attr_has_builder(&f.attrs) {
            if !ty_is_type(&f.ty, "Vec") { panic!("expected 'Vec' fround {:?}", f.ty) }
            return quote! { #name: vec![] };
        }
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
        } else if attr_has_builder(&f.attrs).is_some() {
            quote! { #name: #ty }
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

    let impl_builder_fields = fields.iter().filter_map(|f| {
        let name = &f.ident.to_owned().unwrap();
        let ty = match ty_is_option(&f.ty) {
            Some(inner_ty) => inner_ty,
            None => f.ty.to_owned()
        };
        
        if let Some(ref ident) = attr_has_builder(&f.attrs) {
            if ident == name { return None; }
            // -=-=- //

            return Some( quote! {
                fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = #name;
                    self
                }
            } );
        }

        Some( quote! {
            fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        } )
    });
    let impl_builder_build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        if ty_is_option(&f.ty).is_some() || attr_has_builder(&f.attrs).is_some() {
            quote! {
                #name : self.#name.clone()
            }
        } else {
            quote! {
                #name : self.#name.clone().ok_or(concat!(stringify!(#name), " is not set."))?
            }
        }
    });
    let impl_extend_methods = fields.iter().filter_map(|f| {
        for attr in &f.attrs {
            if let Some(arg) = attr_is_builder(&attr) {
                if !ty_is_type(&f.ty, "Vec") { panic!("expected 'Vec' fround {:?}", f.ty) }
                // -=-=- //
                let name = &f.ident;
                let ty = get_inner_type(&f.ty, 0);
                // -=-=- //
                return Some(quote! {
                    fn #arg(&mut self, #arg: #ty) -> &mut Self {
                        self.#name.push(#arg);
                        self
                    }
                });
            }
        }
        None
    });
    let impl_builder_build = quote! {
        pub fn build(&self) -> Result<#name, Box<dyn std::error::Error>> {
            Ok(#name{ #( #impl_builder_build_fields ),* })
    }
    };
    let block_impl_builder = quote! {
        impl #builder {
            #( #impl_builder_fields )*
            #( #impl_extend_methods )*

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
