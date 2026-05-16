# Lumina v2.1.4 — Adapter Hardening & Multi-Provider

## Goal
Make the adapter system production-ready. Support running multiple adapters of the same type and add robust error handling.

## Deliverables

### 1. Multi-Provider Support
- Support multiple adapters of the same type (e.g., 3 Docker hosts, 2 AWS regions)
- Adapter instance identification and routing
- Provider selection in `resource entity` declarations

### 2. Error Recovery
- Retry logic with exponential backoff for failed adapter operations
- Circuit breaker pattern: stop hammering a failing provider
- Graceful degradation when an adapter is unreachable

### 3. Adapter Health Monitoring
- Each adapter exposes its own health status as an entity
- Rules can react to adapter failures (e.g., failover to a different provider)

### 4. Provider Registry Groundwork
- `lumina provider install <name>` fetches and registers adapter plugins
- Local adapter manifest file for tracking installed providers

## Dependencies
- v2.1.1, v2.1.2, v2.1.3 (concrete adapters to harden)
