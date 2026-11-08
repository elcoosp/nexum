//! Attribute macros for the [nexum](https://github.com/elcoosp/nexum)
//! deep-linking toolkit.
//!
//! [`deep_link`] removes the bootstrap ceremony of a nexum application: it
//! builds and validates the `Config` (from `nexum_core`) and runs the
//! platform registration (`nexum_platform::init`) before your `fn main`
//! body executes. Use it through the [`nexum`](https://docs.rs/nexum)
//! facade crate:
//!
//! ```ignore
//! use nexum::deep_link;
//!
//! #[deep_link(schemes = ["myapp"], icon = "/icons/app.png")]
//! fn main() {
//!     // Deep links are registered; `_nexum_handle` holds the consumer
//!     // handle if you need it.
//!     if let Ok(url) = _nexum_handle.recv_blocking() {
//!         println!("opened {url}");
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::Parser;
use syn::{Expr, Ident, ItemFn, Meta, Token, parse_macro_input, punctuated::Punctuated};

/// Renders a `syn::Error` as compiler diagnostics inside the macro output.
fn into_tokens(error: &syn::Error) -> TokenStream {
    error.to_compile_error().into()
}

/// The parsed `#[deep_link(...)]` arguments.
#[derive(Debug, Default)]
struct DeepLinkArgs {
    schemes: Vec<String>,
    app_link_hosts: Vec<String>,
    icon: Option<String>,
}

impl DeepLinkArgs {
    /// Parses `key = value` pairs.
    fn parse(attrs: proc_macro2::TokenStream) -> syn::Result<Self> {
        let mut args = Self::default();
        let pairs = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(attrs)?;
        for pair in pairs {
            let Meta::NameValue(name_value) = pair else {
                return Err(syn::Error::new_spanned(
                    &pair,
                    "expected `key = value`, e.g. `schemes = [\"myapp\"]`",
                ));
            };
            let key = name_value
                .path
                .get_ident()
                .map(Ident::to_string)
                .unwrap_or_default();
            match (key.as_str(), &name_value.value) {
                ("schemes", Expr::Array(array)) => {
                    args.schemes = array
                        .elems
                        .iter()
                        .map(|expr| expect_string(expr, "schemes"))
                        .collect::<syn::Result<_>>()?;
                }
                ("schemes", expr) => {
                    args.schemes = vec![expect_string(expr, "schemes")?];
                }
                ("app_links", Expr::Array(array)) => {
                    args.app_link_hosts = array
                        .elems
                        .iter()
                        .map(|expr| expect_string(expr, "app_links"))
                        .collect::<syn::Result<_>>()?;
                }
                ("app_links", expr) => {
                    args.app_link_hosts = vec![expect_string(expr, "app_links")?];
                }
                ("icon", expr) => {
                    args.icon = Some(expect_string(expr, "icon")?);
                }
                (unknown, _) => {
                    return Err(syn::Error::new_spanned(
                        &name_value.path,
                        format!(
                            "unknown key `{unknown}`; expected `schemes`, `app_links` or `icon`"
                        ),
                    ));
                }
            }
        }
        if args.schemes.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                "`#[deep_link]` needs at least one scheme: `schemes = [\"myapp\"]`",
            ));
        }
        Ok(args)
    }
}

/// Extracts a string literal from an expression.
fn expect_string(expr: &Expr, key: &str) -> syn::Result<String> {
    let Expr::Lit(lit) = expr else {
        return Err(syn::Error::new_spanned(
            expr,
            format!("`{key}` expects string literals"),
        ));
    };
    match &lit.lit {
        syn::Lit::Str(string) => Ok(string.value()),
        other => Err(syn::Error::new_spanned(
            other,
            format!("`{key}` expects string literals"),
        )),
    }
}

/// The `#[deep_link(...)]` attribute: registers deep links before `main`.
///
/// ```text
/// #[deep_link(schemes = ["myapp", "beta"], app_links = ["example.com"], icon = "/icons/app.png")]
/// fn main() { ... }
/// ```
///
/// Expands to the original `fn main` preceded by:
///
/// - a validated `Config` built from the arguments,
/// - `nexum_platform::init(&config)`, whose `DeepLinkHandle` is bound to
///   the variable `_nexum_handle` (leading underscore keeps the binding
///   warning-free when the body does not use it).
///
/// # Panics
///
/// The generated code panics on an invalid configuration or a failed
/// registration — both are unrecoverable startup errors for an
/// application. Use `nexum_platform::init` directly for fallible control.
#[proc_macro_attribute]
pub fn deep_link(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = proc_macro2::TokenStream::from(attrs);
    let function = parse_macro_input!(item as ItemFn);
    match deep_link_impl(attrs, &function) {
        Ok(tokens) => tokens.into(),
        Err(ref error) => into_tokens(error),
    }
}

