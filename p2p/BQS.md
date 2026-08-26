# Byzantine Quorum Systems for Decentralized Version Control

## Abstract

`gnostr` can be modeled as a Byzantine quorum system (BQS) for decentralized version control. In this framing, peers participate in quorums for replication, validation, and recovery while tolerating missing, faulty, or adversarial participants. This note records the BQS basis for the repository design and points to the related BFT time synchronization work in `p2p/src/BFT_TIME.md`.

## 1. Motivation

Decentralized version control needs safety under partial failure and adversarial behavior. A BQS model provides a natural way to express that requirement: progress and correctness depend on quorum intersection rather than on a single trusted coordinator.

In `gnostr`, that means repository state can be understood as distributed across peers that must agree through quorum overlap. The system is therefore closer to a Byzantine quorum design than to a classic centralized VCS.

## 2. Relation to Time Synchronization

Quorum-based coordination is also relevant to distributed time. The companion paper `p2p/src/BFT_TIME.md` describes a Byzantine fault-tolerant time synchronization module that uses consensus windows, bounded correction, and fault tolerance to preserve monotonic logical time.

Together, the two papers describe complementary parts of the same design space:

* `BQS.md` models repository consistency and availability.
* `BFT_TIME.md` models peer time agreement under faults.

## 3. Conclusion

Viewing `gnostr` as a Byzantine quorum system provides a compact way to reason about decentralized version control under failure. The same quorum assumptions motivate the time module and the broader p2p architecture.

## References

1. Malkhi, D., and Reiter, M. K. "Byzantine Quorum Systems." Proceedings of the twenty-ninth annual ACM symposium on Theory of Computing, 1997.
2. Malkhi, D., and Reiter, M. K. "Byzantine Quorum Systems." https://people.cs.umass.edu/~arun/cs691ee/reading/BQS97.pdf
3. `p2p/src/BFT_TIME.md`
