# Lumina v2.2.1 — SSH Adapter

## Goal
Enable Lumina to execute commands on remote hosts over SSH, allowing it to configure machines after provisioning.

## Deliverables

### 1. SSH Adapter (`ssh_adapter.rs`)
- New adapter implementing `LuminaAdapter` trait
- Key-based and credential-based authentication (using `Secret` type)
- Command execution with stdout/stderr capture into entity fields
- File transfer: push config files to remote machines

### 2. Host Entity Pattern
- Standard entity pattern for representing a managed host
- Connection details stored as entity fields

## Example Usage

```lumina
external entity Host {
  sync_path: "ssh://root@192.168.1.10"
  hostname: Text
  uptime: Number
  packages_updated: Boolean
}

rule "Keep Updated"
when Host.packages_updated becomes false {
  write Host.command to "apt update && apt upgrade -y"
}
```

## Dependencies
- v2.1 (infrastructure must be provisionable first)
