# Byzantine Fault-Tolerant Time Synchronization in a libp2p-Based Peer-to-Peer System

## Abstract

We present a lightweight Byzantine fault-tolerant (BFT) time synchronization mechanism for a peer-to-peer system built on libp2p. The design combines monotonic logical time, checkpointed slew correction, gossip-based health alerts, and request-response clock sampling. A consensus window derived from peer estimates is used to reject outliers and update the local clock state only when sufficient agreement exists. The implementation emphasizes persistence, monotonicity, and operational simplicity, while remaining compatible with distributed chat and relay workloads.

## 1. Introduction

Time synchronization is a foundational requirement in distributed systems. In peer-to-peer environments, however, no single trusted clock source can be assumed, and peers may be delayed, partitioned, or malicious. This motivates a clock model that can advance monotonically, degrade safely under disagreement, and tolerate a bounded number of faulty participants.

This work describes a BFT time module implemented in Rust for a libp2p-based peer network. The module maintains a logical UTC clock, exchanges time samples with peers, and updates its local state only when the sample set satisfies a Byzantine consensus bound.

## 2. System Model

Let each peer produce an estimate `e_i = (d_i, a_i)`, where `d_i` is the estimated clock difference and `a_i` is the associated accuracy bound. The system tolerates up to `f` Byzantine peers and therefore requires at least `2f + 1` estimates before attempting synchronization.

The local state tracks:

* a base UTC reference,
* a monotonic reference instant,
* a slew rate,
* the last emitted timestamp,
* the last successful synchronization time,
* a persistence checkpoint,
* and the current health status.

If synchronization fails or becomes stale, the clock transitions to an unreliable state and may broadcast a health alert.

## 3. Method

### 3.1 Logical Time Advancement

The clock is advanced from a stored base UTC using elapsed monotonic time:

`current = base_utc + elapsed * slew_rate`

To preserve monotonicity, each returned timestamp is forced to be strictly greater than the last emitted timestamp.

### 3.2 Byzantine Consensus Window

Given peer estimates `e_i = (d_i, a_i)`, the algorithm computes:

`d_over_i = d_i + a_i`

`d_under_i = d_i - a_i`

After sorting these values, the consensus window is defined as:

`m_min = sorted(d_over)[f]`

`m_max = sorted(d_under)[n - 1 - f]`

Synchronization proceeds only if `m_min <= m_max`. The chosen offset is the midpoint of the window:

`offset = (m_min + m_max) / 2`

The slew rate is then adjusted conservatively and clamped to a small correction range.

### 3.3 Failure Handling

If too few estimates are available, or if the consensus window collapses, the module marks the clock as unreliable and records a pending alert. If no successful synchronization occurs for a prolonged interval, the clock also transitions into an unreliable state to surface possible partition or liveness failures.

## 4. Implementation

The implementation is organized around the following components:

* `Clock` - a trait exposing time, status, and metrics.
* `ClockStatus` - a state machine with `Init`, `Synced`, `Slewing`, and `Unreliable`.
* `ClockMetrics` - operational telemetry, including offset and slew rate.
* `SyncState` - the core state machine with persistence and monotonicity guarantees.
* `P2PClock` - a thread-safe wrapper around `SyncState`.
* `TimeSyncBehaviour` - a libp2p `NetworkBehaviour` combining request-response and gossipsub.

The network path uses request-response exchanges to collect samples and gossipsub to publish alerts. The current implementation stores checkpoints on disk so the clock can resume with prior adjustment state after restart.

## 5. Evaluation

The implementation is validated by unit tests that demonstrate the intended properties:

1. initialization with the expected fault-tolerance parameter,
2. monotonic timestamp generation,
3. rejection of synchronization attempts with insufficient peers,
4. successful state update with a valid peer set,
5. and consensus under a multi-peer scenario containing an outlier.

The multi-peer consensus test prints peer samples and the resulting consensus window, making the outlier rejection behavior observable under `--nocapture`.

## 6. Discussion

This design favors clarity and bounded correction over aggressive clock convergence. It is therefore suitable for collaborative distributed applications where relative ordering and operational stability matter more than sub-millisecond precision.

The current network loop applies synchronization after collecting a small peer sample set and uses a conservative slew-rate adjustment. That approach is robust for moderate drift, but it is not a substitute for a fully hardened production time service.

The approach is also informed by earlier work on optimal clock synchronization under different delay assumptions, which motivates bounded-correction designs that remain stable when peer timing uncertainty changes.

It also relates to Byzantine quorum systems, which study how consensus-style quorums can tolerate faults while preserving availability and safety.

## 7. Conclusion

We described a BFT time synchronization module for a libp2p peer-to-peer system. The design combines monotonic logical time, bounded correction, persistence, and consensus-based sample filtering to tolerate Byzantine outliers while maintaining a usable local clock. The resulting mechanism is small, inspectable, and suitable for integration into distributed chat and relay workflows.

## References

1. Castro, M. and Liskov, B. "Practical Byzantine Fault Tolerance."
2. Attiya, H., Herzberg, A., and Rajsbaum, S. "Optimal Clock Synchronization Under Different Delay Assumptions." Proceedings of the twelfth annual ACM symposium on Principles of Distributed Computing, 1993.
3. Barak, B., Halevi, S., Herzberg, A., and Naor, D. "Clock Synchronization with Faults and Recoveries." Proceedings of the nineteenth annual ACM symposium on Principles of Distributed Computing, 2000.
4. Malkhi, D., and Reiter, M. K. "Byzantine Quorum Systems." Proceedings of the twenty-ninth annual ACM symposium on Theory of Computing, 1997.
5. libp2p project documentation.
6. Distributed systems literature on logical clocks and time synchronization.
