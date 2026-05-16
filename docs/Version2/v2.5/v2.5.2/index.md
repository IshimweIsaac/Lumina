# Lumina v2.5.2 — Alert Routing & Escalation

## Goal
Route alerts to the right people based on severity, time of day, and on-call schedules.

## Deliverables
- Severity-based routing (info → Slack, critical → SMS)
- On-call schedules as Lumina entities
- Alert deduplication and grouping
- Escalation chains (if not acknowledged in 10m, escalate)

## Dependencies
- v2.5.1 (notification adapters)
