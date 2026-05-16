# Lumina v2.2.4 — Template Engine

## Goal
Support config file generation using Lumina's interpolated strings, replacing Ansible's Jinja2 templating.

## Deliverables

### 1. Template Function
- `template(path, context)` built-in that reads a `.lum` template and renders it with entity state
- Uses existing Lumina interpolated string syntax (`{expr}`)

### 2. Config File Push
- `write ConfigFile.content to template("nginx.conf.lum", server)` action
- Automatic hash comparison: only push if content has changed
- Trigger service restart after config change

## Example Usage

```lumina
-- nginx.conf.lum template
-- server {{ server_name: {server.hostname}, port: {server.port} }}

rule "Update Nginx Config"
when server.port becomes new_value {
  write NginxConfig.content to template("nginx.conf.lum", server)
  write NginxService.status to "restarting"
}
```

## Dependencies
- v2.2.1, v2.2.2, v2.2.3
