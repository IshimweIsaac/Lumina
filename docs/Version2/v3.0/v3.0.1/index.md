# Lumina v3.0.1 — Git Adapter

## Goal
Monitor Git repositories as external entities. Code changes become reactive triggers just like any other state change in Lumina.

## Deliverables

### 1. Git Adapter (`git_adapter.rs`)
- New adapter implementing `LuminaAdapter` trait
- Poll Git repositories for commit/push/tag events
- Track `commit_hash`, `branch`, `author`, `message` as entity fields
- Support for GitHub, GitLab, and Bitbucket webhooks

### 2. Reactive Deployments
- Rules trigger on code changes: `when Repo.commit_hash becomes new_hash { ... }`
- Branch-based filtering (only deploy from `main`)

## Example Usage

```lumina
external entity Repo {
  sync_path: "https://github.com/org/app"
  commit_hash: Text
  branch: Text
  author: Text
}

rule "Auto Deploy"
when Repo.branch == "main" and Repo.commit_hash becomes new {
  deploy(Repo)
}
```

## Dependencies
- v2.1 (provisioning to deploy to)
- v2.3 (orchestration for rolling deploys)
