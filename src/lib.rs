//! Simple way to use an enum as an Axum Response
//! MSRV: 1.65.0
//!
//! # Example Usage
//! ```
//! #[derive(axum_enum_response::EnumIntoResponse)]
//! enum ErrorResponse {
//!     #[status_code(UNAUTHORIZED)]
//!     Unauthorized, // 401, empty body
//!     #[status_code(INTERNAL_SERVER_ERROR)]
//!     InternalServerError(#[key("error")] String), // 500, body = {"error": STRING}
//! }
//! ```
//!
//! You can also use any struct that implements `serde::Serialize` as a field like this:
//! ```no_run
//! #[derive(serde::Serialize)]
//! struct SomeData {
//!     meow: String,
//! }
//!
//! #[derive(axum_enum_response::EnumIntoResponse)]
//! enum ErrorResponse {
//!     #[status_code(BAD_REQUEST)]
//!     BadRequest(SomeData), // 400, body = {"meow": STRING}
//! }
//! ```
//!

#![warn(clippy::pedantic)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Error, Ident, Meta};

type TokenStream2 = proc_macro2::TokenStream;

#[proc_macro_derive(EnumIntoResponse, attributes(status_code, key))]
pub fn enum_into_response(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	match impl_enum_into_response(input) {
		Ok(tokens) => tokens,
		Err(err) => err.into_compile_error().into(),
	}
}

fn impl_enum_into_response(input: DeriveInput) -> syn::Result<TokenStream> {
	let enum_name = input.ident;
	let Data::Enum(data_enum) = input.data else {
		return Err(Error::new_spanned(
			enum_name,
			"You may only use 'EnumIntoResponse' on enums",
		));
	};

	let match_branches = data_enum.variants.into_iter().map(|variant| {
		let ident = &variant.ident;
		let body_field = parse_fields(&variant.fields)?;
		let AttributeData { status_code } = parse_attributes(ident, &variant.attrs)?;

		syn::Result::Ok(if let Some(body_field) = body_field {
			if let Some(key) = body_field.json_key {
				quote! {
					#enum_name::#ident(v) => (::axum::http::StatusCode::#status_code, Some(::axum::Json(::std::collections::HashMap::from([(#key, v)])).into_response())),
				}
			} else {
				quote! {
					#enum_name::#ident(v) => (::axum::http::StatusCode::#status_code, Some(::axum::Json(v).into_response())),
				}
			}
		} else {
			quote! {
				#enum_name::#ident => (::axum::http::StatusCode::#status_code, None),
			}
		})
	});

	for result in match_branches.clone() {
		result?;
	}

	let match_branches = match_branches.filter_map(Result::ok).collect::<Vec<_>>();
	let output = quote! {
		impl ::axum::response::IntoResponse for #enum_name {
			fn into_response(self) -> ::axum::response::Response {
				let (status_code, body): (::axum::http::StatusCode, Option<::axum::response::Response>) = match self {
					#( #match_branches )*
				};

				let Some(body) = body else {
					return status_code.into_response();
				};

				(status_code, body).into_response()
			}
		}

		impl ::core::convert::From<#enum_name> for ::axum::response::Response {
			fn from(value: #enum_name) -> ::axum::response::Response {
				::axum::response::IntoResponse::into_response(value)
			}
		}
	};

	Ok(output.into())
}

struct FieldData {
	json_key: Option<TokenStream2>,
}

fn parse_fields(fields: &syn::Fields) -> syn::Result<Option<FieldData>> {
	let mut fields = fields.iter();
	let Some(field) = fields.next() else {
		return Ok(None);
	};

	if field.ident.is_some() {
		return Err(syn::Error::new_spanned(
			field,
			"EnumIntoResponse only supports unnamed fields.",
		));
	}

	if let Some(field) = fields.next() {
		return Err(syn::Error::new_spanned(
			field,
			"EnumIntoResponse only supports up to one unnamed field.",
		));
	}

	let mut json_key = None;

	for attribute in &field.attrs {
		let Some(iden) = attribute.path().get_ident() else {
			return Err(Error::new_spanned(attribute, "You must name attributes"));
		};

		if let "key" = iden.to_string().as_str() {
			if let Meta::List(list) = &attribute.meta {
				let tokens = &list.tokens;
				json_key = Some(quote! {
					#tokens
				});
			} else {
				return Err(Error::new_spanned(attribute, "'key' attribute value must be a string"));
			}
		}
	}

	Ok(Some(FieldData { json_key }))
}

struct AttributeData {
	status_code: TokenStream2,
}

fn parse_attributes(ident: &Ident, attributes: &Vec<Attribute>) -> syn::Result<AttributeData> {
	if attributes.is_empty() {
		return Err(Error::new_spanned(
			ident,
			"You must specify the 'status_code' attribute",
		));
	}

	let mut status_code = None;

	for attribute in attributes {
		let Some(iden) = attribute.path().get_ident() else {
			return Err(Error::new_spanned(ident, "You must name attributes"));
		};

		if let "status_code" = iden.to_string().as_str() {
			if let Meta::List(list) = &attribute.meta {
				let tokens = &list.tokens;
				status_code = Some(quote! {
					#tokens
				});
			} else {
				return Err(Error::new_spanned(
					attribute,
					"Invalid usage of 'status_code' attribute",
				));
			}
		}
	}

	let Some(status_code) = status_code else {
		return Err(Error::new_spanned(ident, "'status_code' attribute must be specified"));
	};

	Ok(AttributeData { status_code })
}
