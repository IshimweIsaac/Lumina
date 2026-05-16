# Lumina v2.3 — "The Conductor" (Replacing Kubernetes)

## Theme
Full container orchestration at scale. The Lumina runtime becomes a lightweight, reactive alternative to the Kubernetes control plane.

## Sub-Versions

| Version | Focus |
|---------|-------|
| [v2.3.1](v2.3.1/) | Dynamic Fleets — bulk provisioning and auto-scaling |
| [v2.3.2](v2.3.2/) | Scheduling & Placement — constraint-based container placement |
| [v2.3.3](v2.3.3/) | Service Discovery & Load Balancing |
| [v2.3.4](v2.3.4/) | Rolling Updates & Canary Deployments |

## What This Replaces
- Kubernetes (k8s)
- Docker Swarm
- Nomad
- Amazon ECS

## Why Lumina is Better
The existing `lumina-cluster` crate already provides gossip, leader election, and state mesh — replacing etcd and the Kubernetes API server. Combined with the Docker adapter from v2.1, Lumina's reactive rules naturally handle scheduling, scaling, and self-healing without the massive Kubernetes control plane overhead.
