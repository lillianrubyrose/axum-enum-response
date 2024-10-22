//! Easily create axum::http::Response's from Enums!
//! MSRV: 1.65.0
//!
//! # Example Usage
//! ```
//! #[derive(axum_enum_response::EnumIntoResponse)]
//! enum ErrorResponse {
//!    #[status_code(UNAUTHORIZED)]
//!    Unauthorized, // 401, empty body
//!    #[status_code(OK)]
//!    #[body("hello"=>"world")]
//!    Ok, // 200, body = {"hello": "world"}
//!    #[status_code(FORBIDDEN)]
//!    #[body("mew")]
//!    Forbidden, // 403, body = {"error": "mew"}
//!    #[status_code(INTERNAL_SERVER_ERROR)]
//!	   FromUtf8Error(#[from] FromUtf8Error), // 500, body = {"error": FromUtf8Error::to_string()}
//!    #[status_code(INTERNAL_SERVER_ERROR)]
//!    InternalServerError(#[key("awwa")] String), // 500, body = {"awwa": STRING}
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
use syn::{
	parse::Parse, parse_macro_input, Attribute, Data, DeriveInput, Error, Ident, LitStr, Meta, Result, Token, Type,
};

type TokenStream2 = proc_macro2::TokenStream;

#[proc_macro_derive(EnumIntoResponse, attributes(status_code, body, key, from))]
pub fn enum_into_response(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	match impl_enum_into_response(input) {
		Ok(tokens) => tokens,
		Err(err) => err.into_compile_error().into(),
	}
}

fn impl_enum_into_response(input: DeriveInput) -> Result<TokenStream> {
	let enum_name = input.ident;
	let Data::Enum(data_enum) = input.data else {
		return Err(Error::new_spanned(
			enum_name,
			"You may only use 'EnumIntoResponse' on enums",
		));
	};

	let (match_branches, impls) = data_enum.variants.into_iter().map(|variant| {
		let ident = &variant.ident;
		let field_attributes = parse_field_attributes(&variant.fields)?;
		let VariantAttributes { status_code, body } = parse_attributes(ident, &variant.attrs)?;

		let match_branches = if let Some(FieldAttributes { key, from_ty }) = &field_attributes {
			if from_ty.is_some() {
				if let Some(key) = key {
					quote! {
						#enum_name::#ident(v) => (::axum::http::StatusCode::#status_code, Some(::axum::Json(::std::collections::HashMap::from([(#key, v.to_string())])).into_response())),
					}
				} else {
					quote! {
						#enum_name::#ident(v) => (::axum::http::StatusCode::#status_code, Some(::axum::Json(::std::collections::HashMap::from([("error", v.to_string())])).into_response())),
					}
				}
			} else if let Some(key) = key {
				quote! {
					#enum_name::#ident(v) => (::axum::http::StatusCode::#status_code, Some(::axum::Json(::std::collections::HashMap::from([(#key, v)])).into_response())),
				}
			} else {
				quote! {
					#enum_name::#ident(v) => (::axum::http::StatusCode::#status_code, Some(::axum::Json(v).into_response())),
				}
			}
		} else if let Some(BodyAttribute { key, value }) = body {
			let key = key.unwrap_or_else(|| "error".to_string());
			quote! {
				#enum_name::#ident => (::axum::http::StatusCode::#status_code, Some(::axum::Json(::std::collections::HashMap::from([(#key, #value)])).into_response())),
			}
		} else {
			quote! {
				#enum_name::#ident => (::axum::http::StatusCode::#status_code, None),
			}
		};

		Result::Ok((match_branches, if let Some(FieldAttributes { from_ty: Some(ty), .. }) = field_attributes {
			Some(quote! {
			impl From<#ty> for #enum_name {
				fn from(value: #ty) -> Self {
					Self::#ident(value)
				}
			}
			})
		} else {
			None
		}))
	}).collect::<Result<(Vec<_>, Vec<_>)>>()?;

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

		#( #impls )*
	};

	Ok(output.into())
}

struct FieldAttributes {
	key: Option<TokenStream2>,
	from_ty: Option<Type>,
}

fn parse_field_attributes(fields: &syn::Fields) -> Result<Option<FieldAttributes>> {
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

	let mut key = None;
	let mut from_ty = None;

	for attribute in &field.attrs {
		let Some(iden) = attribute.path().get_ident() else {
			return Err(Error::new_spanned(attribute, "You must name attributes"));
		};

		match iden.to_string().as_str() {
			"key" => {
				if let Meta::List(list) = &attribute.meta {
					let tokens = &list.tokens;
					key = Some(quote! {
						#tokens
					});
				} else {
					return Err(Error::new_spanned(attribute, "'key' attribute value must be a string"));
				}
			}

			"from" => {
				from_ty = Some(field.ty.clone());
			}

			_ => {}
		}
	}

	Ok(Some(FieldAttributes { key, from_ty }))
}

struct VariantAttributes {
	status_code: TokenStream2,
	body: Option<BodyAttribute>,
}

struct BodyAttribute {
	key: Option<String>,
	value: String,
}

impl Parse for BodyAttribute {
	fn parse(input: syn::parse::ParseStream) -> Result<Self> {
		let first = input.parse::<LitStr>()?;
		let mut second: Option<LitStr> = None;

		if input.peek(Token![=>]) {
			input.parse::<Token![=>]>()?;
			second = Some(input.parse::<LitStr>()?);
		}

		if let Some(value) = second {
			Ok(Self {
				key: Some(first.value()),
				value: value.value(),
			})
		} else {
			Ok(Self {
				key: None,
				value: first.value(),
			})
		}
	}
}

fn parse_attributes(ident: &Ident, attributes: &Vec<Attribute>) -> Result<VariantAttributes> {
	if attributes.is_empty() {
		return Err(Error::new_spanned(
			ident,
			"You must specify the 'status_code' attribute",
		));
	}

	let mut status_code = None;
	let mut body = None;

	for attribute in attributes {
		let Some(iden) = attribute.path().get_ident() else {
			return Err(Error::new_spanned(ident, "You must name attributes"));
		};

		match iden.to_string().as_str() {
			"status_code" => {
				status_code = Some(attribute.meta.require_list()?.tokens.clone());
			}

			"body" => {
				body = Some(attribute.meta.require_list()?.parse_args::<BodyAttribute>()?);
			}

			_ => {}
		}
	}

	let Some(status_code) = status_code else {
		return Err(Error::new_spanned(ident, "'status_code' attribute must be specified"));
	};

	Ok(VariantAttributes { status_code, body })
}
