#![warn(clippy::pedantic)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Error, Ident, Meta};

type TokenStream2 = proc_macro2::TokenStream;

#[proc_macro_derive(EnumIntoResponse, attributes(status_code, message))]
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
		let AttributeData { status_code, message } = match parse_attributes(ident, &variant.attrs) {
			Ok(v) => v,
			Err(err) => return Err(err),
		};

		Ok(if let Some(message) = message {
			quote! {
				#enum_name::#ident => (axum::http::StatusCode::#status_code, #message.to_string()),
			}
		} else {
			quote! {
				#enum_name::#ident => (axum::http::StatusCode::#status_code, stringify!(#ident).to_string()),
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
				let (status_code, error_message) = match self {
					#( #match_branches )*
				};

				let body: ::std::collections::HashMap<&str, String> = [("message", error_message)].into();
				(status_code, ::axum::Json(body)).into_response()
			}
		}

		impl ::core::convert::From<#enum_name> for ::axum::response::Response {
			fn from(value: #enum_name) -> ::axum::response::Response {
				let (status_code, error_message) = match value {
					#( #match_branches )*
				};

				let body: ::std::collections::HashMap<&str, String> = [("message", error_message)].into();
				::axum::response::IntoResponse::into_response((status_code, ::axum::Json(body)))
			}
		}
	};

	Ok(output.into())
}

struct AttributeData {
	status_code: TokenStream2,
	message: Option<TokenStream2>,
}

fn parse_attributes(ident: &Ident, attributes: &Vec<Attribute>) -> syn::Result<AttributeData> {
	if attributes.is_empty() {
		return Err(Error::new_spanned(
			ident,
			"You must specify the 'status_code' attribute",
		));
	}

	let mut status_code = None;
	let mut message = None;

	for attribute in attributes {
		let Some(iden) = attribute.path().get_ident() else {
			return Err(Error::new_spanned(ident, "You must name attributes"));
		};

		match iden.to_string().as_str() {
			"status_code" => {
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
			"message" => {
				if let Meta::List(list) = &attribute.meta {
					let tokens = &list.tokens;
					message = Some(quote! {
						#tokens
					});
				} else {
					return Err(Error::new_spanned(attribute, "Invalid usage of 'message' attribute"));
				}
			}
			_ => {}
		}
	}

	let Some(status_code) = status_code else {
		return Err(Error::new_spanned(ident, "'status_code' attribute must be specified"));
	};

	Ok(AttributeData { status_code, message })
}
