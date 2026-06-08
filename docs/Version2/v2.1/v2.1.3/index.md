# Lumina v2.1.3 — "The Proxmox & Bare-Metal Release"

> **Release Type:** Minor · **Status:** Planned · **Codename:** Edge-Native  
> **Minimum Rust Version:** 1.75+ · **Breaking Changes:** None from v2.1.2

---

## Overview

Following the AWS Adapter Platform in v2.1.2, Lumina v2.1.3 brings the **Infrastructure Provisioning Layer** back on-premise. This release introduces native orchestration for **Proxmox Virtual Environment (VE)** and bare-metal environments, making Lumina the perfect orchestrator for self-hosted datacenters and edge deployments.

With this update, you can declare, clone, start, and monitor Proxmox VMs alongside your Docker containers and AWS resources in a single `.lum` file.

---

## What's New

### 1. The Native Proxmox Adapter

Lumina v2.1.3 introduces the `proxmox_adapter.rs`, a native Rust implementation that communicates directly with the Proxmox REST API. It bypasses the need for intermediary terraform-providers and communicates securely via TLS.

**Capabilities:**

| Feature | Details |
|---|---|
| **VM Cloning** | Provisions new Virtual Machines by rapidly cloning existing Proxmox templates. |
| **Hardware Configuration** | Declaratively sets `cores`, `memory`, and `disk_size` during or after cloning. |
| **Node Placement** | Maps VMs to specific physical Proxmox nodes within a datacenter cluster. |
| **Lifecycle Actions** | Fully supports `provision` (create/start), `reconcile` (detect drift), and `destroy` (stop/remove). |
| **Secure Auth** | Supports API Tokens via `PROXMOX_TOKEN_ID` and `PROXMOX_TOKEN_SECRET`, alongside native handling for self-signed certificates. |

### 2. Bare-Metal Execution Entities

In addition to VMs, v2.1.3 introduces the concept of `BareMetal` entities. These entities can track physical hardware statuses via Redfish / IPMI sensors, enabling rules that provision VMs only when physical nodes are confirmed healthy.

---

## Example Usage: Self-Hosted Datacenter

```lumina
-- Define a Proxmox Virtual Machine
resource entity DatabaseVM provider "proxmox" {
  node: Text
  template: Text
  cores: Number
  memory: Number
  status: Text
  
  ensure {
    node: "pve-storage-node-01"
    template: "ubuntu-22.04-base"
    cores: 8
    memory: 16384
  }
}

let db_server = DatabaseVM {
  node: "pve-storage-node-01",
  template: "ubuntu-22.04-base",
  cores: 8,
  memory: 16384,
  status: "pending"
}

-- Provision the VM, but only if the host is cool
rule "Deploy On-Prem DB" 
when HostSensors.temperature < 60 {
  provision db_server
}

-- Reconcile drift every 5 minutes
rule "Drift Correction" every 5m {
  reconcile db_server
}
```

---

## Syntax & Best Practices

1. **Authentication Handling**: Never hardcode your Proxmox API tokens in the `.lum` file. Lumina will automatically securely read `PROXMOX_URL`, `PROXMOX_TOKEN_ID`, and `PROXMOX_TOKEN_SECRET` from the environment or a `.env` file.
2. **The `ensure` Block**: Remember that `ensure` defines the target state. If a Proxmox administrator manually changes the `memory` of a VM in the web UI, the `reconcile` action will automatically resize it back during the next tick.
3. **Self-Signed Certificates**: Set `PROXMOX_INSECURE_TLS=true` in your environment if your Proxmox node does not have a valid Let's Encrypt certificate.

---

## Error Codes

| Code | Description |
|---|---|
| `R020` | Provisioning failure — typically invalid Proxmox credentials, a missing template, or insufficient node resources. |
| `R021` | Reconciliation failure — drift correction failed, often due to the VM being locked by another Proxmox task. |

---

## Migration from v2.1.2

No breaking changes. Existing `.lum` files continue to work without modification.
To use the Proxmox features:
1. Recompile Lumina with the `proxmox` feature gate: `cargo build --features proxmox`.
2. Provide your API credentials in the environment.

---

## What's Next

The v2.1.4 release will focus on **Adapter Hardening & Multi-Provider Architectures**, allowing seamless dependency mapping between AWS resources, Proxmox VMs, and local Docker containers.
