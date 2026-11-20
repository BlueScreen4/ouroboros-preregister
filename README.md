# 🌀 Ouroboros — Distributed Offload Inference Framework

**Historical technical disclosure / pre-registration — December 11, 2025**  
**Editorial clarification — November 20, 2026**

This public repository preserves the author's early Ouroboros design and scheduler source. The proposed system aims to orchestrate heterogeneous computing resources, including mobile devices, desktops and accelerators, for offloaded AI tasks and specialized container execution.

This is an engineering design record, not a formal patent registration, scientific performance claim, or assertion that the underlying concepts were invented here. The original publication and subsequent corrections remain in Git history. The private implementation is outside the scope of this public repository.

## Scope and evidence

- The original architecture is a proposal. Features described below are intended capabilities unless explicitly identified as present in the published source.
- The preserved [scheduler excerpt](stc_scheduler.rs) is an early, incomplete prototype, not a standalone buildable or validated inference framework.
- No public benchmark dataset, controlled comparison, reproducible end-to-end test suite, security assessment, or hardware compatibility matrix accompanies this disclosure.
- The initial [architecture notes](docs/architecture.md) described decentralized pooling, whereas the subsequent original README described centralized orchestration. These are historical design descriptions, not two simultaneously established properties of an implemented system.
- For the exact published scoring rules and their limitations, see [Technical limitations and evidence status](docs/technical-limitations.md).

## Original architecture proposal

| Component | Intended function |
|---|---|
| Phi-3 Mini client frontline | Attempt local inference and request offload when appropriate. |
| Poison Protocol + gRPC + TLS 1.3 | Carry structured request metadata and context over a proposed TLS-protected transport. The additional protocol is not asserted to provide cryptographic security. |
| Mistral interpreter | Translate requests into structured task plans and container chains. |
| MFPI-driven scheduler set | Rank eligible nodes according to a proposed set of resource metrics; multiple scheduler instances and shared state were proposed, not demonstrated. |
| M.2 AI container chain | Package specialized models in SSD-backed modules with dynamic activation and deactivation. Physical live M.2 insertion/removal, device-level hot-plug safety and reliable automatic unmounting are not demonstrated. |
| Tagged buffer routing | Transfer intermediate artifacts between stages through proposed routing metadata and RAM or SSD-backed storage. Exact tag schemas, isolation and throughput are not established by the public excerpt. |

Model names and execution technologies above belong to the historical proposal, not a guarantee of current dependencies, performance or support.

## OPI and MFPI resource evaluation

Multi-factor resource scoring, heterogeneous scheduling, and network- and power-aware offloading are established technical approaches. The project-specific disclosure here consists of the author's proposed metric selection, scoring rules, node selection and integration with the proposed container execution pipeline.

The original MFPI proposal contemplated PCIe capacity, memory bandwidth, power state, latency, thermal/load conditions and accelerator/runtime compatibility. The published code is narrower: it computes a weighted raw OPI score, adjusts it for ROCm availability, and applies network/load factors and health-state exclusions.

The numeric coefficients, thresholds and backend preferences are provisional engineering heuristics, not measured performance ratios. Only the ROCm multiplier of 1.1 is present in this public excerpt; it does not establish a 10% speed advantage. Other backend multipliers are not established by this public code. RAM amount, PCIe generation number and compute-unit count are not directly interchangeable measures of inference throughput.

No claim is made that these metrics, their combination, or the broader architecture are first-of-kind, globally optimal, or superior to existing approaches. Patent novelty, inventive step and comparative performance have not been established.

## Proposed execution flow

1. A lightweight client evaluates whether to request offload.
2. The client sends a structured request to the server.
3. An interpreter proposes a task plan and container sequence.
4. The scheduler evaluates eligible resources and selects candidates according to its configured scoring policy.
5. Specialized containers are intended to activate as needed and exchange intermediate artifacts through a tagged buffer.
6. The final result is intended to return through the interpreter to the client.

This sequence describes design intent. The public scheduler excerpt does not establish successful end-to-end execution, distributed sharding, inference, secure transport or orchestration.

## Deployment intent

Potential deployments included on-premises servers, heterogeneous edge clusters and hybrid consumer device pools. Vendor-neutral operation and efficient resource reuse were goals, not demonstrated compatibility or performance results.

## Historical record and purpose

The repository exists to preserve the author's dated technical disclosure, including the original scheduler and the proposed OPI/MFPI and modular execution architecture. It is not a claim of exclusive rights to general scheduling concepts or an independently certified priority date.

- [Original scheduler publication commit, December 11, 2025](https://github.com/BlueScreen4/ouroboros-preregister/commit/b380fd0cb0cf8726dfc80fd117e01edad300be1a)
- [Architecture notes](docs/architecture.md)
- [Vision](docs/vision.md)
- [Known technical limitations](docs/technical-limitations.md)
- [Contribution policy](docs/contributing.md)

**Editorial clarification (2026-11-20):** Earlier wording overstated uniqueness, implementation maturity and performance/security properties. This revision explicitly separates historical proposals, observable code behavior and unverified claims. The original scheduler bytes and Git history are preserved.

## License

The public materials in this repository are provided under the existing **CC BY-NC-SA 4.0** license, subject to its terms and applicable exceptions. See [LICENSE](LICENSE) and the [full legal text](https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode). This editorial revision does not relicense the material, establish patent rights, or change the terms of any private implementation.

## Author

BlueScreen4 (Frozenheart Rhapael)  
Creator of the Ouroboros Offload Inference Framework
