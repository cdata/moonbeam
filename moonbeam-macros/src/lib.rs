extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, FnArg, PatType, ReturnType, Token,
};

#[derive(Debug)]
enum Item {
    Fn(syn::ItemFn),
    Unknown,
}

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![fn]) {
            println!("FN");
            input.parse().map(Item::Fn)
        } else {
            Ok(Item::Unknown)
        }
    }
}

#[proc_macro_attribute]
pub fn process(_input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(annotated_item as Item);

    // println!("INPUT: {:#?}", input);

    match input {
        Item::Fn(mut item_fn) => {
            let inputs = item_fn.sig.inputs.clone();

            let outputs = match &item_fn.sig.output {
                ReturnType::Type(_, sig) => Box::new((*sig).clone()),
                _ => panic!("A return type is required!"),
            };

            // TODO: How to handle effect nodes with no return type?
            // let outputs = match &item_fn.sig.output {
            //     ReturnType::Type(_, sig) => (**sig).clone(),
            //     _ => {
            //         let mut punctuated =
            //             syn::punctuated::Punctuated::<syn::PathSegment, syn::token::Colon2>::new();

            //         punctuated.push(syn::PathSegment {
            //             arguments: syn::PathArguments::None,
            //             ident: proc_macro2::Ident::new("()", proc_macro2::Span::call_site()),
            //         });

            //         syn::Type::Path(syn::TypePath {
            //             qself: None,
            //             path: syn::Path {
            //                 leading_colon: None,
            //                 segments: punctuated,
            //             },
            //         })
            //     }
            // };

            let fn_name = item_fn.sig.ident.clone();
            item_fn.sig.ident = syn::Ident::new("implementation", proc_macro2::Span::call_site());

            let generic_params = &item_fn.sig.generics.params;
            let generic_where = &item_fn.sig.generics.where_clause;

            let input_types: Vec<&Box<syn::Type>> = inputs
                .iter()
                .map(|input| match input {
                    FnArg::Typed(pat_type) => match pat_type {
                        PatType {
                            ty,
                            attrs: _,
                            colon_token: _,
                            pat: _,
                        } => ty,
                    },
                    _ => panic!("Self not allowed in stateless functions!"),
                })
                .collect();

            let input_vars = inputs.iter().map(|input| match input {
                FnArg::Typed(pat_type) => match pat_type {
                    PatType {
                        ty: _,
                        attrs: _,
                        colon_token: _,
                        pat,
                    } => pat,
                },
                _ => panic!("Self not allowed in stateless functions!"),
            });

            let call_vars = input_vars.clone();

            let export_name = format!("{}", fn_name.to_string());

            let tokens = quote! {
                #[export_name = #export_name]
                pub fn #fn_name<#generic_params>(mailbox: lunatic::Mailbox<(lunatic::process::Process<#outputs>, (#(#input_types),*))>) #generic_where {

                    #item_fn

                    while let Ok((process, args)) = mailbox.receive() {
                        let (#(#input_vars),*) = args;

                        let result = implementation(#(#call_vars),*);

                        process.send(result);
                    }
                }
            };

            tokens.into()
        }
        _ => TokenStream::new(),
    }
}
