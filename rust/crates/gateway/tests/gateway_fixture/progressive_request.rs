use axum::body::Body;
use axum::http::header::RANGE;
use axum::http::Request;

pub fn video_request(id: &str, range: Option<&str>) -> Request<Body> {
    let builder = Request::builder().uri(format!("/video.mp4?id={id}"));
    let builder = match range {
        Some(value) => builder.header(RANGE, value),
        None => builder,
    };
    builder.body(Body::empty()).expect("request")
}
