# Lumina v2.3.3 — Service Discovery & Load Balancing

## Goal
Automatic DNS registration and traffic routing for provisioned services.

## Deliverables
- Automatic DNS registration for containers/VMs when provisioned
- Built-in reverse proxy / load balancer adapter
- Health-check-based routing: unhealthy backends removed from pool automatically
- Service mesh awareness across the Lumina cluster

## Dependencies
- v2.1.5 (networking adapter)
- v2.3.1 (fleets)
- v2.3.2 (scheduling)
