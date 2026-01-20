//! # OxideKit Portability Macros
//!
//! Procedural macros for marking APIs with portability levels.
//!
//! ## Available Macros
//!
//! - `#[portable]` - Mark an API as fully portable (all targets)
//! - `#[desktop_only]` - Mark an API as desktop-only
//! - `#[web_only]` - Mark an API as web-only
//! - `#[mobile_only]` - Mark an API as mobile-only
//! - `#[target_specific(...)]` - Mark with specific target requirements
//!
//! ## Example
//!
//! ```rust,ignore
//! use oxide_portable_macros::{portable, desktop_only};
//!
//! #[portable]
//! fn calculate_layout() -> Layout {
//!     // This function works on all platforms
//! }
//!
//! #[desktop_only]
//! fn open_file_dialog() -> Option<PathBuf> {
//!     // This only works on desktop
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Attribute, Ident, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Lit, Meta, Token,
};

/// Arguments for portability macros.
struct PortabilityArgs {
    /// Optional category
    category: Option<String>,
    /// Required capabilities
    requires: Vec<String>,
    /// Reason/documentation
    reason: Option<String>,
    /// Alternative API name
    alternative: Option<String>,
}

impl Default for PortabilityArgs {
    fn default() -> Self {
        Self {
            category: None,
            requires: Vec::new(),
            reason: None,
            alternative: None,
        }
    }
}

impl Parse for PortabilityArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = PortabilityArgs::default();

        if input.is_empty() {
            return Ok(args);
        }

        let items: Punctuated<Meta, Token![,]> = Punctuated::parse_terminated(input)?;

        for meta in items {
            match meta {
                Meta::NameValue(nv) => {
                    let name = nv.path.get_ident().map(|i| i.to_string());
                    match name.as_deref() {
                        Some("category") => {
                            if let syn::Expr::Lit(lit) = nv.value {
                                if let Lit::Str(s) = lit.lit {
                                    args.category = Some(s.value());
                                }
                            }
                        }
                        Some("reason") => {
                            if let syn::Expr::Lit(lit) = nv.value {
                                if let Lit::Str(s) = lit.lit {
                                    args.reason = Some(s.value());
                                }
                            }
                        }
                        Some("alternative") => {
                            if let syn::Expr::Lit(lit) = nv.value {
                                if let Lit::Str(s) = lit.lit {
                                    args.alternative = Some(s.value());
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Meta::List(list) => {
                    let name = list.path.get_ident().map(|i| i.to_string());
                    if name.as_deref() == Some("requires") {
                        let caps: Punctuated<Ident, Token![,]> =
                            list.parse_args_with(Punctuated::parse_terminated)?;
                        args.requires = caps.into_iter().map(|i| i.to_string()).collect();
                    }
                }
                Meta::Path(_) => {}
            }
        }

        Ok(args)
    }
}

/// Mark an API as fully portable (works on all targets).
///
/// # Example
///
/// ```rust,ignore
/// #[portable]
/// fn calculate_layout() -> Layout {
///     // Works on desktop, web, and mobile
/// }
///
/// #[portable(category = "ui")]
/// struct Button {
///     label: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn portable(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as PortabilityArgs);
    expand_portability_attribute("Portable", &args, input)
}

/// Mark an API as desktop-only (macOS, Windows, Linux).
///
/// # Example
///
/// ```rust,ignore
/// #[desktop_only]
/// fn open_file_dialog() -> Option<PathBuf> {
///     // Only works on desktop platforms
/// }
///
/// #[desktop_only(reason = "Requires native window APIs")]
/// fn create_system_tray() -> SystemTray {
///     // Desktop-only feature
/// }
/// ```
#[proc_macro_attribute]
pub fn desktop_only(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as PortabilityArgs);
    expand_portability_attribute("DesktopOnly", &args, input)
}

/// Mark an API as web-only (WASM/browser).
///
/// # Example
///
/// ```rust,ignore
/// #[web_only]
/// fn get_dom_element(id: &str) -> Element {
///     // Only works in browser
/// }
/// ```
#[proc_macro_attribute]
pub fn web_only(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as PortabilityArgs);
    expand_portability_attribute("WebOnly", &args, input)
}

