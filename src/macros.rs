#[macro_export]
macro_rules! simple_response {
    ($status: expr, $body: expr) => {
        axum::response::Response::builder()
            .status($status)
            .body($body)
            .expect("simple response cant fail")
    };
}
