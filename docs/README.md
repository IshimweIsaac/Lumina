# Lumina Documentation Portal

Welcome to the official documentation for **Lumina** — the Distributed Reactive Language for state-driven infrastructure orchestration.

> **Current Version: v2.1.3 — "The Proxmox & Bare-Metal Release"**

---

## 🚀 Quick Links

| Resource | Description |
|---|---|
| [Core Reference](guides/core_reference.md) | Exhaustive guide to Lumina syntax and standard library |
| [Tutorials & Getting Started](guides/tutorials.md) | Quickstarts, installation, and first programs |
| [Language Specification](guides/language_spec.md) | Formal grammar and semantics |
| [Operations & Cluster Guide](guides/operations.md) | Running Lumina in production and distributed mode |
| [The Lumina Book](book.md) | In-depth exploration of the language design |

---

## 🏗️ Version 2: "The Architect"

The current generation of Lumina — systematic replacement of every DevOps tool through infrastructure provisioning adapters.

### Latest Releases
- [v2.1.3 — Proxmox & Bare-Metal Adapter](Version2/v2.1/v2.1.3/index.md) ✅ **CURRENT**
- [v2.1.2 — AWS Adapter Platform](Version2/v2.1/v2.1.2/index.md)
- [v2.1.1 — Docker Adapter](Version2/v2.1/v2.1.1/index.md)

### What's Next
- [v2.1.4 — Adapter Hardening & Multi-Provider](Version2/v2.1/v2.1.4/index.md) ← **NEXT**

---

## 📦 Version 1: "Genesis" (Legacy)

The original single-node reactive engine.

- [Historical Archive (v1.3 – v1.9)](Version1/)

---

## 🧭 Navigation

- [**Full Version Map & Roadmap**](VERSION_MAP.md) — Every planned version from v2.1 through v3.0
- [Developer Guide & Roadmap](guides/developer_guide.md)
- [Contributing Guide](../CONTRIBUTING.md)

---

## 🛠️ For Contributors

If you're contributing to Lumina, start here:

1. Read the [Developer Guide](guides/developer_guide.md) for architecture overview
2. Read the [`.cursorrules`](../.cursorrules) for the strict invariants AI assistants must follow
3. Use `make check` to run formatting, linting, and tests before submitting PRs
4. Use `make release V=x.y.z NAME="..."` to prepare a release (see [SPRINTS.md](../SPRINTS.md))
