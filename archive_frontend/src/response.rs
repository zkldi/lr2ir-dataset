use askama::Template;
use axum::{
	http::StatusCode,
	response::{Html, IntoResponse, Response},
};

use crate::templates::ErrorTemplate;

pub fn render<T: Template>(t: T) -> Response {
	match t.render() {
		Ok(html) => Html(html).into_response(),
		Err(e) => {
			tracing::error!(error = %e, "template render failed");
			StatusCode::INTERNAL_SERVER_ERROR.into_response()
		}
	}
}

pub fn error_page(status: StatusCode, title: &str, message: &str) -> Response {
	let t = ErrorTemplate {
		status: status.as_u16(),
		title: title.to_string(),
		message: message.to_string(),
	};
	match t.render() {
		Ok(html) => (status, Html(html)).into_response(),
		Err(_) => status.into_response(),
	}
}

pub fn not_found(message: &str) -> Response {
	error_page(StatusCode::NOT_FOUND, "Not Found", message)
}

pub fn bad_request(message: &str) -> Response {
	error_page(StatusCode::BAD_REQUEST, "Bad Request", message)
}
