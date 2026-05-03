pub mod static_adapter;
pub mod channel;
#[cfg(all(not(target_arch = "wasm32"), not(windows)))]
pub mod mqtt_adapter;
#[cfg(not(target_arch = "wasm32"))]
pub mod http_adapter;
#[cfg(not(target_arch = "wasm32"))]
pub mod file_adapter;
