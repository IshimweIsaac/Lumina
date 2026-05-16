# Lumina v2.2.2 — System State Entities

## Goal
Define OS-level resources (packages, services, config files) as Lumina entities with desired states, enabling continuous configuration enforcement.

## Deliverables

### 1. Standard System Entity Patterns
- `Package` entity: name, version, desired_state ("installed"/"absent")
- `SystemService` entity: name, desired_state ("running"/"stopped"/"enabled")
- `ConfigFile` entity: path, content hash, desired content

### 2. Reconciliation Loop
- If a service is manually stopped, Lumina detects the drift and restarts it
- If a package is removed, Lumina reinstalls it
- If a config file is modified, Lumina overwrites it with the desired version

## Example Usage

```lumina
resource entity NginxService provider "systemd" {
  name: Text
  status: Text
  desired_state: { status: "running" }
}

rule "Self-Healing Nginx"
when NginxService.status becomes "stopped" {
  reconcile NginxService
  alert severity: "warning", message: "Nginx was down, auto-restarted"
}
```

## Dependencies
- v2.2.1 (SSH adapter for remote execution)
