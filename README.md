# axum-enum-error

MSRV: 1.65.0

Easily create axum::http::Response's from Enums!

# Example Usage
```rs
#[derive(serde::Serialize)]
struct SomeData {
	meow: String,
}

#[derive(EnumIntoResponse)]
enum ErrorResponse {
	#[status_code(UNAUTHORIZED)]
	Unauthorized, // 401, empty body
	#[status_code(FORBIDDEN)]
	#[message("mew")]
	Forbidden, // 403, body = {"message": "mew"}
	#[status_code(BAD_REQUEST)]
	BadRequest(SomeData),
	#[status_code(INTERNAL_SERVER_ERROR)]
	InternalServerError(#[key("error")] String), // 500, body = {"error": STRING},
}
```
