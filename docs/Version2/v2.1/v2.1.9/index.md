# Lumina v2.1.9 — "The Architect Final Polish (LTS)"

> **Release Type:** Minor · **Status:** Planned · **Codename:** The Architect LTS  
> **Minimum Rust Version:** 1.75+ · **Breaking Changes:** None from v2.1.8

---

## Overview

Lumina v2.1.9 is the culmination of the v2.1 series. Having successfully introduced Infrastructure-as-Code capabilities across Docker, Bare-Metal, AWS, GCP, and Azure, this release focuses purely on organization and stability. 

This version introduces **Reusable Modules**, allowing teams to encapsulate and share infrastructure patterns, and serves as the **Long-Term Support (LTS)** foundation before Lumina tackles Configuration Management in v2.2.

---

## What's New

### 1. Reusable Modules

As your `.lum` files grow, maintaining large infrastructure declarations becomes difficult. Lumina v2.1.9 introduces the `module` keyword, heavily inspired by Terraform modules, to allow grouping of resources and rules into parameterized blocks.

```lumina
-- Define a module that spins up a full web stack
module WebStack {
  input instance_type: Text
  input db_size: Number
  
  resource entity WebServer provider "aws-ec2" {
    instance_type: input.instance_type
    -- ...
  }
  
  resource entity Database provider "aws-rds" {
    allocated_storage: input.db_size
    -- ...
  }
  
  rule "Deploy Stack" when true {
    provision WebServer
    provision Database
  }
}

-- Instantiate the module for production
use WebStack as ProductionStack {
  instance_type: "m5.large"
  db_size: 100
}

-- Instantiate the module for staging
use WebStack as StagingStack {
  instance_type: "t3.micro"
  db_size: 20
}
```

### 2. Explicit Broadcast Safety

When rules exist outside of modules and don't bind to a specific instance, they must now be explicitly marked with the `global` keyword. This forces developers to acknowledge that a rule will run globally across the entire mesh, preventing accidental fleet-wide side effects (the "Broadcast Footgun").

### 3. Core Clustering Profiling

The gossip protocol and state mesh have undergone significant profiling, reducing UDP packet sizes and memory footprint by up to 30%, making Lumina even more lightweight for edge deployments.

---

## Error Codes

| Code | Description |
|---|---|
| `L067` | Missing `global` Keyword — A rule attempted to modify global state without the explicit `global` keyword. |
| `L068` | Invalid Module Input — An instantiated module was missing a required `input` parameter. |

---

## Migration from v2.1.8

No breaking changes, except for `L067`. If you have standard rules that attempt to modify un-bound aggregate state without the `global` keyword, the analyzer will now strictly reject them. Update your legacy rules by prefixing them with `global`.

---

## The Future: v2.2 "The Configurator"

With the Architect phase complete, Lumina can now provision any infrastructure. The next major phase, **v2.2 "The Configurator"**, will introduce the ability to run SSH commands, write configuration files, and manage package managers directly inside those provisioned instances—replacing tools like Ansible and Chef.
