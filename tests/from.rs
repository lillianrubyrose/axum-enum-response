#![allow(dead_code)]

use std::string::FromUtf8Error;

use axum::response::IntoResponse;
use axum_enum_response::EnumIntoResponse;

#[derive(EnumIntoResponse)]
enum TestResponse {
	#[status_code(INTERNAL_SERVER_ERROR)]
	FromUtf8Error(#[from] FromUtf8Error),
}

fn a() -> Result<String, TestResponse> {
	Ok(String::from_utf8("meow".as_bytes().to_vec())?)
}

async fn handler() -> impl IntoResponse {
	a()
}
