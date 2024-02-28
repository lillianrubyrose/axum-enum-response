use axum::{
	body::Body,
	http::{Response, StatusCode},
	response::IntoResponse,
};
use axum_enum_response::EnumIntoResponse;
use futures::StreamExt;

#[derive(EnumIntoResponse)]
enum TestResponse {
	#[status_code(INTERNAL_SERVER_ERROR)]
	#[message("InternalServerError")]
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
async fn no_fields() {
	{
		let res = TestResponse::InternalServerError.into_response();
		assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

		let body = get_body(res).await;
		assert_eq!(body, "{\"message\":\"InternalServerError\"}");
	}
}
