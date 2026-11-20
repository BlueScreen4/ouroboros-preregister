# Ouroboros Architecture — Historical Design Notes

Original publication: December 11, 2025. Clarified: November 20, 2026.

The original version of this file described a decentralized approach to pooling fragmented, underutilized hardware. The historical README subsequently described centralized orchestration using a scheduler set. These descriptions conflict if treated as a single implemented architecture. This file preserves the former as a historical design alternative, not a confirmed feature or a claim that a transition was implemented.

## Original decentralized proposal

- Candidate node types: smartphones, legacy desktops, edge devices and GPUs.
- Intended trust-based ranking using reliability, latency and historical result accuracy.
- Intended vendor-neutral execution across AMD, Intel, NVIDIA and ARM systems.
- Proposed self-organizing resource pooling without a central coordinator.
- Proposed flow: register nodes; broadcast tasks for voluntary claim; aggregate results using trust; optionally fall back to higher-performance nodes.

The public scheduler excerpt instead defines a scheduler object with a master identifier, an in-memory node map and candidate ranking. It does not demonstrate decentralized negotiation, trust-weighted result aggregation, cross-vendor runtime support, or a distributed shared-state scheduler cluster.

## Historical centralized proposal

The original README proposed a centralized orchestration pipeline with multiple scheduler instances, specialized model containers and tagged intermediate artifacts. See the [README](../README.md). The public excerpt is insufficient to establish statelessness, coordination, failover or production readiness.

## Intended deployment targets

Edge clusters, on-premises enterprise servers and hybrid consumer networks were proposed targets, not validated deployments.

See [technical limitations](technical-limitations.md) for the observed implementation and evidence boundary. The original wording remains accessible in Git history.
