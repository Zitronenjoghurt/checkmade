use metrics::{counter, histogram};
use std::time::Instant;

pub async fn metrics_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(req).await;

    let status = response.status().as_u16().to_string();
    counter!("http.requests_total", "method" => method.clone(), "path" => path.clone(), "status" => status).increment(1);
    histogram!("http.request_duration_secs", "method" => method, "path" => path)
        .record(start.elapsed().as_secs_f64());

    response
}
