<!-- Source: cluster.md -->

# Cluster Deployment Guide

Welcome to the **Lumina Cluster Release**. This guide explains how to transition your Lumina models from a single-node setup into a high-availability, decentralized state mesh.

## 1. Understanding the State Mesh

Lumina introduces the `lumina-cluster` crate, which provides:
*  **Gossip Layer**: Fast, peer-to-peer UDP transport for state broadcasting.
*  **Conflict-Free State**: `StateMesh` automatically handles conflict resolution using Last-Write-Wins (LWW) and logical version vectors.
*  **Orchestration**: Built-in support for workload migration.

When running in cluster mode, your local node evaluates rules and manages instances, and seamlessly syncs them across the network.

## 2. Defining a Cluster Topology

You define a cluster by adding `cluster` blocks to your `.lum` files. Each node needs to know its own identity and the addresses of its peers.

### Example: Node 1 Configuration
Create a file called `node1.lum`:
```lumina
cluster {
  node_id: "node1"
  bind_addr: "127.0.0.1:9101"
  peers: ["node2@127.0.0.1:9102"]
  quorum: 2
}

entity Task {
  status: Text
}

let t1 = Task { status: "pending" }
```

### Example: Node 2 Configuration
Create a file called `node2.lum`:
```lumina
cluster {
  node_id: "node2"
  bind_addr: "127.0.0.1:9102"
  peers: ["node1@127.0.0.1:9101"]
  quorum: 2
}

entity Task {
  status: Text
}
```

## 3. Workload Orchestration

The true power of the Lumina cluster is **Orchestration**. You can write rules that monitor system health and dynamically move workloads.

For example, on `node1.lum`, you could add a rule to evacuate `t1` if the node detects it is degrading:
```lumina
entity HealthMonitor {
  cpu_usage: Number
  is_failing := cpu_usage > 95
}

rule "Evacuate on Failure"
when HealthMonitor.is_failing becomes true {
  -- The migrate command serializes 't1', removes it from node1, 
  -- and re-instantiates it on node2 via the Gossip protocol.
  migrate { workloads: "t1", target: "node2" }
}
```

## 4. Running the Cluster

To run this locally, open two terminal windows:

**Terminal 1:**
```bash
lumina run node1.lum
```

**Terminal 2:**
```bash
lumina run node2.lum
```

You will see the UDP transport initialize:
`[UDP] Transport listening on 127.0.0.1:9101`

When `node1`'s health monitor breaches the threshold, `t1` will seamlessly migrate to `node2` and continue evaluating against `node2`'s local environment.

## 5. Best Practices
*  **Bind Addresses**: For production, ensure your `bind_addr` is set to `0.0.0.0:<port>` to listen on all interfaces.
*  **Quorum**: Always set your quorum to `(N/2) + 1` where N is your total number of nodes. This prevents split-brain scenarios during Raft elections.
*  **Firewalls**: Ensure UDP traffic is allowed between nodes on the specified bind ports.


---

<!-- Source: distribution.md -->

# Lumina: Distribution & Installation 🌍

Lumina is designed to be accessible everywhere. This guide explains how to install the engine and its ecosystem on your preferred platform.

---

## 1. Quick Install (Shell)

For most developers on Linux or macOS, the easiest way to install Lumina is using the official shell script:
```bash
curl -sSL https://lumina-lang.web.app/install.sh | bash
```
This script automatically detects your OS and architecture, fetches the latest binary, and adds it to your path.

---

## 2. Package Managers

### **Homebrew (macOS / Linux)**
We maintain an official Homebrew formula. Use it to keep Lumina updated easily:
```bash
brew tap lumina-lang/tap
brew install lumina
```

### **NPM (JavaScript / Node.js)**
For developers building web or Node.js applications, use the `@lumina-lang/core` package:
```bash
npm install @lumina-lang/core
npx lumina --version
```
This package uses a post-install hook to fetch the correct native binary for your system.

---

## 3. Containerization (Docker)

For reproducible environments or IoT edge deployments, use our official Docker images:
```bash
docker pull luminalang/core:
```
Or use the provided `docker-compose.yml` to start an isolated development environment.

---

## 4. IDE Integration: VS Code

The official **Lumina Extension** provides syntax highlighting, real-time diagnostics, and "Go to Definition" support through the built-in Language Server (LSP).
1. Open VS Code.
2. Search for "Lumina Language" in the Extensions marketplace.
3. Click "Install".
Note: The extension automatically looks for the `lumina` binary in your system path.

---

## 5. Summary: One Engine, Everywhere
Whether you are running on a Raspberry Pi at the edge or a powerful workstation, Lumina provides the tools you need for a seamless developer experience.


---

