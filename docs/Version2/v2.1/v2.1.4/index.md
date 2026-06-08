# Lumina v2.1.4 — "The Adapter Hardening & Multi-Provider Release"

> **Release Type:** Minor · **Status:** Planned · **Codename:** Hybrid-Cloud  
> **Minimum Rust Version:** 1.75+ · **Breaking Changes:** None from v2.1.3

---

## Overview

As Lumina's ecosystem grows to support multiple cloud and on-premise providers, the complexity of orchestrating them together increases. Lumina v2.1.4 focuses on **Adapter Hardening and Multi-Provider orchestration**. 

This release ensures that the underlying adapter trait can safely handle rate limits, network partitions, and cross-provider dependencies (e.g., waiting for an AWS RDS database to be healthy before starting a Docker container on a Proxmox VM).

---

## What's New

### 1. Cross-Provider Dependency Injection

Lumina's DAG (Directed Acyclic Graph) engine has been upgraded to natively understand cross-provider dependencies via the `depends on` clause inside a `resource entity`.

```lumina
resource entity BackendApp provider "docker" {
  image: "my-app:latest"
  env_vars: Text
  
  -- The engine guarantees the DB is provisioned and healthy FIRST
  depends on ProductionDB
  
  ensure {
    env_vars: "DB_URL={ProductionDB.connection_string}"
  }
}
```

### 2. Built-in Exponential Backoff & Jitter

Provider APIs (like AWS) heavily rate-limit requests. The `LuminaAdapter` trait now implements automatic exponential backoff with jitter for all `poll`, `provision`, `reconcile`, and `destroy` operations.

If AWS throttles a `provision` request, Lumina will gracefully wait and retry without failing the rule evaluation or crashing the DAG.

### 3. Fallback Providers (High Availability)

v2.1.4 introduces the `fallback` provider syntax. If the primary provider fails to provision a resource (e.g., AWS is down in `us-east-1`), Lumina can automatically attempt to provision a fallback resource (e.g., on Proxmox or a different AWS region).

```lumina
resource entity ComputeNode provider "aws-ec2" fallback "proxmox" {
  -- configuration shared or mapped across both providers
}
```

---

## Error Codes

| Code | Description |
|---|---|
| `R022` | Dependency Deadlock — Two resources depend on each other, preventing provisioning. |
| `R023` | Provider Throttling — The adapter has exhausted its maximum retry backoff limit due to extreme API throttling. |

---

## Migration from v2.1.3

No breaking changes. Existing `.lum` files continue to work without modification.
If you are currently relying on implicit timing (e.g. `every 30s`) to orchestrate multi-provider bring-ups, it is highly recommended to migrate to explicit `depends on` declarations for deterministic provisioning.

---

## What's Next

The v2.1.5 release will tackle **Networking Adapters**, introducing native orchestration for DNS (Route53, Cloudflare), Load Balancers, and Firewalls to tie your compute instances together.
