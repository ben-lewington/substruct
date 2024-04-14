use std::collections::HashMap;

use proc_macro::{self, TokenStream};
use proc_macro2::{Ident, TokenTree};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input,
    punctuated::{Pair, Punctuated},
    token::Comma,
    Attribute, Data, DataStruct, DeriveInput, Error, Expr, ExprLit, Field, Fields, FieldsNamed,
    Lit, MacroDelimiter, Meta, MetaList, MetaNameValue,
};

type Result<T, E = Error> = core::result::Result<T, E>;

#[proc_macro_derive(SubStruct, attributes(parts))]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive = parse_macro_input!(input);

    match process_substruct(derive) {
        Ok(t) => t,
        Err(e) => e.into_compile_error().into(),
    }
}

fn parse_parts_attr(a: &Attribute) -> Result<HashMap<Ident, (Vec<Field>, Vec<Field>)>> {
    let Meta::List(MetaList {
        path: _,
        delimiter: MacroDelimiter::Paren(_),
        tokens: t,
    }) = a.meta.clone()
    else {
        return Err(Error::new_spanned(a, "expected #[parts(Sub1, .., SubN)]"));
    };

    let idents = t.into_iter().filter_map(|tt| {
        let TokenTree::Ident(ident) = tt else {
            return None;
        };
        Some((ident, (Vec::<Field>::new(), Vec::<Field>::new())))
    });

    if idents.clone().count() == 0 {
        return Err(Error::new_spanned(
            a,
            "at least one prefix required: expected #[parts(Sub1, .., SubN)]",
        ));
    }

    Ok(idents.collect())
}

fn process_substruct(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        vis,
        attrs,
        generics,
        data,
        ident,
    } = input;

    let (impl_generics, ty_generics, where_clause) = dbg!(generics.clone().split_for_impl());

    let Data::Struct(DataStruct {
        struct_token: _,
        fields:
            Fields::Named(FieldsNamed {
                brace_token: _,
                named: fields,
            }),
        semi_token: None,
    }) = data
    else {
        return Err(Error::new_spanned(
            ident,
            "Substruct can only be derived on structs with named fields",
        ));
    };

    let mut parts = match attrs
        .iter()
        .find(|a| a.path().is_ident("parts"))
        .map(parse_parts_attr)
    {
        Some(Ok(p)) => p,
        Some(Err(e)) => {
            return Err(e);
        }
        _ => {
            return Err(Error::new_spanned(
                ident,
                "To correctly derive SubStruct, you need a parts attr: #[parts(S1,..,SN))]",
            ));
        }
    };

    // 00 | f0 | f1 | f2 |
    // p0 | t  | f  | f  |
    // p1 | t  | t  | f  |
    // p2 | f  | t  | t  |

    for field in fields {
        // let f_id = field.clone().ident.unwrap().to_string();
        if let Some(parts_attr) = field.attrs.iter().find(|a| a.path().is_ident("parts")) {
            // eprint!("field {} has a parts attr", f_id);
            // validate parts attr
            let Attribute {
                meta:
                    Meta::NameValue(MetaNameValue {
                        value:
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(ps), ..
                            }),
                        ..
                    }),
                ..
            } = parts_attr
            else {
                return Err(Error::new_spanned(
                    field,
                    "Expected #[parts = \"S1,..,Sk\"], where each S<i> is specified in the top level list",
                ));
            };

            // eprintln!(" {}", &ps.token());
            // walk the contents of the attr
            let mut f_part_idents: Vec<Ident> = Vec::with_capacity(parts.keys().len());

            for c in ps.value().split(',').map(|c| c.trim()) {
                // TODO: wrapper type with a PartialEq impl that supports entry for the keys
                let Some(pk) = parts.keys().find(|i| i.to_string().to_lowercase() == c) else {
                    return Err(Error::new_spanned(
                        field,
                        format!(
                            "specified part value {} was not present in top level parts",
                            c
                        ),
                    ));
                };

                f_part_idents.push(pk.clone());
            }


            for (k, (fs, cofs)) in parts.iter_mut() {
                if f_part_idents.contains(k) {
                    // eprintln!("push {} into {}Sut", f_id, &k.to_string());
                    fs.push(field.clone());
                } else {
                    // eprintln!("push {} into Co{}Sut", f_id, &k.to_string());
                    cofs.push(field.clone());
                }
            }
        } else {
            // eprintln!("{} has a no parts attr", f_id);
            // if a field does not have a parts attr, it should be included in all complements
            for (_, (_, cofs)) in parts.iter_mut() {
                // eprintln!("push into Co{}Sut", &k.to_string());
                cofs.push(field.clone());
            }
        };
    }

    let part_structs: Vec<_> = parts
        .into_iter()
        .map(|(prefix, (fs, cofs))| {
            let s_name = format_ident!("{}{}", prefix, ident);
            let cos_name = format_ident!("Co{}{}", prefix, ident);

            let fs: Punctuated<Field, Comma> = fs
                .into_iter()
                .map(
                    |Field {
                         attrs: _,
                         vis,
                         mutability,
                         ident,
                         colon_token,
                         ty,
                     }| {
                        Pair::new(
                            Field {
                                attrs: vec![],
                                vis,
                                mutability,
                                ident,
                                colon_token,
                                ty,
                            },
                            Some(Comma::default()),
                        )
                    },
                )
                .collect();

            let cofs: Punctuated<Field, Comma> = cofs
                .into_iter()
                .map(
                    |Field {
                         attrs: _,
                         vis,
                         mutability,
                         ident,
                         colon_token,
                         ty,
                     }| {
                        Pair::new(
                            Field {
                                attrs: vec![],
                                vis,
                                mutability,
                                ident,
                                colon_token,
                                ty,
                            },
                            Some(Comma::default()),
                        )
                    },
                )
                .collect();

            quote! {
                #vis struct #s_name #generics {
                    #fs
                }
                #vis struct #cos_name #generics {
                    #cofs
                }
            }
        })
        .collect();

    Ok(quote! {
        #(#part_structs)*
    }
    .into())
}
