# axum-enum-error

MSRV: 1.65.0

Easily use an enum as an Axum Response type.

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
	#[status_code(BAD_REQUEST)]
	BadRequest(SomeData),
	#[status_code(INTERNAL_SERVER_ERROR)]
	InternalServerError(#[key("error")] String), // 500, body = {"error": STRING}
}
```
