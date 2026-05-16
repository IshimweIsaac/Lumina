# Lumina v2.1.5 — Networking Adapter

## Goal
Complete the infrastructure provisioning layer by adding networking primitives. After this release, Lumina can provision compute, networking, and security — fully replacing Terraform.

## Deliverables

### 1. DNS Adapter
- Manage DNS records (Cloudflare, AWS Route53)
- Create, update, and delete A/AAAA/CNAME records
- Automatic DNS registration for provisioned resources

### 2. Load Balancer Adapter
- Configure load balancers (AWS ALB/NLB, nginx, HAProxy)
- Health-check-based backend pool management
- SSL/TLS certificate provisioning

### 3. Firewall Adapter
- Security group / firewall rule management
- Port whitelisting and IP-based access control

## Example Usage

```lumina
resource entity DNSRecord provider "cloudflare" {
  name: Text
  type: Text
  value: Text
  desired_state: {
    name: "api.example.com"
    type: "A"
    value: web_server.ip_address
  }
}

provision api_dns
```

## Dependencies
- v2.1.1 through v2.1.4
