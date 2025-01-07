use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::{Datelike, Timelike, Utc};
use colored::{ColoredString, Colorize};

pub async fn request_logger(request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let uri = request.uri().to_string();
    let response = next.run(request).await;
    let now = Utc::now();
    let (is_pm, hour) = now.hour12();

    let current_date = format!(
        "{}/{} {:02}:{:02}:{:02} {}",
        now.day(),
        now.month(),
        hour,
        now.minute(),
        now.second(),
        if is_pm { "PM" } else { "AM" }
    );
    println!(
        "{} - {} {}",
        format!("[{}]", current_date).blue(),
        format!("[{}]", uri).purple(),
        format_status(response.status())
    );

    Ok(response)
}

fn format_status(status: StatusCode) -> ColoredString {
    let status_str = format!("{}", status);

    if status.as_u16() >= 200 && status.as_u16() < 300 {
        return status_str.green();
    } else if status.as_u16() >= 400 {
        return status_str.red();
    }

    return status_str.yellow();
}
