#![allow(unused_variables)]

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse_quote, ItemFn, Stmt};

use crate::server_send::{lower::Field, Ir};

pub type Rust = TokenStream;

pub fn codegen(ir: Ir) -> Rust {

    let Ir { fields, item, level } = ir;

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = item;

    let flds: Vec<syn::ExprAssign> = fields.into_iter().map(|p| p.expr).collect();
    let stmts = &block.stmts;

    syn::parse_quote! {
        #(#attrs)*
        #[cfg_attr(feature = "trace",
            tracing::instrument(level = #level,
                                skip(input),
                                fields( #(#flds ,)*
                                        otel.name           = "Server::parse",

                                )
                        )
                    )]
        #vis #sig {
            #(#stmts)*
        }
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Field { expr } = self;
        // The trace field line: x = 0,
        let stmt: Stmt = parse_quote!(#expr);
        stmt.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_is_function_item() {
        let pq: syn::ExprAssign = parse_quote!(x=0);
        let i = syn::parse2(quote::quote!(
                fn f() {}
            ).into()).expect("ItemFn");
        let f = vec![Field {
                expr: pq
            }];
        let l = syn::parse_quote!(Level::TRACE);
        let ir = Ir {
                fields: f,
                item: i,
                level: l
            };

        let rust = codegen(ir);

        assert!(syn::parse2::<ItemFn>(rust).is_ok());
    }
}