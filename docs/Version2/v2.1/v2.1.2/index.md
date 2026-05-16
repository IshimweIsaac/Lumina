# Lumina v2.1.2 — AWS EC2 Adapter

## Goal
Extend the adapter ecosystem from Docker to cloud infrastructure. Lumina will be able to provision, monitor, and destroy AWS EC2 instances using the same `resource entity` syntax.

## Deliverables

### 1. AWS Adapter (`aws_adapter.rs`)
- New adapter implementing `LuminaAdapter` trait
- EC2 instance lifecycle: launch, terminate, describe
- Security group management (basic)
- VPC/subnet selection via `desired_state` fields

### 2. Credential Handling
- Use the existing `env("AWS_ACCESS_KEY_ID")` built-in for secure credential access
- Support AWS region configuration via `desired_state`

### 3. Integration Test
- A `.lum` file that provisions an EC2 instance, monitors its health via HTTP adapter, and destroys it on shutdown

## Example Usage

```lumina
resource entity WebServer provider "aws-ec2" {
  instance_type: Text
  region: Text
  status: Text
  desired_state: {
    instance_type: "t3.large"
    region: "us-east-1"
  }
}

let web1 = WebServer {
  instance_type: "t3.large"
  region: "us-east-1"
  status: "pending"
}

provision web1
```

## Dependencies
- v2.1.1 (Docker Adapter pattern established)
