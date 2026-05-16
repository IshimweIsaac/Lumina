# Lumina v2.4.3 — State Persistence (Write-Ahead Log)

## Goal
Implement crash recovery so Lumina can restart without losing state or needing to re-poll all adapters.

## Deliverables
- Write-Ahead Log (WAL) using embedded storage (`sled` or `RocksDB`)
- Every state mutation appended to disk
- On restart, replay WAL to reconstruct the EntityStore instantly
- Configurable retention and compaction

## Dependencies
- Core runtime (v2.0 engine)
