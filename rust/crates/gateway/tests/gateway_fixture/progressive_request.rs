use axum::body::Body;
use axum::http::header::RANGE;
use axum::http::Request;

pub fn video_request(id: &str, range: Option<&str>) -> Request<Body> {
    request(format!("/video.mp4?id={id}"), range)
}

pub fn capability_request(id: &str, capability: &str, range: Option<&str>) -> Request<Body> {
    request(format!("/video.mp4?id={id}&cap={capability}"), range)
}

fn request(uri: String, range: Option<&str>) -> Request<Body> {
    let builder = Request::builder().uri(uri);
    let builder = match range {
        Some(value) => builder.header(RANGE, value),
        None => builder,
    };
    builder.body(Body::empty()).expect("request")
}