/// Mark an API as mobile-only (iOS/Android).
///
/// # Example
///
/// ```rust,ignore
/// #[mobile_only]
/// fn get_device_orientation() -> Orientation {
///     // Only works on mobile devices
/// }
/// ```
#[proc_macro_attribute]
pub fn mobile_only(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as PortabilityArgs);
    expand_portability_attribute("MobileOnly", &args, input)
}

/// Mark an API with specific target requirements.
///
/// # Example
///
/// ```rust,ignore
/// #[target_specific(targets = "macos, linux", requires(filesystem))]
/// fn read_config_file() -> Config {
///     // Only works on macOS and Linux
/// }
///
/// #[target_specific(targets = "ios", reason = "Uses iOS-specific APIs")]
/// fn use_face_id() -> bool {
///     // iOS only
/// }
/// ```
#[proc_macro_attribute]
pub fn target_specific(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as TargetSpecificArgs);
    expand_target_specific_attribute(&args, input)
}

struct TargetSpecificArgs {
    targets: Vec<String>,
    requires: Vec<String>,
    reason: Option<String>,
}

impl Default for TargetSpecificArgs {
    fn default() -> Self {
        Self {
            targets: Vec::new(),
            requires: Vec::new(),
            reason: None,
        }
    }
}

impl Parse for TargetSpecificArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = TargetSpecificArgs::default();

        let items: Punctuated<Meta, Token![,]> = Punctuated::parse_terminated(input)?;

        for meta in items {
            match meta {
                Meta::NameValue(nv) => {
                    let name = nv.path.get_ident().map(|i| i.to_string());
                    match name.as_deref() {
                        Some("targets") => {
                            if let syn::Expr::Lit(lit) = nv.value {
                                if let Lit::Str(s) = lit.lit {
                                    args.targets = s
                                        .value()
                                        .split(',')
                                        .map(|s| s.trim().to_string())
                                        .collect();
                                }
                            }
                        }
                        Some("reason") => {
                            if let syn::Expr::Lit(lit) = nv.value {
                                if let Lit::Str(s) = lit.lit {
                                    args.reason = Some(s.value());
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Meta::List(list) => {
                    let name = list.path.get_ident().map(|i| i.to_string());
                    if name.as_deref() == Some("requires") {
                        let caps: Punctuated<Ident, Token![,]> =
                            list.parse_args_with(Punctuated::parse_terminated)?;
                        args.requires = caps.into_iter().map(|i| i.to_string()).collect();
                    }
                }
                Meta::Path(_) => {}
            }
        }

        Ok(args)
    }
}

fn expand_portability_attribute(
    level: &str,
    args: &PortabilityArgs,
    input: TokenStream,
) -> TokenStream {
    // Try to parse as different item types
    if let Ok(item_fn) = syn::parse::<ItemFn>(input.clone()) {
        return expand_fn_portability(level, args, item_fn);
    }

    if let Ok(item_struct) = syn::parse::<ItemStruct>(input.clone()) {
        return expand_struct_portability(level, args, item_struct);
    }

    if let Ok(item_impl) = syn::parse::<ItemImpl>(input.clone()) {
        return expand_impl_portability(level, args, item_impl);
    }

    if let Ok(item_trait) = syn::parse::<ItemTrait>(input.clone()) {
        return expand_trait_portability(level, args, item_trait);
    }

    if let Ok(item_mod) = syn::parse::<ItemMod>(input.clone()) {
        return expand_mod_portability(level, args, item_mod);
    }

    // If we can't parse it as any known item, just return the input unchanged
    // This allows the attribute to be used on items we don't specifically handle
    input
}

fn expand_fn_portability(level: &str, args: &PortabilityArgs, item: ItemFn) -> TokenStream {
    let doc_attr = generate_portability_doc(level, args);
    let cfg_attr = generate_cfg_attribute(level, args);

    let attrs: Vec<Attribute> = item.attrs.clone();
    let vis = &item.vis;
    let sig = &item.sig;
    let block = &item.block;

    // Add the doc attribute at the start
    let doc_tokens: TokenStream2 = doc_attr.parse().unwrap();

    let output = if let Some(cfg) = cfg_attr {
        let cfg_tokens: TokenStream2 = cfg.parse().unwrap();
        quote! {
            #doc_tokens
            #cfg_tokens
            #(#attrs)*
            #vis #sig #block
        }
    } else {
        quote! {
            #doc_tokens
            #(#attrs)*
            #vis #sig #block
        }
    };

    output.into()
}

fn expand_struct_portability(level: &str, args: &PortabilityArgs, item: ItemStruct) -> TokenStream {
    let doc_attr = generate_portability_doc(level, args);
    let cfg_attr = generate_cfg_attribute(level, args);

    let attrs = &item.attrs;
    let vis = &item.vis;
    let struct_token = &item.struct_token;
    let ident = &item.ident;
    let generics = &item.generics;
    let fields = &item.fields;
    let semi_token = &item.semi_token;

    let doc_tokens: TokenStream2 = doc_attr.parse().unwrap();

    let fields_tokens = match fields {
        syn::Fields::Named(named) => named.to_token_stream(),
        syn::Fields::Unnamed(unnamed) => unnamed.to_token_stream(),
        syn::Fields::Unit => quote! {},
    };

    let semi = semi_token.map(|_| quote! { ; }).unwrap_or_default();

    let output = if let Some(cfg) = cfg_attr {
        let cfg_tokens: TokenStream2 = cfg.parse().unwrap();
        quote! {
            #doc_tokens
            #cfg_tokens
            #(#attrs)*
            #vis #struct_token #ident #generics #fields_tokens #semi
        }
    } else {
        quote! {
            #doc_tokens
            #(#attrs)*
            #vis #struct_token #ident #generics #fields_tokens #semi
        }
    };

    output.into()
}

fn expand_impl_portability(level: &str, args: &PortabilityArgs, item: ItemImpl) -> TokenStream {
    let cfg_attr = generate_cfg_attribute(level, args);

    let attrs = &item.attrs;
    let defaultness = &item.defaultness;
    let unsafety = &item.unsafety;
    let impl_token = &item.impl_token;
    let generics = &item.generics;
    let trait_ = &item.trait_;
    let self_ty = &item.self_ty;
    let items = &item.items;

    let trait_tokens = trait_.as_ref().map(|(bang, path, for_token)| {
        let bang = bang.map(|_| quote! { ! }).unwrap_or_default();
        quote! { #bang #path #for_token }
    });

    let output = if let Some(cfg) = cfg_attr {
        let cfg_tokens: TokenStream2 = cfg.parse().unwrap();
        quote! {
            #cfg_tokens
            #(#attrs)*
            #defaultness #unsafety #impl_token #generics #trait_tokens #self_ty {
                #(#items)*
            }
        }
    } else {
        quote! {
            #(#attrs)*
            #defaultness #unsafety #impl_token #generics #trait_tokens #self_ty {
                #(#items)*
            }
        }
    };

    output.into()
}

fn expand_trait_portability(level: &str, args: &PortabilityArgs, item: ItemTrait) -> TokenStream {
    let doc_attr = generate_portability_doc(level, args);
    let cfg_attr = generate_cfg_attribute(level, args);

    let attrs = &item.attrs;
    let vis = &item.vis;
    let unsafety = &item.unsafety;
    let auto_token = &item.auto_token;
    let trait_token = &item.trait_token;
    let ident = &item.ident;
    let generics = &item.generics;
    let colon_token = &item.colon_token;
    let supertraits = &item.supertraits;
    let items = &item.items;

    let doc_tokens: TokenStream2 = doc_attr.parse().unwrap();

    let supertrait_tokens = if colon_token.is_some() {
        quote! { : #supertraits }
    } else {
        quote! {}
    };

    let output = if let Some(cfg) = cfg_attr {
        let cfg_tokens: TokenStream2 = cfg.parse().unwrap();
        quote! {
            #doc_tokens
            #cfg_tokens
            #(#attrs)*
            #vis #unsafety #auto_token #trait_token #ident #generics #supertrait_tokens {
                #(#items)*
            }
        }
    } else {
        quote! {
            #doc_tokens
            #(#attrs)*
            #vis #unsafety #auto_token #trait_token #ident #generics #supertrait_tokens {
                #(#items)*
            }
        }
    };

    output.into()
}

fn expand_mod_portability(level: &str, args: &PortabilityArgs, item: ItemMod) -> TokenStream {
    let doc_attr = generate_portability_doc(level, args);
    let cfg_attr = generate_cfg_attribute(level, args);

    let attrs = &item.attrs;
    let vis = &item.vis;
    let unsafety = &item.unsafety;
    let mod_token = &item.mod_token;
    let ident = &item.ident;
    let content = &item.content;
    let semi = &item.semi;

    let doc_tokens: TokenStream2 = doc_attr.parse().unwrap();

    let content_tokens = content.as_ref().map(|(_, items)| {
        quote! { { #(#items)* } }
    });

    let semi_tokens = semi.map(|_| quote! { ; });

    let output = if let Some(cfg) = cfg_attr {
        let cfg_tokens: TokenStream2 = cfg.parse().unwrap();
        quote! {
            #doc_tokens
            #cfg_tokens
            #(#attrs)*
            #vis #unsafety #mod_token #ident #content_tokens #semi_tokens
        }
    } else {
        quote! {
            #doc_tokens
            #(#attrs)*
            #vis #unsafety #mod_token #ident #content_tokens #semi_tokens
        }
    };

    output.into()
}

fn expand_target_specific_attribute(args: &TargetSpecificArgs, input: TokenStream) -> TokenStream {
    // Generate cfg based on targets
    let cfg_conditions: Vec<String> = args
        .targets
        .iter()
        .map(|t| match t.as_str() {
            "macos" => "target_os = \"macos\"".to_string(),
            "windows" => "target_os = \"windows\"".to_string(),
            "linux" => "target_os = \"linux\"".to_string(),
            "ios" => "target_os = \"ios\"".to_string(),
            "android" => "target_os = \"android\"".to_string(),
            "web" | "wasm" => "target_arch = \"wasm32\"".to_string(),
            _ => format!("target_os = \"{}\"", t),
        })
        .collect();

    let cfg_str = if cfg_conditions.len() == 1 {
        format!("#[cfg({})]", cfg_conditions[0])
    } else {
        format!("#[cfg(any({}))]", cfg_conditions.join(", "))
    };

    // Parse as function or just return modified
    if let Ok(item_fn) = syn::parse::<ItemFn>(input.clone()) {
        let doc = format!(
            "/// **Target-specific**: Only available on: {}\n",
            args.targets.join(", ")
        );

        let doc_with_reason = if let Some(reason) = &args.reason {
            format!("{}///\n/// {}\n", doc, reason)
        } else {
            doc
        };

        let attrs = &item_fn.attrs;
        let vis = &item_fn.vis;
        let sig = &item_fn.sig;
        let block = &item_fn.block;

        let doc_tokens: TokenStream2 = doc_with_reason.parse().unwrap();
        let cfg_tokens: TokenStream2 = cfg_str.parse().unwrap();

        let output = quote! {
            #doc_tokens
            #cfg_tokens
            #(#attrs)*
            #vis #sig #block
        };

        return output.into();
    }

    // For other items, just add cfg
    let cfg_tokens: TokenStream2 = cfg_str.parse().unwrap();
    let input2: TokenStream2 = input.into();

    let output = quote! {
        #cfg_tokens
        #input2
    };

    output.into()
}

fn generate_portability_doc(level: &str, args: &PortabilityArgs) -> String {
    let mut doc = format!(
        "/// **Portability: {}**\n",
        match level {
            "Portable" => "All platforms",
            "DesktopOnly" => "Desktop only (macOS, Windows, Linux)",
            "WebOnly" => "Web only (WASM)",
            "MobileOnly" => "Mobile only (iOS, Android)",
            _ => level,
        }
    );

    if let Some(category) = &args.category {
        doc.push_str(&format!("///\n/// Category: {}\n", category));
    }

    if !args.requires.is_empty() {
        doc.push_str(&format!(
            "///\n/// Required capabilities: {}\n",
            args.requires.join(", ")
        ));
    }

    if let Some(reason) = &args.reason {
        doc.push_str(&format!("///\n/// {}\n", reason));
    }

    if let Some(alt) = &args.alternative {
        doc.push_str(&format!("///\n/// Alternative: {}\n", alt));
    }

    doc
}

fn generate_cfg_attribute(level: &str, _args: &PortabilityArgs) -> Option<String> {
    match level {
        "DesktopOnly" => Some("#[cfg(not(target_arch = \"wasm32\"))]".to_string()),
        "WebOnly" => Some("#[cfg(target_arch = \"wasm32\")]".to_string()),
        "MobileOnly" => {
            Some("#[cfg(any(target_os = \"ios\", target_os = \"android\"))]".to_string())
        }
        _ => None, // Portable doesn't need cfg
    }
}
