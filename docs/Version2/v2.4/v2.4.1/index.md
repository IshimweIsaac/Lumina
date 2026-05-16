# Lumina v2.4.1 — Deep History (Temporal Ring Buffers)

## Goal
Upgrade the `StateSlot` to hold the last N state changes with timestamps, enabling time-series queries directly in rules.

## Deliverables
- Ring buffer storage per field (configurable depth)
- New query syntax: `server.cpu.avg(5m)`, `server.cpu.max(1h)`
- Flapping detection: `server.status.changes(10m) > 5`
- Rate-of-change calculations

## Dependencies
- Core runtime (v2.0 engine)
