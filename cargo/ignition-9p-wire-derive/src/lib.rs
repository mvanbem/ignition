use std::error::Error;
use std::unimplemented;

use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parenthesized, parse_macro_input, parse_quote, Attribute, Data, DeriveInput, Fields,
    GenericParam, Generics, Index, LitStr, Path, Token,
};

#[proc_macro_derive(ReadWireFormat, attributes(ignition_9p_wire))]
pub fn derive_read_wire_format(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let struct_attrs = parse_struct_attrs(&input.attrs);
    let struct_attr_errors = &struct_attrs.errors;
    let read_body = make_read_body(&struct_attrs, &name, &input.data);

    let expanded = quote! {
        #(#struct_attr_errors)*
        impl #impl_generics crate::wire::ReadWireFormat for #name #ty_generics #where_clause {
            fn read_from<R: ::std::io::Read>(r: &mut R) -> ::std::io::Result<Self> {
                #read_body
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(WriteWireFormat, attributes(ignition_9p_wire))]
pub fn derive_write_wire_format(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let struct_attrs = parse_struct_attrs(&input.attrs);
    let struct_attr_errors = &struct_attrs.errors;
    let write_body = make_write_body(&struct_attrs, &input.data);

    let expanded = quote! {
        #(#struct_attr_errors)*
        impl #impl_generics crate::wire::WriteWireFormat for #name #ty_generics #where_clause {
            fn write_to<W: ::std::io::Write>(&self, w: &mut W) -> ::std::io::Result<()> {
                #write_body
            }
        }
    };

    expanded.into()
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(crate::wire::ReadWireFormat))
        }
    }
    generics
}

struct StructAttrs {
    embedded_size_prefix: Option<Path>,
    errors: Vec<AttrError>,
}

fn parse_struct_attrs(attrs: &[Attribute]) -> StructAttrs {
    let mut embedded_size_prefix = vec![];
    let mut errors = vec![];
    for attr in attrs {
        if !path_is_ident(&attr.path, "ignition_9p_wire") {
            continue;
        }
        let key_values: KeyValues = syn::parse2(attr.tokens.clone()).unwrap();
        for key_value in key_values.0 {
            match key_value.key {
                x if x == "embedded_size_prefix" => {
                    let ty: Path = match key_value.value.parse() {
                        Ok(ty) => ty,
                        Err(e) => {
                            errors.push(AttrError::Other(e.into()));
                            continue;
                        }
                    };
                    embedded_size_prefix.push(ty);
                }
                _ => errors.push(AttrError::UnexpectedKey {
                    key: key_value.key.clone(),
                }),
            }
        }
    }
    StructAttrs {
        embedded_size_prefix: {
            let mut iter = embedded_size_prefix.drain(..);
            match iter.next() {
                Some(first) => match iter.next() {
                    Some(_) => {
                        errors.push(AttrError::EmbeddedSizeConflict);
                        None
                    }
                    None => Some(first),
                },
                None => None,
            }
        },
        errors,
    }
}

struct FieldAttrs {
    prefixed: Prefixed,
    errors: Vec<AttrError>,
}
enum Prefixed {
    None,
    Count { ty: Path },
    Size { ty: Path },
}

fn parse_field_attrs(attrs: &[Attribute]) -> FieldAttrs {
    let mut prefixed = vec![];
    let mut errors = vec![];
    for attr in attrs {
        if !path_is_ident(&attr.path, "ignition_9p_wire") {
            continue;
        }
        let key_values: KeyValues = syn::parse2(attr.tokens.clone()).unwrap();
        for key_value in key_values.0 {
            match key_value.key {
                x if x == "count_prefixed" => {
                    let ty: Path = match key_value.value.parse() {
                        Ok(ty) => ty,
                        Err(e) => {
                            errors.push(AttrError::Other(e.into()));
                            continue;
                        }
                    };
                    prefixed.push(Prefixed::Count { ty });
                }
                x if x == "size_prefixed" => {
                    let ty: Path = match key_value.value.parse() {
                        Ok(ty) => ty,
                        Err(e) => {
                            errors.push(AttrError::Other(e.into()));
                            continue;
                        }
                    };
                    prefixed.push(Prefixed::Size { ty });
                }
                _ => errors.push(AttrError::UnexpectedKey {
                    key: key_value.key.clone(),
                }),
            }
        }
    }
    FieldAttrs {
        prefixed: {
            let mut iter = prefixed.drain(..);
            match iter.next() {
                Some(first) => match iter.next() {
                    Some(_) => {
                        errors.push(AttrError::PrefixConflict);
                        Prefixed::None
                    }
                    None => first,
                },
                None => Prefixed::None,
            }
        },
        errors,
    }
}

