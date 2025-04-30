# 🔁 PQ-Core Protocol Flow

This doc walks you through **exactly what happens** when two users connect and communicate using PQ-Core. No fluff, no crypto-mysticism—just the flow.

---

## 🧩 The Big Picture

Imagine two users:
- **Alice**: A hopeless romantic using her phone over Tor.
- **Bob**: A faceless guy with a `.onion` address and some spicy key material.

They’ve never met before. They have no shared trust. Yet within seconds, they’re chatting securely, privately, and quantum-resiliently.

Here’s how.

---

## 🏁 1. Initialization: *Hello, Darkness* 😉

Each user starts by spinning up:

- A **Tor Onion Service** (for inbound communication).
- A **KEM keypair** (for encrypting session secrets).
- A **Digital Signature keypair** (optional, for persistent identity or verifying requests).
- A **PQ-Core identity object**, which includes:
  - Public KEM key
  - Public Signature key
  - Tor `.onion` address (v3)
  - Metadata (optional, e.g., app version, avatar hash)

**Think of this like a self-contained, self-sovereign contact card—crypto edition.**

---

## 🔍 2. Discovery (Peer Introduction)

There are a few ways users can find each other:

- Share identity out-of-band (invite code, QR, mutual friend)
- Use a privacy-preserving DHT (Decentralized Hash Table over Tor)
- Sneaky QR on a bar napkin

**No central server. No phone numbers. No logins. You are your `.onion`.**

---

## 🤝 3. Connection Initiation

Alice wants to talk to Bob. Here’s what she does:

1. **Tor Connects**: Alice connects to Bob’s `.onion` address via SOCKS5 proxy.
2. **Public Info Request**: Alice requests Bob’s public KEM and SIG keys.
3. **Verification**: Optionally, she checks the signature of Bob’s identity payload.

At this point, **Alice still hasn’t revealed any of her own info**. She’s operating like a ghost.

---

## 🔐 4. Key Encapsulation (KEM Phase)

Alice wants to start a secure session, so she encapsulates a shared secret:

1. Alice generates a **random session key `K`**.
2. She encrypts `K` using Bob’s public KEM key → `ciphertext_C`.
3. She sends `ciphertext_C` to Bob.

Bob decrypts `ciphertext_C` with his KEM secret key → and now **both share `K`**.

🔒 Boom. Shared post-quantum session key in place. No quantum-backdoor handshake. No sniffable data. No forwardable secrets.

---

## 📝 5. Optional Signature Verification

If identity matters (e.g., persistent contact, trusted session), Alice and Bob can:

- Sign the initial handshake messages.
- Include timestamp + context hash to prevent replay attacks.
- Rotate keys if expired or compromised.

No centralized CA. No blind trust. Just raw, user-controlled crypto identities.

---

## 🔄 6. Symmetric Session Starts

Using the shared session key `K`:

- Messages are encrypted with AES-256 (or your chosen algorithm).
- Every message includes:
  - A nonce (for replay protection)
  - Optional padding (for traffic normalization)
  - Timestamp and optional TTL
  - MAC for tamper detection

Each message is a little encrypted box of joy—unreadable, unlinkable, and quantum-resistant.

---

## 🧅 7. Tor Routing & Anonymity Layer

Here’s what’s wild: the whole exchange so far has been happening **over Tor**.

- Neither party sees the other’s IP.
- Timing is obscured by relays.
- DNS? Never even touched.
- Metadata? Choked out and dumped in a shallow grave.

PQ-Core rides inside Tor like a ninja inside a tank.

---

## 🔄 8. Optional Ratcheting & Key Renewal

Every few messages—or at user-defined intervals—Alice and Bob can:

- Re-run the KEM phase (generate new session keys)
- Use ephemeral keypairs for temporary chats
- Ratchet forward like in Signal (but using PQ-safe operations)

📌 This gives:
- **Forward secrecy**: compromise one message? Good luck with the rest.
- **Post-compromise security**: even if a device is later hacked, the past is sealed tight.

---

## ⛓️ 9. Message Chaining & Replay Protection

Each message contains:

- A message counter
- A previous hash
- A MAC (Message Auth Code)

This means:
- Messages are verifiably ordered.
- If someone replays an old message, it’ll be flagged or rejected.
- Chains break gracefully if needed (e.g., interrupted session, dropped packets).

---

## 💣 10. Session Termination

When either party wants out:

1. Send a “BYE” message (encrypted and signed).
2. Securely zero out keys from memory.
3. Destroy ephemeral Tor HS (optional).
4. Optionally blacklist the peer in local config.

No lingering presence. No half-open sockets. No ghosts in the wire.

---

## 📋 Flow Recap (TL;DR)

1. Both users run Tor HS and generate KEM/SIG keypairs.
2. One connects to the other via `.onion`.
3. KEM encapsulation gives them a shared symmetric key.
4. Messages are encrypted + authenticated using that key.
5. All traffic flows through Tor.
6. Optionally verify identity via signatures.
7. Optional ratcheting for extra paranoia.
8. Everything is self-contained, decentralized, and PQ-hardened.

---

## 🎮 Bonus: Simulation Flow Diagram

Want a flowchart? Ping me and I’ll draw it out for you—each state, handshake, ratchet, and teardown in full glory.

---

## 👁️‍🗨️ Real-World Equivalent

This is like if **Signal**, **Ricochet IM**, and **a crypto-anarchist bunker** had a post-quantum lovechild that refused to use phone numbers, servers, or anything that smelled like surveillance capitalism.

You’re not just building a messaging protocol. You’re building **the blueprint for communication in the age of quantum spying and corporate overreach**.

---

## Final Note

**PQ-Core is about ownership.** Of your keys, your metadata, your words, your identity. No one else gets a say.

This protocol isn't just paranoid—it's principled. And if someone breaks it, they earned it.
