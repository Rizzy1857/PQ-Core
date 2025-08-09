# 🛣️ PQ-Core Technical Roadmap Progress (as of June 2025)

## Progress Overview

```mermaid
gantt
    title PQ-Core Roadmap Progress (2025)
    dateFormat  YYYY-MM
    section Core Foundations
    KEM, SIG, Protocol Skeleton      :done, 2025-04, 2025-07
    Replay Protection, Symmetric Enc :done, 2025-06, 2025-08
    Zeroization, Test Suite, CI      :done, 2025-06, 2025-08
    E2E Encrypted Message Milestone  :active, 2025-08, 2025-09
    section Protocol Hardening
    AEAD, Padding, Ratchet           : 2025-09, 2025-12
    Side-Channel, Fuzzing            : 2025-10, 2026-01
    section Network Integration
    Tor/Transport, Peer Discovery    : 2026-01, 2026-04
    Metadata Hardening, Relays       : 2026-03, 2026-06
    section Advanced Features
    ZK, Multi-Device, FFI, Plugins   : 2026-06, 2026-12
    section Audits & Production
    Formal Verification, Audits      : 2027-01, 2027-06
    Docs, Compliance, Release        : 2027-04, 2027-08
```

---

## Completion Estimate

- **Phase 1 (Core Foundations): ~85%**
- **Overall Roadmap: ~15%**

- Core structure, stubs, and basic tests are present.
- Not yet production-ready; advanced protocol features and real cryptography are next.

---

## Next Steps
- Complete end-to-end encrypted message exchange with real cryptography.
- Begin protocol hardening and advanced features.

---

*Progress auto-updated as of June 2025.*
