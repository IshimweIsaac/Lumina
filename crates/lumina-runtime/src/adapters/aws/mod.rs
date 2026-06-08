pub mod credentials;

#[cfg(feature = "aws-ec2")]
pub mod ec2_adapter;

#[cfg(feature = "aws-s3")]
pub mod s3_adapter;

#[cfg(feature = "aws-rds")]
pub mod rds_adapter;

#[cfg(feature = "aws-lambda")]
pub mod lambda_adapter;

#[cfg(feature = "aws-dynamodb")]
pub mod dynamodb_adapter;

#[cfg(feature = "aws-sqs")]
pub mod sqs_adapter;

#[cfg(feature = "aws-sns")]
pub mod sns_adapter;
