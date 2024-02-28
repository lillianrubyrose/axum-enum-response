use axum::{http::StatusCode, response::IntoResponse};
use axum_enum_response::EnumIntoResponse;

#[derive(Debug, EnumIntoResponse)]
enum TestError {
	#[status_code(INTERNAL_SERVER_ERROR)]
	Ise,
	#[status_code(BAD_REQUEST)]
	BadReq,
	#[status_code(OK)]
	#[message("HELLO")]
	Ok,
}

#[test]
fn compiles() {
	{
		let res = TestError::Ise.into_response();
		assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
	}

	{
		let res = TestError::BadReq.into_response();
		assert_eq!(res.status(), StatusCode::BAD_REQUEST);
	}

	{
		let res = TestError::Ok.into_response();
		assert_eq!(res.status(), StatusCode::OK);
	}
}
