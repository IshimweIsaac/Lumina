# Lumina v2.1.1: The Docker Adapter Release (Current)

## Overview
Lumina v2.1.1 marks the official launch of the **Infrastructure Provisioning Layer**. With this release, Lumina ceases to be just a reactive rules engine and becomes a full **Infrastructure-as-Code** orchestrator.

The flagship feature of this release is the native **Docker Adapter** (`docker_adapter.rs`).

## The Docker Adapter
Lumina can now natively manage Docker containers by communicating directly with the local Docker Daemon via Unix sockets (using the `bollard` crate). It completely replaces the need for `docker-compose` or shell scripts.

### Capabilities:
- **Image Pulling:** Automatically pulls required images before provisioning.
- **Port Binding:** Maps host ports to container ports natively.
- **Environment Variables:** Injects secrets and config into containers.
- **Lifecycle Management:** Full support for `poll`, `provision`, `reconcile`, and `destroy` operations.

### The `resource entity` Syntax
This release introduces the `resource entity` keyword, allowing you to declare desired infrastructure state.

```lumina
resource entity WebApp provider "docker" {
  image: Text
  port: Number
  status: Text
  desired_state: {
    image: "nginx:alpine"
    port: 8080
    status: "running"
  }
}

let my_app = WebApp {
  image: "nginx:alpine"
  port: 8080
  status: "pending"
}

// The Lumina engine will now instruct the Docker adapter to create and start the container
provision my_app
```

## Note on Core Features
*All other core Lumina capabilities—including the reactive DAG engine, standard adapters (HTTP, MQTT, File), and basic syntax—were implemented and stabilized in **Version 1 (Genesis)**. Please refer to the `Version1/v1.9/` documentation for details on the core language.*

*The distributed cluster mesh (gossip protocol, leader election) was implemented in **v2.0**.*
