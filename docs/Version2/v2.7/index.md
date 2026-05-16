# Lumina v2.7 — Parallel ECS Execution

## Goal
Multi-threaded rule evaluation for massive scale. Target: 500,000+ entities per node.

## Deliverables
- Migrate core engine loop to concurrent ECS backend (`bevy_ecs` or `rayon`)
- Lock-free entity store for high-throughput ticks
- Benchmarking suite to validate scale improvements
