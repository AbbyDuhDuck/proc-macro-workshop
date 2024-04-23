use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let builder = syn::Ident::new(&format!("{}Builder", name), name.span());

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl #name {
            fn builder() -> #builder {
                #builder {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }

        pub struct #builder {                
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }
        impl #builder {
            fn executable(&mut self, executable: String) -> &mut Self {
                self.executable = Some(executable);
                self
            }
            pub fn args(&mut self, args: Vec<String>) -> &mut Self { 
                self.args = Some(args);
                self
            }
            pub fn env(&mut self, env: Vec<String>) -> &mut Self {
                self.env = Some(env);
                self
            }
            pub fn current_dir(&mut self, current_dir: String) -> &mut Self {
                self.current_dir = Some(current_dir);
                self
            }

            pub fn build(&self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name{
                    executable: self.executable.clone().expect("expected an executable"),
                    args: self.args.clone().expect("expected an args"),
                    env: self.env.clone().expect("expected an env"),
                    current_dir: self.current_dir.clone().expect("expected an current_dir"),
                })
            }
        }
    };

    // Hand the output tokens back to the compiler
    expanded.into()
}
