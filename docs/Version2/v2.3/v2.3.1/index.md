# Lumina v2.3.1 — Dynamic Fleets

## Goal
Introduce a `fleet` keyword for bulk provisioning and dynamic auto-scaling of entity instances.

## Deliverables
- `fleet` keyword in parser/AST
- Fleet sizing: `fleet WebCluster of Container(size: 10)`
- Auto-scaling via rules: `update WebCluster.size = WebCluster.size + 5`
- Fleet-level aggregates: `aggregate ClusterStats over WebCluster { ... }`

## Dependencies
- v2.1 (provisioning adapters)
- v2.2 (configuration management)
