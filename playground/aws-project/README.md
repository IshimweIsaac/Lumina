# Lumina AWS Playground Project

This project demonstrates how Lumina can orchestrate a multi-service AWS architecture natively.

## Prerequisites
1. You must compile Lumina with AWS features enabled:
   ```bash
   cargo build --features aws-full
   ```
2. You must have valid AWS credentials configured in your environment. Lumina automatically uses your default AWS profile or environment variables (e.g. `~/.aws/credentials`).

## Included Resources
- **PostgreSQL Database** (`aws-rds`)
- **SQS Job Queue** (`aws-sqs`)
- **S3 Asset Bucket** (`aws-s3`)

## Running

To provision this infrastructure and start monitoring it, run:
```bash
lumina run main.lum
```

> **Warning:** Running this will actually provision resources in your AWS account! AWS charges may apply. When you're done testing, you can modify `main.lum` to `destroy` the resources or delete them manually through the AWS console.
