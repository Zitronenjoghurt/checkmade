mod app;
mod http;
mod server_time;
mod store;
mod ui;
mod ws;

pub use app::Checkmade;

#[cfg(target_arch = "wasm32")]
pub fn get_ws_url() -> String {
    let window = web_sys::window().expect("Failed to get browser window");
    let location = window.location();

    let host = location.host().expect("Failed to get host");

    let protocol = location.protocol().unwrap_or_else(|_| "http:".to_string());

    let ws_protocol = if protocol == "https:" { "wss" } else { "ws" };

    format!("{}://{}/ws", ws_protocol, host)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_ws_url() -> String {
    unimplemented!()
}

#[cfg(target_arch = "wasm32")]
pub fn reload() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().reload();
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn reload() {
    std::process::exit(0);
}

pub fn client_time_ms() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
