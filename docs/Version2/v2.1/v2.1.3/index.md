# Lumina v2.1.3 — Proxmox / Bare-Metal Adapter

## Goal
Support self-hosted datacenters that don't use cloud providers. Lumina will be able to manage VMs on Proxmox VE, making it viable for on-premise infrastructure.

## Deliverables

### 1. Proxmox Adapter (`proxmox_adapter.rs`)
- New adapter implementing `LuminaAdapter` trait
- VM lifecycle: create, start, stop, destroy via Proxmox REST API
- Template cloning support (create VMs from existing templates)
- Node selection (which Proxmox host to place the VM on)

### 2. Authentication
- API token-based auth via `env("PROXMOX_TOKEN")`
- TLS certificate handling for self-signed Proxmox installations

### 3. Integration Test
- A `.lum` file that clones a VM template, starts it, monitors status, and destroys it

## Example Usage

```lumina
resource entity VM provider "proxmox" {
  template: Text
  node: Text
  cores: Number
  memory: Number
  status: Text
  desired_state: {
    template: "ubuntu-22.04"
    node: "pve-node1"
    cores: 4
    memory: 8192
  }
}

provision db_server
```

## Dependencies
- v2.1.1 (adapter trait)
- v2.1.2 (cloud adapter patterns established)
