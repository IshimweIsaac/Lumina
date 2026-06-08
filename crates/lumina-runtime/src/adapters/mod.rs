pub mod channel;
#[cfg(not(target_arch = "wasm32"))]
pub mod file_adapter;
#[cfg(not(target_arch = "wasm32"))]
pub mod http_adapter;
#[cfg(all(not(target_arch = "wasm32"), not(windows)))]
pub mod mqtt_adapter;
pub mod static_adapter;
#[cfg(not(target_arch = "wasm32"))]
pub mod docker_adapter;
#[cfg(not(target_arch = "wasm32"))]
pub mod ping_adapter;
#[cfg(not(target_arch = "wasm32"))]
pub mod process_adapter;
#[cfg(all(not(target_arch = "wasm32"), any(feature = "aws-ec2", feature = "aws-s3", feature = "aws-rds", feature = "aws-lambda", feature = "aws-dynamodb", feature = "aws-sqs", feature = "aws-sns")))]
pub mod aws;
#[cfg(not(target_arch = "wasm32"))]
pub mod proxmox_adapter;
