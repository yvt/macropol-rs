#![doc = include_str!("./lib.md")]
use proc_macro2::{Group, Ident, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::Error;

#[proc_macro_attribute]
pub fn macropol(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut errors = Vec::new();
    let out = transcribe(item.clone().into(), &mut errors).into();
    if errors.is_empty() {
        out
    } else {
        // Output `compile_error!` at the same level as the input. Literals
        // may be inside `macro_rules!`, so if we outputted `compile_error!`
        // there, the errors would be reported in that macro's call site, which
        // would be super confusing.
        let errors = errors
            .into_iter()
            .map(|e| TokenStream::from(e.into_compile_error()));
        let item = TokenStream::from(item);
        proc_macro::TokenStream::from(quote! {
            #(#errors)*
            #item
        })
    }
}

fn transcribe(s: TokenStream, collected_errors: &mut Vec<Error>) -> TokenStream {
    let mut out = Vec::new();
    for tt in s {
        match tt {
            TokenTree::Literal(lit) => {
                // Transcribe if it's a string literal
                if let Ok(s) = syn::parse2(quote! { #lit }) {
                    match transcribe_lit_str(s) {
                        Ok(x) => out.extend(x),
                        Err(e) => {
                            out.push(lit.into());
                            collected_errors.push(e);
                        }
                    }
                } else {
                    out.push(lit.into());
                }
            }
            TokenTree::Group(gr) => {
                out.push(
                    Group::new(gr.delimiter(), transcribe(gr.stream(), collected_errors)).into(),
                );
            }
            _ => out.push(tt),
        }
    }
    TokenStream::from_iter(out)
}

fn transcribe_lit_str(lit_str: syn::LitStr) -> Result<TokenStream, Error> {
    let input = lit_str.value();
    let mut input = &input[..];
    let mut parts = Vec::new();

    enum Part<'a> {
        Input(&'a str),
        Tokens(TokenStream),
    }

    while !input.is_empty() {
        if let Some(i) = input.find("$") {
            parts.push(Part::Input(&input[..i]));
            input = &input[i + 1..];
        } else {
            parts.push(Part::Input(input));
            break;
        }

        if input.is_empty() {
            return Err(Error::new_spanned(
                &lit_str,
                "`$` must be followed by something",
            ));
        }

        if input.starts_with("$") {
            // Output `$` literally
            parts.push(Part::Input(&input[..1]));
            input = &input[1..];
            continue;
        }

        // Should we `stringify!` the variable?
        let should_stringify = if let Some(s) = input.strip_prefix("&") {
            input = s;
            true
        } else {
            false
        };

        // `${ expression... }`
        if input.starts_with("{") {
            input = &input[1..];
            if let Some(i) = input.find("}") {
                let expr = &input[..i];
                input = &input[i + 1..];
                match expr.parse() {
                    Ok(mut tokens) => {
                        if should_stringify {
                            tokens = quote! { ::core::stringify!( # tokens ) };
                        }
                        parts.push(Part::Tokens(tokens));
                    }
                    Err(e) => {
                        return Err(Error::new_spanned(
                            &lit_str,
                            format_args!("could not tokenize `{}`: {:?}", expr, e),
                        ));
                    }
                }
                continue;
            } else {
                return Err(Error::new_spanned(&lit_str, "unclosed `${ ... }`"));
            }
        }

        // Recognize the ASCII subset of `XID_Start XID_Continue*`
        let b = input.as_bytes()[0];
        if !b.is_ascii_alphabetic() && b != b'_' {
            return Err(Error::new_spanned(
                &lit_str,
                "`$` must be followed by `{ ... }` or a valid identifier",
            ));
        }

        let len = input
            .bytes()
            .take_while(|&b| b.is_ascii_alphanumeric() || b == b'_')
            .count();
        let metavar_name = &input[..len];
        input = &input[len..];

        let metavar_name: Ident = match syn::parse_str(metavar_name) {
            Ok(x) => x,
            Err(_) => {
                return Err(Error::new_spanned(
                    &lit_str,
                    format_args!("invalid metavariable name: `{}`", metavar_name),
                ))
            }
        };

        if should_stringify {
            parts.push(Part::Tokens(
                quote! { ::core::stringify!( $ #metavar_name ) },
            ));
        } else {
            parts.push(Part::Tokens(quote! { $ #metavar_name }));
        }
    }

    if parts.len() <= 1 {
        // No change - return `lit_str` as-is
        return Ok(quote! { #lit_str });
    }

    let parts = parts.into_iter().map(|p| match p {
        Part::Input(s) => syn::LitStr::new(s, lit_str.span()).into_token_stream(),
        Part::Tokens(v) => v,
    });

    Ok(quote! { ::core::concat!( #( #parts ),* ) })
}
