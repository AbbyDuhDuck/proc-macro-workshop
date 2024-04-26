// extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{parse_macro_input, AngleBracketedGenericArguments, DeriveInput, PathSegment};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    //! This macro generates the boilerplate code involved in implementing the [builder
    //! pattern] in Rust. Builders are a mechanism for instantiating structs, especially
    //! structs with many fields, and especially if many of those fields are optional or
    //! the set of fields may need to grow backward compatibly over time.
    //! 
    //! [builder pattern]: https://en.wikipedia.org/wiki/Builder_pattern
    //! 
    //! ---
    //! 
    //! ## Example Use
    //! 
    //! ```
    //! use derive_builder::Builder;
    //! 
    //! #[derive(Builder)]
    //! pub struct Command {
    //!     executable: String,
    //!     #[builder(each = "arg")]
    //!     args: Vec<String>,
    //!     current_dir: Option<String>,
    //! }
    //! 
    //! fn main() {
    //!     let command = Command::builder()
    //!         .executable("cargo".to_owned())
    //!         .arg("build".to_owned())
    //!         .arg("--release".to_owned())
    //!         .build()
    //!         .unwrap();
    //!     assert_eq!(command.executable, "cargo");
    //! }
    //! ```
   
    // -=-=- Helper Functions -=-=- //

    // Function to get the inner type of a `syn::Type` and panic! if it doesn't find one.
    // Note: you should check your type before traversing
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

    // Check if a syn:Type has the ident == to string `s`
    let ty_is_type = |ty: &syn::Type, s: &str| {
        if let syn::Type::Path(ref p) = ty {
            p.path.segments.last().unwrap().ident == s
        } else {
            false
        }
    };
    // Check if a syn::Type is an Option<...> and return the inner type or None 
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

    // Make a standardized attr error and pass the tokens for the span
    let make_attr_error = |t: &dyn quote::ToTokens| {
        syn::Error::new_spanned(t, "expected `builder(each = \"...\")`").to_compile_error()
    };

    // check if an attr is a builder
    // returns (ident: Option<syn::Ident>, err: Option<TokenStream>)
    // it is a properly formatted builder `if let (Some(_), _) = attr_is_builder(&attr)`
    let attr_is_builder = |attr: &syn::Attribute| {
        if let syn::Meta::List(ref list) = attr.meta {
            if list.path.segments.len() == 1 && list.path.segments[0].ident == "builder" {
                let mut tokens = list.tokens.to_owned().into_iter();
                
                match tokens.next().unwrap() {
                    TokenTree::Ident(ref i) => if i != "each" { return (None, Some(make_attr_error(list)) ) },
                    _ => return (None, Some(make_attr_error(list)) ), 
                } // assert `each``

                match tokens.next().unwrap() {
                    TokenTree::Punct(ref p) => if p.as_char() != '=' { return (None, Some(make_attr_error(list)) ) },
                    _ => return (None, Some(make_attr_error(list)) ), 
                } // assert `=`

                let lit = match tokens.next().unwrap() {
                    TokenTree::Literal(l) => l,
                    _ => return (None, Some(make_attr_error(list)) ),
                }; // assert literal

                match syn::Lit::new(lit) {
                    syn::Lit::Str(s) => {
                        return (Some(syn::Ident::new(&s.value(), s.span())), None)
                    },
                    _ => return (None, Some(make_attr_error(list)) ),
                } // assert literal string
            }
            return (None, None)
        };
        (None, None)
    };
    // check if an [attr] has a builder attr
    // returns (ident: Option<syn::Ident>, err: Option<TokenStream>) if either is Some
    // it has a properly formatted builder `if let (Some(_), _) = attr_has_builder(&attrs)`
    let attr_has_builder = |attrs| {
        for attr in attrs {
            let (i, e) = attr_is_builder(attr);
            if i.is_some() || e.is_some() { return (i, e) }
        }
        (None, None)
    };

    // -=-=- impl derive for Builder -=-=- //

    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // ident names for structs
    let name = &input.ident;
    let builder = syn::Ident::new(&format!("{}Builder", name), name.span());

    // get the structs fields to operate on.
    let fields =
    if let syn::Data::Struct(data) = &input.data {
        &data.fields
    } else { panic!() };

    // -=-=- Err Check -=-=- //

    for field in fields {
        // check if there is a parser error 
        if let (_, Some(err)) = attr_has_builder(&field.attrs) {
            return err.into();
        }
    }

    // -=-=- Impl struct -=-=- //

    let impl_fields = fields.iter().map(|f| {
        let name = &f.ident;
        if let (Some(_), _) = attr_has_builder(&f.attrs) {
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

    // -=-=- define struct Builder -=-=- //

    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if let Some(ty) = ty_is_option(ty) {
            quote! { #name: std::option::Option<#ty> }
        } else if let (Some(_), _) = attr_has_builder(&f.attrs) {
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

    // build the Builder functions for setting the full named value for the attr passed.
    let impl_builder_fields = fields.iter().filter_map(|f| {
        // field name
        let name = &f.ident.to_owned().unwrap();
        // field type
        let ty = match ty_is_option(&f.ty) {
            Some(inner_ty) => inner_ty,
            None => f.ty.to_owned()
        };

        // if attr has `builder(each = "...")` and no name conflict then build setter
        if let (Some(ref ident), _) = attr_has_builder(&f.attrs) {
            if ident == name { return None; }
            // -=-=- //
            return Some(quote! {
                fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = #name;
                    self
                }
            });
        }

        // else build generic setter
        Some(quote! {
            fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        })
    });

    // build the object fields for the `build()` function.
    let impl_builder_build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        if ty_is_option(&f.ty).is_some() || attr_has_builder(&f.attrs).0.is_some() {
            quote! {
                #name : self.#name.clone()
            }
        } else {
            quote! {
                #name : self.#name.clone().ok_or(concat!(stringify!(#name), " is not set."))?
            }
        }
    });
    
    // build the extenc methods for all the `builder(each = "...")` attributes.
    let impl_extend_methods = fields.iter().filter_map(|f| {
        for attr in &f.attrs {
            if let (Some(arg), _) = attr_is_builder(&attr) {
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

    // the `build()` function
    let impl_builder_build = quote! {
        pub fn build(&self) -> std::result::Result<#name, std::boxed::Box<dyn std::error::Error>> {
            std::result::Result::Ok(#name{ #( #impl_builder_build_fields ),* })
    }
    };

    // impl the whole builder
    let block_impl_builder = quote! {
        impl #builder {
            #( #impl_builder_fields )*
            #( #impl_extend_methods )*

            #impl_builder_build
        }
    };

    // -=-=- Output -=-=- //

    // Build the output using the other blocks
    let expanded = quote! {
        #block_impl  
        #block_builder
        #block_impl_builder
    };

    // Hand the output tokens back to the compiler
    expanded.into()
}
