use proc_macro2::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens};
use syn::punctuated::Punctuated;
use syn::token::{Comma, PathSep};
use syn::{AngleBracketedGenericArguments, Expr, ExprPath, FnArg, GenericArgument, Ident, ItemFn, Pat, PatIdent, PatTuple, PatType, Path, PathArguments, PathSegment, Signature, Type, TypePath, TypeTuple, ExprReference};

pub fn to_bevy_system_fn (input: ItemFn) -> Result<TokenStream, TokenStream> {
    let item_fn = create_fn_item(input)?;

    return Ok(item_fn.into_token_stream());
}

pub fn create_fn_item (input: ItemFn) -> Result<ItemFn, TokenStream> {
    let new_sig = create_new_sig(&input.sig)?;
    let for_loop_stmts = create_new_loop_stmts(&input)?;

    let new_fn = ItemFn {
        attrs: input.attrs,
        vis: input.vis,
        sig: new_sig,
        block: Box::new(syn::Block{
            brace_token: Default::default(),
            stmts: for_loop_stmts
        }),
    };

    return Ok(new_fn);
}

fn create_new_sig(sig: &Signature) -> Result<Signature, TokenStream> {
    let new_sig = Signature {
        inputs: create_new_args(sig)?,
        ..sig.clone()
    };

    Ok(new_sig)
}

fn create_new_args(sig: &Signature) -> Result<Punctuated<FnArg, Comma>, TokenStream> {
    let mut new_args = Punctuated::<FnArg, Comma>::new();

    new_args.push(FnArg::Typed(PatType {
        attrs: vec![],
        pat: Box::new(Pat::Ident(PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: if query_is_mutable(sig) {
                Some(Default::default())
            } else {
                None
            },
            subpat: None,
            ident: Ident::new("query", Span::call_site()),
        })),
        colon_token: Default::default(),
        ty: Box::new(Type::Path(TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: create_query_arg_path_segments(sig)?,
            },
        })),
    }));

    Ok(new_args)
}

fn create_query_arg_path_segments(
    sig: &Signature,
) -> Result<Punctuated<PathSegment, PathSep>, TokenStream> {
    let mut query_arg_path_segments = Punctuated::<PathSegment, PathSep>::new();

    query_arg_path_segments.push(PathSegment{
        ident: Ident::new("bevy_ecs", Span::call_site()),
        arguments: PathArguments::None
    });

    query_arg_path_segments.push(PathSegment{
        ident: Ident::new("prelude", Span::call_site()),
        arguments: PathArguments::None
    });

    query_arg_path_segments.push(PathSegment {
        ident: Ident::new("Query", Span::call_site()),
        arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: Default::default(),
            args: create_query_arg_generic(&sig)?,
            gt_token: Default::default(),
        }),
    });

    return Ok(query_arg_path_segments);
}

fn create_query_arg_generic (sig: &Signature) -> Result<Punctuated<GenericArgument, Comma>, TokenStream> {
    let mut query_arg_generics_tuple = TypeTuple {
        paren_token: Default::default(),
        elems: Punctuated::<Type, Comma>::new(),
    };

    for fn_arg in &sig.inputs {
        query_arg_generics_tuple.elems.push(extract_type(&fn_arg)?);
    }

    let mut query_arg_generic = Punctuated::<GenericArgument, Comma>::new();
    query_arg_generic.push(GenericArgument::Type(Type::Tuple(query_arg_generics_tuple)));

    return Ok(query_arg_generic);
}

fn extract_type (fn_arg: &FnArg) -> Result<Type, TokenStream> {
    match fn_arg {
        FnArg::Receiver(receiver) => {
            Err(syn::Error::new(
                    receiver.self_token.span,
                    "systems can't have a self parameter",
                )
                    .into_compile_error(),
            )
        },
        FnArg::Typed(typed) => Ok(*typed.ty.clone())
    }
}

fn create_new_loop_stmts (input: &ItemFn) -> Result<Vec<syn::Stmt>, TokenStream> {
    let for_loop = create_new_loop(input)?;
    let mut statements = Vec::with_capacity(1);
    statements.push(
        syn::Stmt::Expr(
            syn::Expr::ForLoop(for_loop),
            None
        )
    );

    Ok(statements)
}

fn create_new_loop(input: &ItemFn) -> Result<syn::ExprForLoop, TokenStream> {
    //for <pattern> in <expr> {...}
    let loop_pattern = create_loop_pattern(&input.sig)?;
    let loop_expr = create_loop_expr();

    let for_loop = syn::ExprForLoop {
        attrs: vec![],
        label: None, //what does this field do?
        for_token: Default::default(),
        pat: Box::new(Pat::Tuple(PatTuple {
            attrs: vec![],
            paren_token: Default::default(),
            elems: loop_pattern,
        })),
        in_token: Default::default(),
        expr: Box::new(Expr::Reference(ExprReference{
            attrs: vec![],
            and_token: Default::default(),
            mutability: if query_is_mutable(&input.sig) {
                Some(Default::default())
            } else {
                None
            },
            expr: Box::new(Expr::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: loop_expr,
                },
            }))
        })),
        body: *input.block.clone(),
    };

    Ok(for_loop)
}

fn create_loop_pattern(sig: &Signature) -> Result<Punctuated<Pat, Comma>, TokenStream> {
    let mut for_pat_tuple_elems = Punctuated::<Pat, Comma>::new();
    for fn_arg in &sig.inputs {
        for_pat_tuple_elems.push(extract_pattern(&fn_arg)?);
    }

    Ok(for_pat_tuple_elems)
}

fn extract_pattern(fn_arg: &FnArg) -> Result<Pat, TokenStream> {
    match fn_arg {
        FnArg::Receiver(rec) => Err(syn::Error::new(rec.self_token.span, "systems can't have a self parameter")
                .into_compile_error(),
        ),
        FnArg::Typed(typ) => match &(*typ.pat) {
            Pat::Ident(ident) => Ok(Pat::Ident(PatIdent {
                mutability: if fn_arg_is_mutable(&fn_arg) {
                    Some(Default::default())
                } else {
                    None
                },
                ..ident.clone()
            })),
            _ => Ok(*typ.pat.clone())
        }
    }
}

fn create_loop_expr() -> Punctuated<PathSegment, PathSep> {
    let mut for_expr_segments = Punctuated::<PathSegment, PathSep>::new();
    for_expr_segments.push(PathSegment {
        ident: Ident::new("query", Span::call_site()),
        arguments: PathArguments::None,
    });

    return for_expr_segments;
}

fn query_is_mutable (sig: &Signature) -> bool {
    sig.inputs.iter().any(|fn_arg| {
        fn_arg_is_mutable(fn_arg)
    })
}

fn fn_arg_is_mutable (arg: &FnArg) -> bool {
    match arg {
        FnArg::Receiver(_) => false,
        FnArg::Typed(typed) => {
            //todo it should work without the .clone()
            return if let Type::Reference(refer) = *typed.ty.clone() {
                refer.mutability.is_some()
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use proc_macro2::TokenStream;
    use quote::ToTokens;
    use super::*;

    #[test]
    fn parses_basic_function () {
        let ts = TokenStream::from_str(
            "#[animal_game_ecs_macros::system]
            fn fn_name (a: &A, b: &B, c: &mut C) {}"
        ).unwrap();

        let item_fn = syn::parse2::<syn::ItemFn>(ts).unwrap();

        let result = create_fn_item(item_fn).unwrap();
        let _a = result.into_token_stream().to_string();
        //println!("{}", a);
    }
}