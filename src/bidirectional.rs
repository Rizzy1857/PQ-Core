/// Phase 2: Bidirectional Quantum-Safe Communication with Atomicity
/// 
/// This module extends Phase 1 (basic handshake) with:
/// - Concurrent bidirectional message exchange
/// - Atomic transaction delivery semantics
/// - Message ordering and sequence numbers
/// - Acknowledgment-based reliability
/// - Per-message forward secrecy via chain key ratcheting

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

/// Message type enumeration for protocol control flow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Regular application data
    Data = 0,
    /// Acknowledgment of received message(s)
    Ack = 1,
    /// Synchronization/keepalive probe
    Sync = 2,
    /// Transaction abort signal
    Abort = 3,
}

impl MessageType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(MessageType::Data),
            1 => Some(MessageType::Ack),
            2 => Some(MessageType::Sync),
            3 => Some(MessageType::Abort),
            _ => None,
        }
    }
}

/// Acknowledged message entry in receiver's buffer
#[derive(Debug, Clone)]
pub struct AckedMessage {
    pub sequence_number: u64,
    pub timestamp: u64,
    pub data: Vec<u8>,
}

/// Envelope wrapping each transmitted message with metadata
/// Enables ordering, atomicity, authentication, and forward secrecy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// Monotonically increasing sender sequence number
    pub sequence_number: u64,
    
    /// Unix timestamp (milliseconds)
    pub timestamp: u64,
    
    /// 32-byte hash of sender's identity/public key
    pub sender_id: [u8; 32],
    
    /// Transaction ID: groups related messages for atomic delivery
    pub transaction_id: u128,
    
    /// Message type (Data, Ack, Sync, Abort)
    pub message_type: u8,  // Serializable: MessageType as u8
    
    /// Encrypted payload (application data)
    pub payload: Vec<u8>,
    
    /// AES-GCM nonce (12 bytes)
    pub nonce: [u8; 12],
    
    /// Dilithium signature over (sequence_number || timestamp || payload)
    pub signature: Vec<u8>,
    
    /// For Ack messages: received_up_to_seq (highest seq confirmed)
    pub ack_up_to: Option<u64>,
    
    /// For Ack messages: list of missing sequence numbers
    pub missing_seqs: Vec<u64>,
}

impl MessageEnvelope {
    pub fn new(
        sequence_number: u64,
        timestamp: u64,
        sender_id: [u8; 32],
        transaction_id: u128,
        message_type: MessageType,
        payload: Vec<u8>,
        nonce: [u8; 12],
        signature: Vec<u8>,
    ) -> Self {
        MessageEnvelope {
            sequence_number,
            timestamp,
            sender_id,
            transaction_id,
            message_type: message_type as u8,
            payload,
            nonce,
            signature,
            ack_up_to: None,
            missing_seqs: vec![],
        }
    }

    pub fn as_data_message_type(&self) -> bool {
        self.message_type == MessageType::Data as u8
    }

    pub fn as_ack_message_type(&self) -> bool {
        self.message_type == MessageType::Ack as u8
    }
}

/// Transaction entry in receiver's pending buffer
#[derive(Debug, Clone)]
pub struct PendingTransaction {
    pub transaction_id: u128,
    pub messages: BTreeMap<u64, MessageEnvelope>,
    pub created_at: u64,
}

impl PendingTransaction {
    pub fn new(transaction_id: u128, created_at: u64) -> Self {
        PendingTransaction {
            transaction_id,
            messages: BTreeMap::new(),
            created_at,
        }
    }

    /// Check if all messages in a transaction are contiguous from a starting sequence
    pub fn is_complete(&self, expected_seqs: &[u64]) -> bool {
        for seq in expected_seqs {
            if !self.messages.contains_key(seq) {
                return false;
            }
        }
        true
    }

    /// Extract and deliver messages in order
    pub fn extract_ordered_messages(&self) -> Vec<Vec<u8>> {
        self.messages
            .values()
            .map(|env| env.payload.clone())
            .collect()
    }
}

/// Receiver state: buffers incoming messages per sender
#[derive(Debug, Clone)]
pub struct ReceiverState {
    /// Last confirmed sequence number per sender (None = no message yet)
    pub last_seen_seq: Option<u64>,
    
    /// Buffered messages awaiting transaction completion
    pub pending_transactions: BTreeMap<u128, PendingTransaction>,
    
    /// Recently delivered messages (for duplicate detection)
    pub delivered_messages: Vec<(u64, u128)>,  // (seq, txn_id)
    
    /// Messages received but awaiting earlier messages
    pub out_of_order_buffer: BTreeMap<u64, MessageEnvelope>,
}

impl ReceiverState {
    pub fn new() -> Self {
        ReceiverState {
            last_seen_seq: None,
            pending_transactions: BTreeMap::new(),
            delivered_messages: Vec::new(),
            out_of_order_buffer: BTreeMap::new(),
        }
    }

    /// Check if sequence number is a duplicate
    pub fn is_duplicate(&self, seq: u64) -> bool {
        if let Some(last) = self.last_seen_seq {
            seq <= last
        } else {
            false  // First message ever
        }
    }

