# Lumina v2.1.2 — "The AWS Adapter Platform Release"

> **Release Type:** Minor · **Status:** Current Stable · **Codename:** Cloud-Native  
> **Minimum Rust Version:** 1.75+ · **Breaking Changes:** None from v2.1.1

---

## Overview

Lumina v2.1.2 massively expands the **Infrastructure Provisioning Layer** introduced in v2.1.1. While the previous release proved the orchestration concept using local Docker containers, this release elevates Lumina to a full **Cloud Infrastructure-as-Code (IaC)** platform by introducing native support for 7 major Amazon Web Services (AWS).

With the new AWS Adapter Platform, Lumina can provision, reconcile, monitor, and destroy complex cloud architectures—spanning compute, storage, databases, serverless, and messaging—entirely through the declarative `resource entity` syntax and native lifecycle actions (`provision`, `reconcile`, `destroy`).

---

## What's New

### 1. The Shared AWS Credentials Layer

All new AWS adapters share a unified and robust authentication and configuration module (`adapters/aws/credentials.rs`).

- **Secure by Default:** Credentials and configuration are resolved automatically from the environment (e.g., `~/.aws/credentials`, ambient IAM roles, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`) using the official `aws-config::defaults()` behavior.
- **Declarative Overrides:** Regions and other configuration values can be easily overridden directly inside the `desired_state` block of a resource.

### 2. The Native AWS Service Adapters

Lumina v2.1.2 introduces 7 brand-new native adapters to orchestrate AWS resources. Because they use the official strongly-typed `aws-sdk-rust` crates, operations are fast, async-native, and safe.

| Service | Provider Name | Purpose | Key Capabilities |
|---|---|---|---|
| **EC2** | `"aws-ec2"` | Virtual Machines | Provisions instances with dynamic AMI selection, tags, subnet bindings, and security group assignment. |
| **S3** | `"aws-s3"` | Object Storage | Manages storage buckets, including optional enablement of bucket versioning. |
| **RDS** | `"aws-rds"` | Relational Databases | Provisions PostgreSQL, MySQL, etc., with configurable instance classes and allocated storage sizes. |
| **Lambda** | `"aws-lambda"` | Serverless Functions | Provisions and monitors function states and execution roles. |
| **DynamoDB** | `"aws-dynamodb"` | NoSQL Tables | Creates tables with configurable partition/sort keys. Supports both `PROVISIONED` and `PAY_PER_REQUEST` billing modes. |
| **SQS** | `"aws-sqs"` | Message Queues | Provisions standard or `.fifo` queues with adjustable attributes like visibility timeouts and delay seconds. |
| **SNS** | `"aws-sns"` | Notifications | Creates topics and automatically configures protocol subscriptions (e.g. HTTPS, email). |

### 3. Opt-in Compilation & Feature Gates

AWS SDKs are notoriously large. To ensure the core Lumina binary remains lean and blazingly fast for edge and local use cases, the AWS Adapter Platform is strictly controlled via **Cargo Feature Gates**.

- Users can opt-in to specific services to minimize compile times and binary size (e.g., `cargo build --features aws-s3,aws-ec2`).
- Or, compile the entire suite at once using `cargo build --features aws-full`.
- If no AWS features are enabled, Lumina compiles without pulling down any AWS dependencies, completely preventing module leakage.

---

## Example Usage: Multi-Service Architecture

Deploying a database, a queue, and an object storage bucket together is as simple as defining the resources and issuing the `provision` action:

```lumina
-- Define an RDS PostgreSQL Database
resource entity AppDB provider "aws-rds" {
  db_name: Text
  engine: Text
  instance_class: Text
  status: Text
  allocated_storage: Number
  master_username: Text
  master_password: Text
  ensure {
    db_name: "lumina_production_db"
    engine: "postgres"
    instance_class: "db.t4g.micro"
    allocated_storage: 20
    master_username: "postgres"
    master_password: "supersecretpassword123"
  }
}

-- Define a Job Queue
resource entity JobQueue provider "aws-sqs" {
  queue_name: Text
  delay_seconds: Number
  status: Text
  visibility_timeout: Number
  ensure {
    queue_name: "lumina-job-queue"
    delay_seconds: 0
    visibility_timeout: 30
  }
}

-- Define an Asset Bucket
resource entity AssetBucket provider "aws-s3" {
  bucket_name: Text
  aws_region: Text
  status: Text
  versioning: Boolean
  ensure {
    bucket_name: "lumina-app-assets"
    aws_region: "us-east-1"
    versioning: true
  }
}

let db = AppDB { db_name: "lumina_production_db", engine: "postgres", instance_class: "db.t4g.micro", allocated_storage: 20, master_username: "postgres", master_password: "supersecretpassword123", status: "pending" }
let queue = JobQueue { queue_name: "lumina-job-queue", delay_seconds: 0, visibility_timeout: 30, status: "pending" }
let bucket = AssetBucket { bucket_name: "lumina-app-assets", aws_region: "us-east-1", versioning: true, status: "pending" }

rule "Provision Infrastructure" when true {
  provision db
  provision queue
  provision bucket
}
```

---

## Syntax & Best Practices

As the `resource entity` syntax evolves, keep these critical rules in mind when defining your infrastructure:

1. **The `ensure` Keyword**: Target state is defined using the `ensure { }` block, *not* `desired_state: { }`.
2. **Explicit Field Declarations**: Every configuration field you set inside the `ensure` block **must** be declared as a property on the entity (e.g. `allocated_storage: Number`). If it is not declared, the analyzer will throw a `L010` error.
3. **Correct Types**: The boolean type in Lumina is written as `Boolean` (not `Bool`).
4. **Reserved Keywords**: The word `region` is a reserved keyword in Lumina (used for cluster aggregate scoping). For AWS services, you should declare your field as `aws_region: Text`. The AWS adapter automatically understands both!
5. **Action Blocks**: Lifecycle actions like `provision`, `reconcile`, and `destroy` cannot sit loosely at the top level of a file. They must be placed inside a `rule` block with a valid trigger (e.g., `when true`, `every 5m`, or `whenever <condition>`).

---

## Error Codes

The AWS adapters map underlying SDK errors into Lumina's standard orchestration error codes introduced in v2.1.1:

| Code | Description |
|---|---|
| `R020` | Provisioning failure — typically invalid credentials, missing permissions, or invalid configuration fields provided to the AWS SDK. |
| `R021` | Reconciliation failure — drift correction failed. |

---

## Migration from v2.1.1

No breaking changes. Existing `.lum` files continue to work without modification.

To begin using the AWS adapters:
1. Recompile Lumina with the desired AWS features (e.g., `--features aws-full`).
2. Ensure you have valid AWS credentials configured in your environment (`~/.aws/credentials` or ambient environment variables).
3. Use the new `"aws-*"` provider strings in your `resource entity` declarations.

---

## What's Next

The v2.1.3 release will focus on expanding the provisioning layer to on-premise virtualization with a **Proxmox / Bare-Metal Adapter**. See the [Version Map](../../VERSION_MAP.md) for the full roadmap.
