# Lumina v2.3.2 — Scheduling & Placement

## Goal
Constraint-based scheduling to intelligently place containers on nodes with available resources.

## Deliverables
- Resource tracking (CPU, memory) as entity fields on cluster nodes
- Affinity/anti-affinity rules (keep services together or apart)
- Resource limits enforcement
- Automatic placement decisions by the cluster leader

## Dependencies
- v2.3.1 (fleets to schedule)
