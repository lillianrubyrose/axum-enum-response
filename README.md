# axum-enum-response

MSRV: 1.65.0

Easily create axum::http::Response's from Enums!

# Example Usage
```rs
#[derive(serde::Serialize)]
struct SomeData {
	meow: String,
}

enum ErrorResponse {
   #[status_code(UNAUTHORIZED)]
   Unauthorized, // 401, empty body
   #[status_code(OK)]
   #[body("hello"=>"world")]
   Ok, // 200, body = {"hello": "world"}
   #[status_code(FORBIDDEN)]
   #[body("mew")]
   Forbidden, // 403, body = {"error": "mew"}
   #[status_code(INTERNAL_SERVER_ERROR)]
   FromUtf8Error(#[from] FromUtf8Error), // 500, body = {"error": FromUtf8Error::to_string()}
   #[status_code(INTERNAL_SERVER_ERROR)]
   InternalServerError(#[key("awwa")] String), // 500, body = {"awwa": STRING}
}
```