fn path_is_ident(path: &Path, expected: &str) -> bool {
    match path.get_ident() {
        Some(ident) => ident == expected,
        None => return false,
    }
}

struct KeyValues(Punctuated<KeyValue, Token![,]>);
struct KeyValue {
    key: Ident,
    value: LitStr,
}
impl Parse for KeyValues {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        Ok(KeyValues(Punctuated::parse_terminated(&content)?))
    }
}
impl Parse for KeyValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        let _eq: Token![=] = input.parse()?;
        let value: Literal = input.parse()?;
        let value: LitStr = syn::parse2(value.into_token_stream())?;
        Ok(KeyValue { key, value })
    }
}

enum AttrError {
    UnexpectedKey { key: Ident },
    PrefixConflict,
    EmbeddedSizeConflict,
    Other(Box<dyn Error>),
}
impl ToTokens for &AttrError {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            AttrError::UnexpectedKey { key } => {
                let message = Literal::string(&format!(
                    "unsupported attribute key: {}",
                    key.to_token_stream(),
                ));
                let error = quote! {
                    compile_error!(#message);
                };
                error.to_tokens(tokens);
            }
            AttrError::PrefixConflict => {
                let error = quote! {
                    compile_error!("multiple `count_prefixed` or `size_prefixed` attributes");
                };
                error.to_tokens(tokens);
            }
            AttrError::EmbeddedSizeConflict => {
                let error = quote! {
                    compile_error!("multiple `embedded_size_prefix` attributes");
                };
                error.to_tokens(tokens);
            }
            AttrError::Other(e) => {
                let message = Literal::string(&e.to_string());
                let error = quote! {
                    compile_error!(#message);
                };
                error.to_tokens(tokens);
            }
        }
    }
}

// BUG: Fields named `r` or `w` would shadow the reader or writer and either fail to compile or
// maybe rarely do something unexpected.
fn make_read_body(struct_attrs: &StructAttrs, struct_name: &Ident, data: &Data) -> TokenStream {
    let embedded_length_decl = struct_attrs.embedded_size_prefix.as_ref().map(|ty| {
        quote! {
            let len = <u64 as ::std::convert::TryFrom<_>>::try_from(
                <#ty as crate::wire::ReadWireFormat>::read_from(r)?,
            ).map_err(|_| ::std::io::Error::new(
                ::std::io::ErrorKind::InvalidInput,
                "value too large to represent in memory",
            ))?;
            let r = &mut ::std::io::Read::take(::std::io::Read::by_ref(r), len);
        }
    });
    let embedded_length_check = struct_attrs.embedded_size_prefix.as_ref().map(|_| {
        quote! {
            if r.limit() != 0 {
                return Err(::std::io::Error::new(
                    ::std::io::ErrorKind::InvalidData,
                    // TODO: Break this out into a proper Error type and surface the specific numbers.
                    "unread bytes after length-prefixed value",
                ));
            }
        }
    });

    let (let_statements, build_result) = match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let let_statements = fields.named.iter().map(|f| {
                    let name = f.ident.as_ref().unwrap();
                    let attrs = parse_field_attrs(&f.attrs);
                    let errors = &attrs.errors;
                    match attrs.prefixed {
                        Prefixed::None => quote_spanned! {f.span()=>
                            let #name = crate::wire::ReadWireFormat::read_from(r)?;
                            #(#errors)*
                        },
                        Prefixed::Count { ref ty } => quote_spanned! {f.span()=>
                            let #name = <crate::wire::OwnedCountPrefixedList::<#ty, _> as crate::wire::ReadWireFormat>::read_from(r)?.into_inner();
                        },
                        Prefixed::Size { ref ty } => quote_spanned! {f.span()=>
                            let #name = crate::wire::OwnedSizePrefixed::<#ty, _>::read_from(r)?.into_inner();
                        },
                    }
                }).collect::<Vec<_>>();
                let assignments = fields.named.iter().map(|f| {
                    let name = f.ident.as_ref().unwrap();
                    quote_spanned! {f.span()=>
                        #name,
                    }
                });
                let build_result = quote! { #struct_name { #(#assignments)* } };
                (let_statements, build_result)
            }
            Fields::Unnamed(ref fields) => {
                let let_statements = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let temporary_name = format_ident!("f{}", i);
                        quote_spanned! {f.span()=>
                            let #temporary_name = crate::wire::ReadWireFormat::read_from(r)?;
                        }
                    })
                    .collect();
                let assignments = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let temporary_name = format_ident!("f{}", i);
                    quote_spanned! {f.span()=>
                        #temporary_name,
                    }
                });
                let build_result = quote! { #struct_name( #(#assignments)* ) };
                (let_statements, build_result)
            }
            Fields::Unit => unimplemented!("unit struct fields"),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!("something other than a struct"),
    };

    quote! {
        #embedded_length_decl
        #(#let_statements)*
        #embedded_length_check
        Ok(#build_result)
    }
}