/// Testable core of [`deep_link`].
fn deep_link_impl(
    attrs: proc_macro2::TokenStream,
    function: &ItemFn,
) -> syn::Result<proc_macro2::TokenStream> {
    if function.sig.ident != "main" {
        return Err(syn::Error::new_spanned(
            &function.sig.ident,
            "`#[deep_link]` can only be applied to `fn main`",
        ));
    }
    let args = DeepLinkArgs::parse(attrs)?;

    let mut builder = quote! { ::nexum::Config::builder() };
    for scheme in &args.schemes {
        builder = quote! { #builder.scheme(#scheme) };
    }
    for host in &args.app_link_hosts {
        builder = quote! { #builder.app_link(::nexum::AppLink::new(#host, [])) };
    }
    let config = if let Some(icon) = &args.icon {
        quote! { #builder.icon_path(#icon).build() }
    } else {
        quote! { #builder.build() }
    };

    let statements = &function.block.stmts;
    let attributes = &function.attrs;
    let visibility = &function.vis;
    let signature = &function.sig;
    Ok(quote! {
        #(#attributes)*
        #visibility #signature {
            let _nexum_handle = {
                let __nexum_config = #config
                    .expect("nexum: invalid deep-link configuration");
                ::nexum::platform::init(&__nexum_config)
                    .expect("nexum: deep-link initialization failed")
            };
            #(#statements)*
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;

    fn expand(attrs: &str, item: &str) -> syn::Result<String> {
        let attrs: proc_macro2::TokenStream = attrs.parse().expect("attr tokens");
        let function: ItemFn = syn::parse_str(item).expect("fn tokens");
        deep_link_impl(attrs, &function).map(|tokens| tokens.to_token_stream().to_string())
    }

    #[test]
    fn injects_config_and_init_into_main() {
        let expanded = expand(
            r#"schemes = ["myapp", "beta"], icon = "/icons/app.png""#,
            "fn main() { println!(\"hi\"); }",
        )
        .expect("expand ok");
        assert!(expanded.contains("Config :: builder"));
        assert!(expanded.contains(". scheme (\"myapp\")"));
        assert!(expanded.contains(". scheme (\"beta\")"));
        assert!(expanded.contains(". icon_path (\"/icons/app.png\")"));
        assert!(expanded.contains("platform :: init"));
        assert!(expanded.contains("_nexum_handle"));
        assert!(expanded.contains("println ! (\"hi\")"));
    }

    #[test]
    fn app_links_map_to_app_link_builder_calls() {
        let expanded = expand(
            r#"schemes = ["myapp"], app_links = ["example.com"]"#,
            "fn main() {}",
        )
        .expect("expand ok");
        assert!(expanded.contains("AppLink :: new (\"example.com\""));
    }

    #[test]
    fn rejects_non_main_functions() {
        let error = expand("schemes = [\"myapp\"]", "fn setup() {}").expect_err("rejected");
        assert!(error.to_string().contains("`fn main`"));
    }

    #[test]
    fn rejects_unknown_keys_and_missing_schemes() {
        let error = expand("colour = [\"red\"]", "fn main() {}").expect_err("rejected");
        assert!(error.to_string().contains("unknown key"));

        let error = expand("icon = \"/x.png\"", "fn main() {}").expect_err("rejected");
        assert!(error.to_string().contains("at least one scheme"));

        let error = expand("", "fn main() {}").expect_err("rejected");
        assert!(error.to_string().contains("at least one scheme"));
    }

    #[test]
    fn rejects_non_string_values_and_bare_idents() {
        let error = expand("schemes = [1, 2]", "fn main() {}").expect_err("rejected");
        assert!(error.to_string().contains("string literals"));

        let error = expand("icon = var", "fn main() {}").expect_err("rejected");
        assert!(error.to_string().contains("`icon` expects string literals"));

        let error = expand("schemes", "fn main() {}").expect_err("rejected");
        assert!(error.to_string().contains("key = value"));
    }
}