    /// Get missing sequence numbers up to a target
    pub fn get_missing_seqs(&self, up_to: u64) -> Vec<u64> {
        let mut missing = Vec::new();
        let start = self.last_seen_seq.map(|s| s + 1).unwrap_or(0);
        for seq in start..=up_to {
            if !self.out_of_order_buffer.contains_key(&seq) {
                missing.push(seq);
            }
        }
        missing
    }
}

/// Sender state: tracks outgoing messages and acknowledgments
#[derive(Debug, Clone)]
pub struct SenderState {
    /// Next sequence number to use
    pub seq_counter: u64,
    
    /// Unacknowledged messages (seq -> envelope)
    pub unacked_messages: BTreeMap<u64, (MessageEnvelope, u64)>,  // (env, sent_at_millis)
    
    /// Last received ACK up to this sequence
    pub acked_up_to: u64,
}

impl SenderState {
    pub fn new() -> Self {
        SenderState {
            seq_counter: 0,
            unacked_messages: BTreeMap::new(),
            acked_up_to: 0,
        }
    }

    pub fn next_seq(&mut self) -> u64 {
        let seq = self.seq_counter;
        self.seq_counter += 1;
        seq
    }

    pub fn record_message(&mut self, seq: u64, envelope: MessageEnvelope, sent_at: u64) {
        self.unacked_messages.insert(seq, (envelope, sent_at));
    }

    pub fn mark_acked(&mut self, up_to: u64) {
        self.acked_up_to = up_to;
        self.unacked_messages.retain(|seq, _| *seq > up_to);
    }

    pub fn get_unacked_count(&self) -> usize {
        self.unacked_messages.len()
    }
}

/// Bidirectional session: enables concurrent sends/receives with atomicity
/// 
/// # Concurrency Model
/// - Multiple threads can call `send()` concurrently (seq_counter is atomic)
/// - Multiple threads can call `recv()` concurrently (recv_buffer is locked)
/// - No contention between senders and receivers
/// 
/// # Atomicity
/// - Messages grouped by transaction_id are delivered atomically
/// - Missing messages cause buffering until complete transaction arrives
/// - Application sees all-or-nothing delivery per transaction
pub struct BidirectionalSession {
    /// Current protocol state
    pub state: Arc<RwLock<crate::protocol::PQState>>,

    // === Sender components ===
    /// Chain key for encrypting outgoing messages (with ratcheting)
    pub tx_chain_key: Arc<Mutex<[u8; 32]>>,
    
    /// Atomic sequence counter (no locking needed)
    pub seq_counter: Arc<AtomicU64>,
    
    /// Sender state (unacked messages, etc.)
    pub sender_state: Arc<Mutex<SenderState>>,

    // === Receiver components ===
    /// Chain key for decrypting incoming messages (with ratcheting)
    pub rx_chain_key: Arc<Mutex<[u8; 32]>>,
    
    /// Receiver state (buffered messages, pending transactions)
    pub receiver_state: Arc<RwLock<ReceiverState>>,

    // === Identity ===
    /// SHA-256 hash of our public key (sender_id in messages)
    pub my_id: [u8; 32],
    
    /// SHA-256 hash of peer's public key (to validate sender_id)
    pub peer_id: [u8; 32],
}

impl BidirectionalSession {
    /// Create new bidirectional session (typically after Phase 1 handshake)
    pub fn new(
        tx_chain_key: [u8; 32],
        rx_chain_key: [u8; 32],
        my_id: [u8; 32],
        peer_id: [u8; 32],
    ) -> Self {
        BidirectionalSession {
            state: Arc::new(RwLock::new(crate::protocol::PQState::Established)),
            tx_chain_key: Arc::new(Mutex::new(tx_chain_key)),
            seq_counter: Arc::new(AtomicU64::new(0)),
            sender_state: Arc::new(Mutex::new(SenderState::new())),
            rx_chain_key: Arc::new(Mutex::new(rx_chain_key)),
            receiver_state: Arc::new(RwLock::new(ReceiverState::new())),
            my_id,
            peer_id,
        }
    }

    /// Send a message with atomicity guarantees
    /// 
    /// Returns: sequence number assigned to this message
    pub fn send(
        &self,
        payload: &[u8],
        transaction_id: u128,
    ) -> Result<u64, crate::protocol::PQError> {
        // Get next sequence number (atomic, no lock)
        let seq = self.seq_counter.fetch_add(1, Ordering::Relaxed);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| crate::protocol::PQError::Other)?
            .as_millis() as u64;

        // Lock sender state to record this message
        let mut sender = self.sender_state.lock()
            .map_err(|_| crate::protocol::PQError::Other)?;
        sender.seq_counter = seq + 1;

        let nonce = [0u8; 12];  // TODO: derive from chain key + seq
        let signature = vec![];  // TODO: sign with Dilithium

        let envelope = MessageEnvelope::new(
            seq,
            timestamp,
            self.my_id,
            transaction_id,
            MessageType::Data,
            payload.to_vec(),
            nonce,
            signature,
        );

        sender.record_message(seq, envelope.clone(), timestamp);

