# Lumina v3.0.2 — Pipeline Syntax

## Goal
Native build/test/deploy pipelines expressed as Lumina rules. No more YAML pipeline files.

## Deliverables

### 1. Pipeline as Rules
- Build steps as sequential actions within a rule
- Test execution with pass/fail tracked as entity fields
- Artifact management (build outputs tracked as entities)

### 2. Pipeline Visualization
- `lumina pipeline status` CLI command
- Show current stage, history, and failure points

### 3. Conditional Stages
- Skip stages based on file changes (only rebuild what changed)
- Approval gates as Lumina rules (wait for human confirmation before production deploy)

## Example Usage

```lumina
rule "Build Pipeline"
when Repo.commit_hash becomes new {
  write Build.stage to "testing"
  write Build.stage to "deploying"
  deploy(Repo)
  write Build.stage to "complete"
}

rule "Rollback on Failure"
when Build.stage == "deploying" and HealthCheck.status becomes "failing" {
  write Build.stage to "rolling_back"
  deploy(Build.previous_artifact)
}
```

## Dependencies
- v3.0.1 (Git adapter)
