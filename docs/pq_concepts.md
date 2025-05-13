# 🧠 PQ-Core: Core Cryptographic Concepts

Welcome to the underbelly of **PQ-Core**. This is where the math lives, where the paranoid flourish, and where we finally answer: _"What the hell makes this thing post-quantum?"_

If you’re looking for shallow “we use AES because it’s secure” nonsense, you’re in the wrong repo. This document is where we **break down the cryptographic primitives**, **justify the design choices**, and keep it clear enough that even your sleep-deprived future self can understand it.

Let’s get into it.

---

## 🌪️ 1. The Quantum Problem (And Why RSA Can Cry)

Classical crypto is great... until it isn’t.

Algorithms like **RSA**, **ECDSA**, and **Diffie-Hellman** have one thing in common: **they die fast when quantum computers show up**. Specifically:

- **Shor’s Algorithm** breaks RSA, ECC, and DSA like dry twigs.
- **Grover’s Algorithm** weakens symmetric ciphers but not fatally—just double your key length and call it a day.

So we ditch the **number field playground** and jump into **lattices**, **modules**, and weird math that quantum computers hate.

---

## 🧱 2. Our Core Stack (In Plain English)

Here’s what PQ-Core is built on. Each part is custom-coded. No trust in opaque crypto boxes. Every primitive here is designed for **post-quantum resilience** and then some.

---

### 🔑 Kyber-like KEM (Key Encapsulation Mechanism)

**Purpose:** Secure key exchange over an insecure channel.

**How it works (simplified):**

1. Bob generates a public/private keypair using structured lattice problems.
2. Alice takes Bob’s public key, encrypts a random secret using it.
3. She sends Bob the ciphertext.
4. Bob decrypts it with his private key.
5. Now they both share the same secret—but quantum adversaries can't reverse-engineer it.

**Why it slaps:**
- Fast as hell.
- Small key sizes (relatively).
- Based on the **Module-LWE** problem. Quantum-resistant with nasty math nobody knows how to break efficiently.

**What we’re doing differently:**
- Tweaking compression/packing for better Tor throughput.
- Built-in key rotation logic—because “permanent” is a security liability.

---

### ✍️ Dilithium-style Digital Signatures

**Purpose:** Verify identity without revealing anything more than you must.

**How it works:**
- It’s like Schnorr or ECDSA, but instead of using elliptic curves, we’re slinging structured lattices and rejection sampling.
- The signer generates a random blinding value, proves knowledge of a secret vector, and the verifier checks everything against a public key.

**Why it rocks:**
- Post-quantum secure.
- Relatively fast signing and verification.
- No “I hope the RNG gods smile on me today” moments like DSA.

**Our approach:**
- Tune parameters for mobile clients (signatures are a bit chunky).
- Optional support for ephemeral signatures with zero persistence.

---

### 📦 Symmetric Encryption (PQ-Agnostic, Quantum-Aware)

**Symmetric crypto** is still alive and kicking in the post-quantum era. Quantum computers _halve_ the effective strength, so we adapt accordingly.

- We use **AES-256** (or a customizable pluggable module).
- All data is encrypted using a shared key derived from the Kyber-like KEM.
- Message payloads are wrapped in authenticated encryption (e.g., AES-GCM or ChaCha20-Poly1305).

**Why it’s solid:**
- Grover’s gives you a square root speed-up at best. AES-256 still offers 128-bit post-quantum security.
- Symmetric crypto is fast, efficient, and hardware-accelerated on most platforms.

---

### 🧅 Onion-Style Message Wrapping

We’re not just encrypting—**we’re layering**.

Each message can be:
- Encrypted at multiple hops (for Tor integration or P2P relay models).
- Encapsulated in padding to normalize size.
- Timestamped and jittered to mess with traffic analysis.

**You don’t just get encrypted messages—you get plausible deniability baked into the protocol.**

---

## ⚠️ 3. Security Assumptions

Let’s talk straight.

We assume:
- Lattice problems like **MLWE** and **MSIS** are still hard for quantum attackers. If they’re not, the entire post-quantum world is screwed together.
- Quantum computers won’t get cheap and mobile before we finish building this thing.
- Your operating system isn't leaking your secrets through spyware.

**We do NOT assume:**
- Perfect entropy.
- Trustworthy silicon.
- That users read security warnings.

PQ-Core is designed to keep working even when your users are doing dumb things like reusing keys or pasting secrets in Notepad.

---

## 🏗️ 4. Modularity by Design

PQ-Core isn’t some monolithic black box. It’s a toolkit, not a straitjacket.

Each cryptographic component:
- Lives in its own module (e.g., `kem/`, `sig/`, `symmetric/`)
- Can be swapped out or upgraded without rewriting the entire stack.
- Has sane defaults, but if you know what you’re doing, you can tinker freely.

In short: **you own the stack**.

---

## 🧪 5. Testing Philosophy

We test like someone’s watching. Because they probably are.

- Every crypto operation is fuzzed.
- All implementations are tested against known vectors and edge cases.
- Constant-time where needed, or we don’t ship it.

We’d rather take two weeks to optimize the right curve than rush a buggy proof-of-concept that leaks secrets over timing channels.

---

## 🛠️ 6. How to Use This Stuff 🚬

If you're building on top of PQ-Core, the typical flow looks like:

1. Generate or load a keypair (KEM + SIG).
2. Share your public key + .onion address.
3. Receive a connection → perform KEM key exchange.
4. Verify incoming signature (if needed).
5. Derive shared symmetric key.
6. Encrypt/decrypt messages like it's 2035 and the NSA is actively crying.

All while running through Tor. All without trusting Google, Facebook, or that weird API that says "free encryption" but calls home.
### ⚠️AS OF NOW, PQ CORE IS IN ITS EARLY STAGES (TOO EARLY), SO WAIT TILL v0.5 FOR DEVOLOPING IN PQ_CORE
---

## 🚨 Bonus: Forward Secrecy & Key Rotation

- Every session can negotiate a fresh symmetric key via ephemeral KEMs.
- You can optionally enable **ratcheting** (like Signal’s Double Ratchet, but PQ).
- Long-term keys are **discouraged** unless you’re doing persistent identities (and even then, rotate like hell).

**Your messages should not outlive your moods.**

---

## Final Thoughts

Crypto isn’t magic. It’s math + paranoia + engineering. But when done right, it becomes **freedom-tech**.

**PQ-Core isn’t perfect. It’s not bulletproof. But it’s designed by people who give a damn—and that’s a rare thing in crypto today.**

Keep it updated. Break it with tests. Understand the primitives. And don’t be afraid to refactor ruthlessly if the math demands it.

Your future self (and your future app users) will thank you for building with eyes wide open.

*(ANY POSITIVE CONTRIBUTION IS WELCOME)*

---