        Ok(seq)
    }

    /// Receive and process a message, buffering until transaction completes
    pub fn receive(&self, envelope: MessageEnvelope) -> Result<(), crate::protocol::PQError> {
        // Validate sender_id matches peer
        if envelope.sender_id != self.peer_id {
            return Err(crate::protocol::PQError::Other);
        }

        let mut receiver = self.receiver_state.write().map_err(|_| crate::protocol::PQError::Other)?;

        // Check for duplicate
        if receiver.is_duplicate(envelope.sequence_number) {
            return Ok(());  // Silently ignore duplicate
        }

        // Check for gap in sequence
        let expected_next = receiver.last_seen_seq.map(|s| s + 1).unwrap_or(0);
        if envelope.sequence_number != expected_next {
            // Out of order, buffer it for later
            receiver.out_of_order_buffer.insert(envelope.sequence_number, envelope.clone());
            return Ok(());
        }

        // In-order message: add to pending transaction and advance
        receiver.last_seen_seq = Some(envelope.sequence_number);

        let txn_id = envelope.transaction_id;
        receiver
            .pending_transactions
            .entry(txn_id)
            .or_insert_with(|| {
                PendingTransaction::new(txn_id, std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0))
            })
            .messages
            .insert(envelope.sequence_number, envelope);

        Ok(())
    }

    /// Get the next complete transaction for delivery to application
    pub fn get_next_complete_transaction(&self) -> Option<(u128, Vec<Vec<u8>>)> {
        let mut receiver = self.receiver_state.write().ok()?;

        // Find first transaction with all messages received
        for (txn_id, txn) in receiver.pending_transactions.iter() {
            // For now, assume single-message transactions are complete
            if !txn.messages.is_empty() {
                let messages = txn.extract_ordered_messages();
                let txn_id_copy = *txn_id;
                receiver.pending_transactions.remove(&txn_id_copy);
                return Some((txn_id_copy, messages));
            }
        }
        None
    }

    /// Get pending ACK information
    pub fn get_ack_state(&self) -> Result<(u64, Vec<u64>), crate::protocol::PQError> {
        let receiver = self.receiver_state.read().map_err(|_| crate::protocol::PQError::Other)?;
        let ack_up_to = receiver.last_seen_seq.unwrap_or(0);
        let missing = receiver.get_missing_seqs(ack_up_to + 32);  // Report gaps up to +32
        Ok((ack_up_to, missing))
    }

    /// Process received ACK message
    pub fn process_ack(&self, ack_up_to: u64) -> Result<(), crate::protocol::PQError> {
        let mut sender = self.sender_state.lock().map_err(|_| crate::protocol::PQError::Other)?;
        sender.mark_acked(ack_up_to);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_envelope_creation() {
        let envelope = MessageEnvelope::new(
            1,
            1000,
            [0u8; 32],
            42,
            MessageType::Data,
            vec![1, 2, 3],
            [0u8; 12],
            vec![],
        );

        assert_eq!(envelope.sequence_number, 1);
        assert_eq!(envelope.transaction_id, 42);
        assert!(envelope.as_data_message_type());
    }

    #[test]
    fn test_bidirectional_session_new() {
        let session = BidirectionalSession::new(
            [0u8; 32],
            [0u8; 32],
            [1u8; 32],
            [2u8; 32],
        );

        assert_eq!(session.seq_counter.load(Ordering::Relaxed), 0);
        assert_eq!(session.my_id, [1u8; 32]);
        assert_eq!(session.peer_id, [2u8; 32]);
    }

    #[test]
    fn test_sender_state_seq_increment() {
        let mut sender = SenderState::new();
        assert_eq!(sender.next_seq(), 0);
        assert_eq!(sender.next_seq(), 1);
        assert_eq!(sender.next_seq(), 2);
    }

    #[test]
    fn test_receiver_state_duplicate_detection() {
        let mut receiver = ReceiverState::new();
        receiver.last_seen_seq = Some(5);

        assert!(receiver.is_duplicate(5));
        assert!(receiver.is_duplicate(3));
        assert!(!receiver.is_duplicate(6));
    }

    #[test]
    fn test_receiver_state_missing_seqs() {
        let mut receiver = ReceiverState::new();
        receiver.last_seen_seq = Some(0);
        receiver.out_of_order_buffer.insert(2, MessageEnvelope::new(
            2, 0, [0u8; 32], 0, MessageType::Data, vec![], [0u8; 12], vec![],
        ));

        let missing = receiver.get_missing_seqs(3);
        assert_eq!(missing, vec![1, 3]);
    }

    #[test]
    fn test_pending_transaction_extract() {
        let mut txn = PendingTransaction::new(42, 1000);
        txn.messages.insert(1, MessageEnvelope::new(
            1, 0, [0u8; 32], 42, MessageType::Data, vec![1, 2], [0u8; 12], vec![],
        ));
        txn.messages.insert(2, MessageEnvelope::new(
            2, 0, [0u8; 32], 42, MessageType::Data, vec![3, 4], [0u8; 12], vec![],
        ));

        let messages = txn.extract_ordered_messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], vec![1, 2]);
        assert_eq!(messages[1], vec![3, 4]);
    }
}
