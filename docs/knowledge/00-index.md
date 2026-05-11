# Lumina Knowledge Base — Index

**Version: 2.0.0-GOLD | Status: AI Training Reference**

This directory is the **single source of truth** for writing, debugging, and architecting Lumina programs. It is designed to be read by AI assistants that do NOT have access to the Lumina Rust codebase.

---

## Quick Reference: Which File Do I Need?

| If you need to...                           | Read this file                                                |
|---------------------------------------------|---------------------------------------------------------------|
| Understand what Lumina IS and how to think   | [01-mental-model.md](01-mental-model.md)                      |
| Look up syntax for ANY construct             | [02-syntax-reference.md](02-syntax-reference.md)              |
| Understand types, operators, coercion        | [03-type-system.md](03-type-system.md)                        |
| Find a built-in function signature           | [04-built-in-functions.md](04-built-in-functions.md)          |
| Write or debug a reactive rule               | [05-rules-and-triggers.md](05-rules-and-triggers.md)          |
| See complete, runnable example programs      | [06-patterns-cookbook.md](06-patterns-cookbook.md)              |
| Diagnose an error code (L-codes / R-codes)   | [07-error-encyclopedia.md](07-error-encyclopedia.md)          |
| Use clusters, FFI, external entities, secrets| [08-advanced-features.md](08-advanced-features.md)            |
| Check if something is supported or broken    | [09-known-limitations.md](09-known-limitations.md)            |
| Architect a full project from scratch        | [10-project-templates.md](10-project-templates.md)            |

---

## About Lumina (One Paragraph)

Lumina is a **Distributed Reactive Language (DRL)** for agentless infrastructure orchestration. Instead of writing procedural scripts that describe *how* to reach a state, you write declarative rules that describe *what is true* about your system. The Lumina engine continuously maintains a **Directed Acyclic Graph (DAG)** of state, reacting atomically to changes. If a sensor value changes, every derived field, rule, and aggregate that depends on it is recomputed in a single atomic tick — or rolled back entirely if anything fails.

## File Format

Lumina source files use the `.lum` extension. Comments start with `--`.

## Running Lumina

```bash
# Run a file
lumina run myfile.lum

# Start the interactive REPL
lumina repl

# Run with live temporal triggers (every/for timers tick in real-time)
lumina run --live myfile.lum
```
