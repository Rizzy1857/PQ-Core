# PQ-Core Protocol Flow: Phase 1 & Phase 2

## Phase 1: Unidirectional Handshake (Complete ✅)

### Overview
Phase 1 establishes a quantum-safe symmetric channel using Kyber512 key encapsulation and Dilithium2 signatures.

### Flow

```
Alice                              Bob
  |                                 |
  |--- HandshakeMessage(KEM_pk_A) --->
  |     (with Dilithium sig)         |
  |                                  |
  |     <--- HandshakeMessage(CT_B) -|
  |          (with Dilithium sig)    |
  |                                  |
  |<-- Established with SS_AB ---->|
```

1. **Alice initiates**: Generates ephemeral KEM keypair, signs PK with Dilithium, sends to Bob
2. **Bob processes**: Verifies signature, encapsulates to Alice's KEM_pk, derives shared secret SS
3. **Alice completes**: Decapsulates Bob's ciphertext, derives matching SS
4. **Both derive**: Use HKDF(SS) → tx_chain_key, rx_chain_key
5. **Encrypt/Decrypt**: Use AES-256-GCM with nonce = message counter

---

## Phase 2: Bidirectional Concurrent Communication with Atomicity (This Document)

### Key Innovations

1. **Concurrent Sends**: Both Alice and Bob can send messages simultaneously without blocking
2. **Atomic Delivery**: Messages are guaranteed to be delivered atomically (all-or-nothing)
3. **Strict Ordering**: Message sequence numbers prevent reordering attacks
4. **No Lost Messages**: Transaction-based delivery with acknowledgments
5. **Forward Secrecy**: Each message encrypted with derived sub-key from chain key
6. **Replay Protection**: Sequence numbers + timestamps prevent replay attacks

---

## Protocol Design: Bidirectional Message Exchange

### Message Structure

```rust
MessageEnvelope {
    sequence_number: u64,          // Monotonic per sender
    timestamp: u64,                // Unix timestamp (ms)
    sender_id: [u8; 32],          // Hash of sender's public key
    transaction_id: u128,         // UUID for atomic grouping
    message_type: MessageType,    // DATA, ACK, SYNC, ABORT
    payload: Vec<u8>,             // Encrypted message body
    nonce: [u8; 12],              // AES-GCM nonce
    tag: [u8; 16],                // AES-GCM authentication tag
    signature: DilithiumSignature, // Sign (sequence + timestamp + payload)
}

enum MessageType {
    Data,      // Regular data message
    Ack,       // Acknowledgment of received message
    Sync,      // Synchronization probe
    Abort,     // Transaction abort signal
}
```

### Transmission Guarantees

#### 1. **Atomicity**
- Each message has a `transaction_id` that groups related operations
- Receiver buffers messages in transaction-order
- Only commits (delivers to application) when entire transaction is received
- Abort mechanism allows sender to rollback uncommitted transactions

#### 2. **Ordering**
- Each sender maintains independent `sequence_number` counter
- Receiver validates: `new_seq > last_seen_seq` per sender
- Detects gaps → requests retransmission via ACK with missing seq ranges
- Application sees messages in sender's order of transmission

#### 3. **Concurrent Sends**
- Each party maintains separate TX/RX chain keys
- Simultaneous outbound messages don't interfere (different chain keys)
- Lock-free architecture using thread-local chain key advancement
- Receiver merges incoming streams by timestamp ordering

#### 4. **Liveness & Acknowledgments**
- Every message triggers ACK from receiver (after validation)
- ACK includes: `received_up_to_seq` + `missing_seqs`
- Sender retransmits unacked messages after timeout (exponential backoff)
- Periodic SYNC probes keep connection alive

---

## State Machine

### Per-Connection State

```
┌─────────────────────────────────────────┐
│        Established (from Phase 1)       │
│  tx_chain_key, rx_chain_key derived     │
│  Both parties ready to exchange data    │
└──────────┬────────────────────────────┬─┘
           │                            │
           ▼                            ▼
    Send MessageEnvelope        Receive MessageEnvelope
           │                            │
           ├─ Increment seq_number      ├─ Validate signature
           ├─ Advance chain key         ├─ Check sequence gap
           ├─ Encrypt payload           ├─ Buffer in transaction
           ├─ Sign + Send               └─ Send ACK
           │
           └─ Wait for ACK (timeout = 5s)
                    │
         ┌──────────┴──────────┐
         ▼                     ▼
      Ack'd                Timeout
        │                     │
        └─ Commit            └─ Retransmit (exp backoff)
           message
```

### Receiver State (per sender)

```
┌──────────────────────────────────────────────┐
│  Receive Buffer:                             │
│  {seq_n → msg_n, seq_n+1 → msg_n+1, ...}   │
│  Pending Transactions:                       │
│  {txn_id → [msg1, msg2, ...]}               │
└──────────────────────────────────────────────┘
           │
           ├─ On message: Add to pending txn
           ├─ On txn complete (all msgs received):
           │    → Deliver to app
           │    → Remove from pending
           └─ On txn abort:
                → Discard all msgs in txn
                → Notify app (optional rollback)
```

---

## Example: Concurrent Bidirectional Exchange

### Scenario: Alice & Bob send simultaneously

