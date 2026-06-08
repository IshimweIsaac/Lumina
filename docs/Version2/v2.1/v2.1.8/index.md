# Lumina v2.1.8 — "State Management & Observability"

> **Release Type:** Minor · **Status:** Planned · **Codename:** Clear-Sight  
> **Minimum Rust Version:** 1.75+ · **Breaking Changes:** None from v2.1.7

---

## Overview

As you start to orchestrate massive hybrid environments (e.g., Docker + AWS + GCP), the complexity of what the engine is doing behind the scenes grows exponentially. Lumina v2.1.8 tackles this complexity by hardening state resolution and providing deep observability into the provisioning DAG.

This release eliminates crashes during massive simultaneous state updates and introduces real-time step-by-step tracing for rule execution.

---

## What's New

### 1. Snapshot Iterators

In earlier versions, if two rules attempted to iterate over the same entity collection while one of them was mutating it (or destroying instances), it could lead to the dreaded `R001` (Concurrent Modification) crash.

Lumina v2.1.8 introduces **Snapshot Iterators**. Now, when a rule begins evaluation or enters a `for` block, the runtime takes a highly optimized, zero-copy snapshot of the state mesh. This guarantees deterministic evaluation and entirely eliminates `R001` crashes.

### 2. Sync Storm Throttling

When hundreds of rules trigger simultaneously (a "Sync Storm"), provider APIs can quickly become overwhelmed, leading to rate limit errors.

The DAG engine now implements **Sync Storm Throttling**. It automatically limits concurrent API calls to a configurable maximum (e.g., `max_concurrent_requests: 50`) and utilizes jitter to space out provisioning requests, protecting both your credentials and the runtime from crashing.

### 3. The `trace` Action

Debugging complex rules is now trivial with the `trace` keyword. When running Lumina with `--trace`, this action prints the value and metadata of any expression directly to the console in real-time, completely bypassing the need for log aggregation just to understand what a rule did.

```lumina
rule "Debug Drift" when AWSNode.status == "drifting" {
  -- This will print the exact instance fields leading to the drift detection
  trace AWSNode
  reconcile AWSNode
}
```

---

## Error Codes

| Code | Description |
|---|---|
| `R024` | Rate Limit Exceeded — The engine was unable to throttle fast enough, and the cloud provider has temporarily locked out your API key. |

---

## Migration from v2.1.7

No breaking changes. Snapshot Iterators are enabled by default, replacing the old iteration model transparently. The `--trace` flag is now required to see `trace` output.

---

## What's Next

The v2.1.9 release serves as the final LTS (Long Term Support) release for the Architect series, focusing on Reusable Modules and final performance polish before moving to Configuration Management in v2.2.
