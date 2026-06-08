# Lumina v2.1.5 — "The Networking Adapter Release"

> **Release Type:** Minor · **Status:** Planned · **Codename:** Net-Mesh  
> **Minimum Rust Version:** 1.75+ · **Breaking Changes:** None from v2.1.4

---

## Overview

Compute and storage are nothing without connectivity. Lumina v2.1.5 introduces the **Networking Adapter Platform**, extending the Infrastructure Provisioning Layer to domain names, firewalls, and load balancing. 

This release allows you to provision DNS records, manage SSL certificates, and configure load balancers declaratively, fully replacing the need to click around in cloud consoles or use separate Terraform workspaces.

---

## What's New

### 1. DNS Adapters (Route53 & Cloudflare)

Provision A, CNAME, TXT, and ALIAS records directly.

| Provider | Prefix | Supported Records |
|---|---|---|
| **AWS Route53** | `"aws-route53"` | A, AAAA, CNAME, MX, TXT, ALIAS |
| **Cloudflare** | `"cloudflare"` | A, AAAA, CNAME, TXT (including proxy status) |

### 2. Load Balancing (AWS ALB & HAProxy)

Manage layer 7 and layer 4 routing rules.

| Provider | Prefix | Purpose |
|---|---|---|
| **AWS ALB** | `"aws-alb"` | Application Load Balancers, Target Groups, Listeners |
| **HAProxy** | `"haproxy"` | Manages `haproxy.cfg` on bare-metal/Proxmox instances |

### 3. Firewall Rules & Security Groups

Security groups can now be defined as independent `resource entity` objects and attached to compute instances.

---

## Example Usage: Public Web Service

```lumina
-- Define the Security Group
resource entity WebFirewall provider "aws-sg" {
  group_name: "web-public-sg"
  vpc_id: "vpc-12345"
  ensure {
    ingress_tcp: [80, 443]
    egress_all: true
  }
}

-- Define the Server
resource entity WebServer provider "aws-ec2" {
  instance_type: "t3.micro"
  security_group_id: Text
  
  depends on WebFirewall
  ensure {
    security_group_id: WebFirewall.id
  }
}

-- Define the DNS Record pointing to the server
resource entity WebDNS provider "aws-route53" {
  zone_id: "Z123456789"
  record_name: "app.lumina-lang.org"
  record_type: "A"
  ip_address: Text
  
  depends on WebServer
  ensure {
    ip_address: WebServer.public_ip
  }
}

rule "Deploy Entire Stack" when true {
  provision WebFirewall
  provision WebServer
  provision WebDNS
}
```

---

## Syntax & Best Practices

1. **Wait for IP Allocation**: Cloud providers take time to allocate public IPs. Because `WebDNS` depends on `WebServer`, Lumina will automatically wait until `WebServer.public_ip` is fully populated by the AWS API before attempting to create the DNS record.
2. **Cloudflare Proxying**: When using the `cloudflare` provider, setting `proxied: true` in your `ensure` block will automatically route traffic through Cloudflare's CDN and DDoS protection.

---

## Migration from v2.1.4

No breaking changes. Recompile with `--features aws-route53,cloudflare` to enable the new networking features.

---

## What's Next

The v2.1.6 release will expand the cloud provider ecosystem to the **Google Cloud (GCP) Adapter Platform**.