```
Time  Alice                           Bob
 t0   ├─ Gen msg_A1 (seq=1)          ├─ Gen msg_B1 (seq=1)
      ├─ Encrypt + sign              ├─ Encrypt + sign
      └─ Send ──────────────────────→└─ Send ──────────────────────→
 t1        (in flight)                       (in flight)
 t2   ←──────────────────── msg_B1 ──┘       ←──────────────────── msg_A1 ──┘
 t3   ├─ Validate msg_B1             ├─ Validate msg_A1
      ├─ Deliver to app              ├─ Deliver to app
      └─ Send ACK_B1 (seq=1, ok) ────→ Send ACK_A1 (seq=1, ok) ────→
 t4        (ACK in flight)                    (ACK in flight)
 t5   ←─────────── ACK_A1 received ──┘       ←─────────── ACK_B1 received ──┘
 t6   ├─ Commit msg_A1               ├─ Commit msg_B1
      ├─ Gen msg_A2 (seq=2)          ├─ Gen msg_B2 (seq=2)
      └─ Send ───────────────────────→└─ Send ───────────────────────→
      ...
```

**Key Properties**:
- Messages sent at **same time** (t0)
- Messages **cross in flight** (t1-t2)
- Both received **concurrently** (t3)
- Both **acknowledged** (t4)
- Both **committed** (t6)
- No blocking, no race conditions

---

## Implementation Details

### Chain Key Advancement (Ratcheting)

When sending message with seq=N:

```
current_tx_chain_key: [u8; 32]
    │
    ├─ KDF(current_tx_chain_key, "send_" || seq) → encryption_key
    ├─ KDF(current_tx_chain_key, "send_" || seq) → new_chain_key
    │
    └─ current_tx_chain_key = new_chain_key  (advance for next message)
```

When receiving message with seq=N:

```
current_rx_chain_key: [u8; 32]
    │
    ├─ KDF(current_rx_chain_key, "recv_" || seq) → decryption_key
    ├─ Advance counter: last_seen_seq = N
    │
    └─ If gap (last_seen + 1 ≠ N):
        └─ Request retransmit of missing seqs
```

**Benefits**:
- Forward secrecy per message
- Cryptographic proof of order (can't decrypt out-of-order)
- Ratcheting is O(1) per message (no tree structure overhead)

### Atomicity Mechanism

**Sender side**:
```rust
transaction_id = UUID::new_v4();  // Unique per atomic operation
messages = [msg1, msg2, msg3];
for msg in messages {
    envelope = build_envelope(msg);
    envelope.transaction_id = transaction_id;
    envelope.sequence_number = ++seq_counter;
    send_encrypted(envelope);
}
// Wait for ACKs covering all seq_numbers
wait_for_ack_up_to(seq_counter);
// If timeout: send ABORT message with transaction_id
```

**Receiver side**:
```rust
pending_txns: Map<u128, Vec<MessageEnvelope>> = {};
on_receive(envelope) {
    validate_signature(envelope);
    if envelope.sequence_number <= last_seen_seq {
        return;  // Duplicate
    }
    pending_txns[envelope.transaction_id].push(envelope);
    
    if is_transaction_complete(envelope.transaction_id) {
        deliver_to_app(pending_txns[envelope.transaction_id]);
        pending_txns.remove(envelope.transaction_id);
    }
    
    send_ack(envelope.sequence_number, missing_seqs);
}
```

### Lock-Free Concurrency

```rust
pub struct BidirectionalSession {
    state: Arc<RwLock<SessionState>>,
    
    // Sender side (can be accessed concurrently)
    tx_chain_key: Arc<Mutex<[u8; 32]>>,
    seq_counter: Arc<AtomicU64>,
    
    // Receiver side
    rx_chain_key: Arc<Mutex<[u8; 32]>>,
    recv_buffer: Arc<Mutex<ReceiverBuffer>>,
    pending_txns: Arc<DashMap<u128, Vec<MessageEnvelope>>>,
    
    // Communication channels
    tx_socket: mpsc::Sender<Vec<u8>>,
    rx_socket: mpsc::Receiver<Vec<u8>>,
}
```

**Concurrency Strategy**:
1. **TX operations**: Lock only chain_key for advancement, use atomic seq_counter
2. **RX operations**: Lock recv_buffer and DashMap (concurrent hash map)
3. **No global locks**: Different senders/receivers never contend

---

## Security Properties

| Property | Mechanism | Notes |
|----------|-----------|-------|
| **Confidentiality** | AES-256-GCM per message | Symmetric, derived from PQ key |
| **Authenticity** | Dilithium signature on {seq, ts, payload} | Prevents tampering |
| **Integrity** | GCM authentication tag | Detects bit flips |
| **Forward Secrecy** | Per-message chain key ratcheting | Compromised key ≠ past messages |
| **Replay Protection** | Sequence numbers + timestamps | Detects duplicate/reordered |
| **Atomicity** | Transaction IDs + ACK mechanism | All-or-nothing delivery |
| **Quantum Resistance** | Kyber512 + Dilithium2 (NIST std) | Safe against quantum computers |

---

## Performance Targets

- **Handshake**: ~50ms (Phase 1, one-time)
- **Per-message overhead**: 16 bytes seq/timestamp/sig
- **Throughput**: >1000 msg/sec per direction
- **Latency**: <10ms round-trip ACK
- **Concurrent sends**: Linear scaling (no cross-contention)

---

## Next Steps (Phase 3+)

- **Perfect Forward Secrecy**: Derive sub-chains that expire
- **DoS Protection**: Rate limiting + replay cache optimization
- **Compression**: Compress payloads before encryption
- **Fragmentation**: Split large messages into fragments
- **Persistence**: Transaction log for recovery from crashes
- **Clustering**: Multi-node consensus on message ordering

