# Lumina v2.1.7 — "The Azure Adapter Platform"

> **Release Type:** Minor · **Status:** Planned · **Codename:** Omni-Cloud  
> **Minimum Rust Version:** 1.75+ · **Breaking Changes:** None from v2.1.6

---

## Overview

Lumina v2.1.7 completes the "Big Three" cloud provider triad by introducing the **Microsoft Azure Adapter Platform**. Built entirely on the official `azure_sdk_for_rust`, this release brings robust Azure Resource Manager (ARM) capabilities natively into Lumina.

---

## What's New

### 1. Azure Resource Group Mapping

In Azure, everything must belong to a Resource Group. Lumina streamlines this by allowing you to define a `resource entity` for the Resource Group itself, and natively inheriting it across all dependent resources.

### 2. Native Azure Adapters

| Service | Provider Name | Purpose | Key Capabilities |
|---|---|---|---|
| **Virtual Machines** | `"azure-vm"` | Compute | Provisions VMs, automatically handles NICs and OS disk attachments. |
| **Blob Storage** | `"azure-blob"` | Object Storage | Manages Storage Accounts and Blob Containers. |
| **SQL Database** | `"azure-sql"` | Relational DBs | Manages SQL Server instances and databases. |
| **Cosmos DB** | `"azure-cosmos"` | NoSQL | Provisions globally distributed databases and containers. |

### 3. Identity and Access

Authentication is seamlessly handled via `DefaultAzureCredential`. This means Lumina running locally uses your `az cli` login, while Lumina running in an Azure VM automatically uses the VM's **Managed Identity**.

---

## Example Usage: Azure Resource Group

```lumina
resource entity AppGroup provider "azure-rg" {
  name: Text
  location: Text
  
  ensure {
    name: "lumina-production-rg"
    location: "eastus"
  }
}

resource entity Storage provider "azure-blob" {
  account_name: Text
  resource_group: Text
  
  depends on AppGroup
  ensure {
    account_name: "luminaprodstorage"
    resource_group: AppGroup.name
  }
}

rule "Provision Azure Stack" when true {
  provision AppGroup
  provision Storage
}
```

---

## Migration from v2.1.6

No breaking changes. Enable Azure support during compilation with `--features azure-full`.

---

## What's Next

With the "Big Three" supported, v2.1.8 will shift focus inward to **State Management & Observability**, ensuring complex DAG resolutions and API rate limits are handled gracefully at scale.
