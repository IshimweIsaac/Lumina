# Lumina Developer Guide

This document covers two areas for developers working on the Lumina project:
1. **Future Roadmap** — planned language and engine features
2. **Publishing Workflows** — how to publish the VS Code extension

For architecture, invariants, and contribution rules, see [CONTRIBUTING.md](../../CONTRIBUTING.md).

---

## Future Roadmap

These features were identified during the Lumina hardening phase to push Lumina toward enterprise-scale orchestration. They are tracked in the [Version Map](../VERSION_MAP.md).

### Dynamic Fleets & Auto-Scaling Groups (v2.3)

**The Problem:**
Currently, users must manually declare every entity instance (e.g., `let worker_1`, `let worker_2`) to provision them. While the `create` action exists, it is difficult to dynamically reference and manage auto-generated instances.

**The Solution:**
Introduce a `fleet` keyword to natively support arrays of entities for bulk provisioning and dynamic load balancing.

```lumina
// Proposed Syntax
fleet AppCluster of LSL::docker::Container(size: 100) {
 image: "nginx:alpine"
 port: 0
 // Other fields...
}

// Dynamically scaling the fleet
global rule "Scale Up" whenever traffic > 80% {
 update AppCluster.size = AppCluster.size + 10
}
```

### Default Values & Optional Fields (v2.8)

**The Problem:**
Lumina strictly requires every stored field to be initialized during entity creation (to prevent `Value::Unknown` crashes). This leads to boilerplate when using large schemas like `LSL::docker::Container`, forcing users to write helper functions as constructors.

**The Solution:**
Add language-level support for safe zero-values or explicit default assignments in schemas.

```lumina
// Proposed Syntax
resource entity Container provider "docker" {
 image: Text
 port: Number = 0     // Explicit default
 status: Text = "unknown"
}
```

### State Persistence — Write-Ahead Log (v2.4)

**The Problem:**
Lumina's `EntityStore` is entirely in-memory. If a node loses power and restarts, its reactive state is wiped blank. It must perform expensive polling across all adapters to rebuild its view of the world.

**The Solution:**
Implement a lightweight **Write-Ahead Log (WAL)** using embedded storage (e.g., `sled` or `RocksDB`). Every state mutation will be appended to the disk. On restart, the engine will instantly replay the WAL to reconstruct the exact `EntityStore` state without blind polling.

### Deep History — Temporal Ring Buffers (v2.4)

**The Problem:**
The `StateSlot` currently only holds `current` and `previous` values. It cannot evaluate long-term trends natively.

**The Solution:**
Upgrade the `StateSlot` to act as a ring buffer holding the last `N` state changes with timestamps. This enables powerful time-series queries directly in rules.

```lumina
// Proposed Syntax
rule "Flapping Detection" whenever container.status.restarted_times(5m) > 3 {
 alert "Container is crashing rapidly!"
}
```

### Parallel ECS Execution (v2.7)

**The Problem:**
The engine evaluates rules and updates the `EntityStore` sequentially (or behind a giant Mutex). At 500,000+ entities, a single CPU core becomes a bottleneck.

**The Solution:**
Migrate the core engine loop to a true concurrent ECS (Entity Component System) backend (like Rust's `bevy_ecs` or `specs`). This will allow the engine to evaluate thousands of rules simultaneously across multiple CPU cores without lock contention, massively increasing the engine's "tick" frequency.

---

## Publishing the VS Code Extension

The `.vsix` package is generated in `extensions/lumina-vscode/`. These are the steps to publish it to the VS Code Marketplace.

### Prerequisites
- A **Microsoft Account**.
- An **Azure DevOps organization** (to generate a Personal Access Token).
- A **Publisher ID** (set as `luminalang` in our `package.json`).

### Generate a Personal Access Token (PAT)
1. Go to [dev.azure.com](https://dev.azure.com/) and sign in.
2. Click on the **User Settings** icon (top right) → **Personal Access Tokens**.
3. Create a **New Token**.
4. Set the name to `lumina-vsce`.
5. Under **Organization**, select `All accessible organizations`.
6. Under **Scopes**, select `Custom defined`.
7. Scroll down to **Marketplace** and check `Publish`.
8. **Copy your token immediately**. You won't see it again!

### Create a Publisher
If you haven't already:
1. Go to the [VS Code Marketplace Management Console](https://marketplace.visualstudio.com/manage).
2. Create a new publisher with the ID: `luminalang`.

### Method A: Command Line (Fastest)
Run the following commands in `extensions/lumina-vscode`:

```bash
# Login to vsce with your PAT
npx @vscode/vsce login luminalang

# Publish the extension
npx @vscode/vsce publish
```

### Method B: Visual Upload (Easiest for First Time)
1. Go to the [Management Console](https://marketplace.visualstudio.com/manage).
2. Click on your `luminalang` publisher.
3. Click **New Extension** → **Visual Studio Code**.
4. Drag and drop your `.vsix` file.
5. Review the details and click **Upload**.

### Verification
Once uploaded, the extension will undergo a brief automated verification. It usually goes live within a few minutes. You can then search for "Lumina" in the VS Code Extensions view!
