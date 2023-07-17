extern crate proc_macro;

use proc_macro::{TokenStream, Ident};
use syn::{parse_macro_input, DeriveInput, Data};
use quote::quote;


#[proc_macro_derive(Storable)]
pub fn derive_storable(input : TokenStream) -> TokenStream{
    let parsed_input: DeriveInput = parse_macro_input!(input);
    let data = parsed_input.data;
    let name = parsed_input.ident;
    let mut enum_token = vec![];
    match data {
        Data::Enum(e) => {
            let mut enum_id = vec![];
            let variants = e.variants;
            for v in variants.clone().into_iter().filter(|v| v.ident != "Null") {
                let name = v.ident;
                let quote = quote! {
                    Data::#name(d) => d.id(),
                };
                enum_id.push(quote);
            }
            let quote = quote! {
                fn id(&self) -> u16 {
                    match self{
                        #(#enum_id)*
                        _ => 0u16
                    }
                }
            };
            enum_token.push(quote);

            let mut enum_uid = vec![];
            for v in variants.clone().into_iter().filter(|v| v.ident != "Null") {
                let name = v.ident;
                let quote = quote! {
                    Data::#name(d) => d.set_uid(uid),
                };
                enum_uid.push(quote);
            }
            let quote = quote! {
                fn set_uid(&mut self, uid : u16){
                    match self{
                        #(#enum_uid)*
                        _ => ()
                    }
                }
            };
            enum_token.push(quote);

            let mut enum_insert = vec![];
            for v in variants.clone().into_iter().filter(|v| v.ident != "Null") {
                let name = v.ident;
                let quote = quote! {
                    Data::#name(d) => d.insert_statement(place),
                };
                enum_insert.push(quote);
            }
            let quote = quote! {
                fn insert_statement(&self, place : String) -> String {
                    match self{
                        #(#enum_insert)*
                        _ => String::from("")
                    }
                }
            };
            enum_token.push(quote);

            let mut enum_value = vec![];
            for v in variants.clone().into_iter().filter(|v| v.ident != "Null") {
                let name = v.ident;
                let quote = quote! {
                    Data::#name(d) => d.value()
                };
                enum_value.push(quote);
            }
            let quote = quote! {
                fn value(&self) -> mysql::params::Params{
                    match self{
                        #(#enum_value,)*
                        _ => params::Params::Empty,
                    }
                }
            };
            enum_token.push(quote);

            let token = quote! {
                impl Storable for #name{
                    #(#enum_token)*
                }
            };

            TokenStream::from(token)

        },
        _ => panic!("Not yet implemented for this type...")
    }

}
