use axum::{
	body::Body,
	http::{Response, StatusCode},
	response::IntoResponse,
};
use axum_enum_response::EnumIntoResponse;
use futures::StreamExt;

#[derive(serde::Serialize)]
struct UnauthorizedStruct {
	a: String,
	b: i32,
}

#[derive(EnumIntoResponse)]
enum TestResponse {
	#[status_code(OK)]
	Ok(#[key("aga")] &'static str),
	#[status_code(UNAUTHORIZED)]
	Unauthorized(UnauthorizedStruct),
	#[status_code(INTERNAL_SERVER_ERROR)]
	InternalServerError,
}

async fn get_body(res: Response<Body>) -> String {
	let stream = res.into_body().into_data_stream();
	String::from_utf8(
		stream
			.collect::<Vec<_>>()
			.await
			.into_iter()
			.map(|v| v.unwrap())
			.collect::<Vec<_>>()
			.concat(),
	)
	.unwrap()
}

#[tokio::test]
async fn compiles() {
	{
		let res = TestResponse::Ok("Hello World").into_response();
		assert_eq!(res.status(), StatusCode::OK);

		let body = get_body(res).await;
		assert_eq!(body, "{\"aga\":\"Hello World\"}");
	}

	{
		let res = TestResponse::Unauthorized(UnauthorizedStruct {
			a: "Hi".into(),
			b: 1337,
		})
		.into_response();
		assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

		let body = get_body(res).await;
		assert_eq!(body, "{\"a\":\"Hi\",\"b\":1337}");
	}

	{
		let res = TestResponse::InternalServerError.into_response();
		assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

		let body = get_body(res).await;
		assert_eq!(body, "");
	}
}
