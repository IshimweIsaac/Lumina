# Lumina v2.4.2 — Metrics Export Adapter

## Goal
Expose Lumina's internal state as scrapeable metrics, enabling gradual migration from existing monitoring stacks.

## Deliverables
- OpenTelemetry-compatible metrics endpoint
- Prometheus `/metrics` scrape endpoint
- Entity field values exposed as labeled metrics
- Aggregate values exported automatically

## Dependencies
- v2.4.1 (deep history for meaningful metrics)
