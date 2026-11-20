# Technical Limitations and Evidence Status — Original OPI/MFPI Disclosure

**Review date:** November 20, 2026  
**Scope:** Static inspection of the public historical [scheduler excerpt](../stc_scheduler.rs), README and associated design documents.  
**Original source:** [December 11, 2025 publication commit](https://github.com/BlueScreen4/ouroboros-preregister/commit/b380fd0cb0cf8726dfc80fd117e01edad300be1a).  
**Source blob:** `3edaf2a474a1da1d88a1df13c3f24f1f9765d91b`.

The source is preserved unchanged. This document records observed limitations, not a repair, validated benchmark or exhaustive security audit. No build, runtime test or private-code review was performed. The repository has no complete Cargo project, protocol definitions, container registry or reproducible benchmark suite.

## 1. Actual score calculation

In `calculate_raw_opi`:

```text
raw = 5 * (total_ram_mb / 1024)
    + memory_bandwidth_gbps / 10
    + 2 * (pcie_lanes * pcie_gen)
    + 0.5 * compute_units
if has_rocm: raw *= 1.1
```

The calculation uses floating-point conversion for RAM before division. Field names follow the source, not a verified telemetry-unit contract.

In `calculate_net_factor`, the RTT penalty is the measured smoothed RTT divided by 10 ms, clamped to [1, 10]; missing/nonpositive RTT is treated as 10 ms. In `calculate_load_factor`, availability is one minus the maximum of CPU and GPU load, clamped to [0, 1]. The effective score is raw divided by the network penalty, multiplied by availability; suspect/quarantined nodes receive zero.

The score is a unitless heuristic ranking index, not a physical throughput measure, probability, or statistically estimated performance function. It does not prove optimal assignment.

## 2. Coefficients and assumptions

| Parameter present in the published source | Historical value | Interpretation / limitation |
|---|---|---|
| RAM coefficient | 5 per 1024 reported MB | Capacity is not compute speed; large capacity can dominate the score. |
| Memory bandwidth divisor | 10 | No common device/workload measurement protocol is supplied. |
| PCIe coefficient | 2 times lanes times generation number | Generation number is not per-lane bandwidth; negotiated link rate and transfer overhead are not modeled. |
| Compute-unit coefficient | 0.5 | Compute units are not equivalent across accelerator architectures. |
| ROCm availability multiplier | 1.1 | Experience-informed provisional preference, not a measured 10% performance improvement. |
| Tier cutoffs | 80 and 200 | Ad hoc thresholds, not validated device classes. |
| RTT EMA coefficient | 0.2 | Smoothing choice without published calibration; mean-like latency is not jitter, bandwidth or packet loss. |
| RTT baseline / maximum penalty | 10 ms / 10x | Context-dependent policy; unknown RTT receives a favorable baseline and high penalties saturate. |
| Degraded RTT cutoff | >150 ms | Historical threshold, not a validated network-quality criterion. |
| Suspect / quarantine timeout | >10 s / >30 s | Historical thresholds; the update path has a freshness defect described below. |
| Candidate load exclusion | CPU or GPU load >0.9 | Assumes normalized loads; does not establish resource headroom for a particular task. |
| Maximum selected candidates | 3 | Fixed policy, not derived from actual partition or job requirements. |

The owner's experience informed early backend preferences. No controlled comparative measurements, calibration data, confidence intervals, or independent validation are published. The other backend multipliers are not present in this excerpt and are not assigned historical numeric values here. ROCm availability is not proof of a functioning backend. A score multiplier must not be interpreted as a universal vendor/runtime performance ratio.

## 3. Observed code limitations

| ID | Function / area | Static finding and consequence |
|---|---|---|
| HIST-01 | `update_node_status_with_rtt` / `update_health_state` | `last_seen` is reset before the timeout check; this path cannot meaningfully detect a previously missed heartbeat. No independent stale-node sweep is shown. Stale nodes may remain candidates. |
| HIST-02 | Registration / RTT handling | New nodes start Healthy with zero RTT; zero/nonpositive RTT updates are ignored and missing RTT receives baseline penalty. Unknown measurements are not distinguished from measured good health. |
| HIST-03 | `calculate_raw_opi` / `determine_tier` | Tier uses static hardware score. Recomputing it on heartbeat does not implement network-/thermal-/load-driven tier changes. CPU core count is not used in this score. |
| HIST-04 | `NodeContext` / effective score | Charging and network type are stored, but not used in candidate scoring. Thermal and battery-level measurements are not implemented in this excerpt. |
| HIST-05 | Candidate selection | No task-specific VRAM-fit, model compatibility, backend functionality probe or workload-specific transfer cost check is shown. Registry VRAM requirements and most backend flags are not used for the ranking. |
| HIST-06 | Load / numeric handling | Maximum CPU/GPU load is used regardless of actual task bottleneck. No explicit finite-value/range validation is shown for reported telemetry; NaN scores can reach `partial_cmp(...).unwrap()` and panic during comparison. The integer PCIe product is computed before conversion to float. |
| HIST-07 | `check_server_overload_and_shard` | Overload checks CPU and VRAM pressure but ignores the defined GPU load threshold. No reservation identity, atomic capacity claim or acknowledgement tracking is shown. |
| HIST-08 | `create_shard_command` | Each command has an independent UUID, shard index 0, shard total 1, empty data and fixed destination/tag. This is a placeholder, not demonstrated partitioning, reassembly or container chaining. |
| HIST-09 | Health and trust | Failure count is initialized but no failure-driven trust calculation is shown. The published code does not implement historical-accuracy weighting or trust-weighted result aggregation. |
| HIST-10 | Offload / admin handlers | Offload acceptance, stream initialization and admin success are skeletal responses; assist returns None. The excerpt does not show authenticated authorization, execution or successful completion. |
| HIST-11 | Registry loading | Read or parse failure becomes an empty registry without explicit error propagation. Configuration errors may be masked. |
| HIST-12 | Distributed state | In-memory maps and locks describe one scheduler object. Cross-instance state consistency, ownership fencing, leader selection, failure recovery and decentralized negotiation are not demonstrated. |

These are findings about this excerpt; surrounding private code may have different behavior. No claim is made that every possible defect has been identified.

## 4. Proposed versus demonstrated

The README mentions additional design elements: power/thermal-aware MFPI, heterogeneous backend compatibility, lightweight-model offload, interpreter-generated task graphs, dynamically activated SSD-backed containers, tagged buffers and protected transport. The source does not establish these as complete working features. Physical M.2 hot swapping requires platform-specific safeguards and is not evidenced here. TLS and the proposed additional protocol do not constitute a security assessment. No benchmark supports descriptions of high throughput, optimal allocation or broad hardware compatibility.

## 5. Future validation criteria (recommendations, not implementation)

A successor resource-selection mechanism should separate admission/safety constraints from workload-specific ranking. Establish model/runtime compatibility, memory fit, measured per-workload throughput and latency, sustained thermal/power limits, communication cost and failure rates before treating a candidate as suitable. Unknown or stale telemetry should be explicit; decisions should be reproducible and explainable. Calibrate any weights and thresholds against a predeclared workload and compare with simple baselines using separate validation data. Require isolated tests for freshness, failure, resource ownership, cancellation and actual completion.

These are proposed engineering checks, not a new claim about the original disclosure or a specification for the private implementation.

## 6. Publication and legal scope

This record preserves what the author disclosed and what can be inspected today. It makes no assertion of first invention, patentability, independent priority certification, exclusivity of general technical ideas, or demonstrated superiority. The existing license is unchanged. The prior README's overstatement of uniqueness and other unsupported capability claims, and the inconsistent Apache 2.0 contribution wording, are corrected through a new commit rather than by rewriting history.