fn make_write_body(struct_attrs: &StructAttrs, data: &Data) -> TokenStream {
    let write_embedded_length = struct_attrs.embedded_size_prefix.as_ref().map(|ty| {
        quote! {
            crate::wire::WriteWireFormat::write_to(
                &<#ty as ::std::convert::TryFrom<_>>::try_from(
                    crate::wire::EmbeddedSize::embedded_size(self),
                )
                .map_err(|_| ::std::io::Error::new(
                    ::std::io::ErrorKind::InvalidInput,
                    "value too large to serailize",
                ))?,
                w,
            )?;
        }
    });

    let write_fields: Vec<TokenStream> = match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                fields.named.iter().map(|f| {
                    let name = f.ident.as_ref().unwrap();
                    let attrs = parse_field_attrs(&f.attrs);
                    let errors = &attrs.errors;
                    let statement = match attrs.prefixed {
                        Prefixed::None => quote_spanned! {f.span()=>
                            crate::wire::WriteWireFormat::write_to(&self.#name, w)?;
                        },
                        Prefixed::Count { ref ty } => quote_spanned! {f.span()=>
                            crate::wire::WriteWireFormat::write_to(&crate::wire::BorrowedCountPrefixedList::<#ty, _>::new(&self.#name), w)?;
                        },
                        Prefixed::Size { ref ty } => quote_spanned! {f.span()=>
                            crate::wire::WriteWireFormat::write_to(&crate::wire::BorrowedSizePrefixed::<#ty, _>::new(&self.#name), w)?;
                        },
                    };
                    quote_spanned! {f.span()=>
                        #statement
                        #(#errors)*
                    }
                }).collect()
            }
            Fields::Unnamed(ref fields) => {
                fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let index = Index::from(i);
                    let attrs = parse_field_attrs(&f.attrs);
                    let errors = &attrs.errors;
                    let statement = match attrs.prefixed {
                        Prefixed::None => quote_spanned! {f.span()=>
                            crate::wire::WriteWireFormat::write_to(&self.#index, w)?;
                        },
                        Prefixed::Count { ref ty } => quote_spanned! {f.span()=>
                            crate::wire::WriteWireFormat::write_to(&crate::wire::BorrowedCountPrefixedList::<#ty, _>::new(&self.#index), w)?;
                        },
                        Prefixed::Size { ref ty } => quote_spanned! {f.span()=>
                            crate::wire::WriteWireFormat::write_to(&crate::wire::BorrowedSizePrefixed::<#ty, _>::new(&self.#index), w)?;
                        },
                    };
                    quote_spanned! {f.span()=>
                        #statement
                        #(#errors)*
                    }
                }).collect()
            }
            Fields::Unit => unimplemented!("unit struct fields"),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!("something other than a struct"),
    };

    quote! {
        #write_embedded_length
        #(#write_fields)*
        Ok(())
    }
}
