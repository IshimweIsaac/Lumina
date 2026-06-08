# Lumina v2.1.6 — "The Google Cloud (GCP) Adapter Platform"

> **Release Type:** Minor · **Status:** Planned · **Codename:** Multi-Cloud  
> **Minimum Rust Version:** 1.75+ · **Breaking Changes:** None from v2.1.5

---

## Overview

Lumina v2.1.6 introduces the **Google Cloud Platform (GCP) Adapter**, bringing parity with the AWS ecosystem. This release leverages the official `google-cloud-rust` SDKs to deliver fast, async-native orchestration for GCP resources directly from your `.lum` files.

Whether you're deploying VMs on Compute Engine or managing Cloud SQL instances, you can now use Lumina to enforce your GCP desired state without writing YAML or Terraform HCL.

---

## What's New

### 1. Unified GCP Authentication

Similar to the AWS adapter, the GCP adapter automatically resolves credentials securely using the standard `GOOGLE_APPLICATION_CREDENTIALS` environment variable. 

Crucially, Lumina v2.1.6 fully supports **Workload Identity Federation**, allowing Lumina to run securely in GitHub Actions or other CI/CD pipelines without long-lived service account keys.

### 2. The Native GCP Adapters

| Service | Provider Name | Purpose | Key Capabilities |
|---|---|---|---|
| **Compute Engine** | `"gcp-compute"` | Virtual Machines | Provisions GCE instances, handles preemptible/spot configs, and manages attached disks. |
| **Cloud Storage** | `"gcp-storage"` | Object Storage | Manages buckets, IAM policies, and object lifecycle rules. |
| **Cloud SQL** | `"gcp-sql"` | Relational Databases | Provisions PostgreSQL and MySQL instances, managing backups and read replicas. |
| **Cloud Functions** | `"gcp-functions"` | Serverless | Deploys functions directly from source or Cloud Storage zips. |

### 3. Example Usage: GCP Compute & Storage

```lumina
resource entity MLBucket provider "gcp-storage" {
  bucket_name: Text
  location: Text
  storage_class: Text
  
  ensure {
    bucket_name: "lumina-ml-models-v1"
    location: "US-CENTRAL1"
    storage_class: "STANDARD"
  }
}

resource entity MLCompute provider "gcp-compute" {
  instance_name: Text
  machine_type: Text
  zone: Text
  
  depends on MLBucket
  ensure {
    instance_name: "gpu-training-node"
    machine_type: "n1-standard-8"
    zone: "us-central1-a"
  }
}

rule "Provision ML Stack" when true {
  provision MLBucket
  provision MLCompute
}
```

---

## Error Codes

| Code | Description |
|---|---|
| `R020` | Provisioning failure — typically invalid `GOOGLE_APPLICATION_CREDENTIALS` or insufficient IAM permissions. |
| `R021` | Reconciliation failure — drift correction failed. |

---

## Migration from v2.1.5

No breaking changes. 
To enable GCP support, compile Lumina with `--features gcp-full` or select specific services like `--features gcp-compute`.

---

## What's Next

The v2.1.7 release will complete the "Big Three" cloud providers with the **Azure Adapter Platform**.
