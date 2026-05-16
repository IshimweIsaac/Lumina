# Lumina v2.2.3 — Lumina Agent

## Goal
Replace SSH-based polling with a lightweight daemon that runs on managed nodes and communicates directly with the Lumina cluster via the gossip protocol.

## Deliverables

### 1. Lumina Agent Binary
- Tiny Rust binary (`lumina-agent`) that runs on managed nodes
- Reports system state (CPU, memory, disk, services) directly to the cluster
- Receives and executes commands from the cluster leader

### 2. Gossip Integration
- Agent communicates via the existing UDP gossip protocol from `lumina-cluster`
- No SSH required — faster, more secure, lower overhead
- Auto-discovery: agents find the cluster automatically

### 3. Agent Lifecycle
- Install via single curl command (similar to current Lumina installer)
- Auto-update capability
- Graceful shutdown and cluster deregistration

## Dependencies
- v2.2.1 (SSH adapter as fallback for nodes without agent)
- v2.2.2 (system state entity patterns to report)
