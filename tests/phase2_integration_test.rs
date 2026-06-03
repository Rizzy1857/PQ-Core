/// Integration tests for Phase 2: Bidirectional quantum-safe communication
/// 
/// Tests verify:
/// - Message ordering guarantees
/// - Atomicity of transaction delivery
/// - Concurrent send/receive correctness
/// - Sequence number validation
/// - Acknowledgment mechanism
/// - Replay protection

use pq_core::{PQSession, BidirectionalSession, MessageEnvelope, MessageType};
use sha2::{Sha256, Digest};

fn hash_identity(name: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(name);
    let result = hasher.finalize();
    let mut id = [0u8; 32];
    id.copy_from_slice(&result[..]);
    id
}

#[test]
fn test_phase2_bidirectional_handshake_to_messaging() {
    // Establish Phase 1 handshake
    let mut alice = PQSession::new();
    let mut bob = PQSession::new();

    let alice_handshake = alice.initiate_handshake().expect("Alice handshake failed");
    let bob_response = bob.process_handshake(alice_handshake).expect("Bob handshake failed");
    alice.complete_handshake(bob_response).expect("Alice complete failed");

    // Create Phase 2 bidirectional sessions
    let alice_id = hash_identity("alice");
    let bob_id = hash_identity("bob");

    let alice_bidir = BidirectionalSession::new(
        [0xAA; 32], [0xBB; 32], alice_id, bob_id,
    );
    let bob_bidir = BidirectionalSession::new(
        [0xBB; 32], [0xAA; 32], bob_id, alice_id,
    );

    // Both can send
    let seq_a = alice_bidir.send(b"Hello", 1).expect("Alice send failed");
    let seq_b = bob_bidir.send(b"World", 2).expect("Bob send failed");

    assert_eq!(seq_a, 0);
    assert_eq!(seq_b, 0);

    // Can send multiple with correct sequences
    let seq_a2 = alice_bidir.send(b"Message 2", 3).expect("Alice send 2 failed");
    let seq_b2 = bob_bidir.send(b"Message 2", 4).expect("Bob send 2 failed");

    assert_eq!(seq_a2, 1);
    assert_eq!(seq_b2, 1);
}

#[test]
fn test_concurrent_bidirectional_sends() {
    let alice_id = hash_identity("alice");
    let bob_id = hash_identity("bob");

    let alice = BidirectionalSession::new(
        [0xAA; 32], [0xBB; 32], alice_id, bob_id,
    );
    let bob = BidirectionalSession::new(
        [0xBB; 32], [0xAA; 32], bob_id, alice_id,
    );

    // Concurrent sends (simulated)
    let alice_seq_1 = alice.send(b"Alice msg 1", 100).expect("Alice 1");
    let bob_seq_1 = bob.send(b"Bob msg 1", 101).expect("Bob 1");
    
    let alice_seq_2 = alice.send(b"Alice msg 2", 102).expect("Alice 2");
    let bob_seq_2 = bob.send(b"Bob msg 2", 103).expect("Bob 2");

    // Verify sequences are monotonic per sender
    assert_eq!(alice_seq_1, 0);
    assert_eq!(alice_seq_2, 1);
    assert!(alice_seq_2 > alice_seq_1);

    assert_eq!(bob_seq_1, 0);
    assert_eq!(bob_seq_2, 1);
    assert!(bob_seq_2 > bob_seq_1);
}

#[test]
fn test_message_ordering_monotonic_sequences() {
    let alice_id = hash_identity("alice");
    let bob_id = hash_identity("bob");

    let alice = BidirectionalSession::new(
        [0xAA; 32], [0xBB; 32], alice_id, bob_id,
    );

    // Send 10 messages
    let mut sequences = Vec::new();
    for i in 0..10 {
        let seq = alice.send(
            format!("Message {}", i).as_bytes(),
            1000 + i as u128,
        ).expect("Send failed");
        sequences.push(seq);
    }

    // Verify all sequences are monotonically increasing and unique
    for i in 0..sequences.len() {
        assert_eq!(sequences[i], i as u64, "Sequence at {} should be {}", i, i);
    }
}

#[test]
fn test_receiver_duplicate_detection() {
    let alice_id = hash_identity("alice");
    let bob_id = hash_identity("bob");

    let bob = BidirectionalSession::new(
        [0xBB; 32], [0xAA; 32], bob_id, alice_id,
    );

    // Create message at seq 5
    let envelope1 = MessageEnvelope::new(
        5, 1000, alice_id, 1, MessageType::Data,
        b"Test".to_vec(), [0; 12], vec![],
    );

    // Try to receive seq 5 before seq 0-4 (should buffer as out-of-order)
    bob.receive(envelope1.clone()).expect("First receive failed");

    let receiver = bob.receiver_state.read().expect("Lock failed");
    // Should be None since seq 5 > expected 0
    assert!(receiver.last_seen_seq.is_none());
    // Should be buffered as out-of-order
    assert!(receiver.out_of_order_buffer.contains_key(&5));

    drop(receiver);

    // Try to receive the same message again (duplicate check happens for in-order msgs)
    // This should silently ignore as buffer already has it
    bob.receive(envelope1).expect("Duplicate receive should not error");
}

#[test]
fn test_out_of_order_buffering() {
    let alice_id = hash_identity("alice");
    let bob_id = hash_identity("bob");

    let bob = BidirectionalSession::new(
        [0xBB; 32], [0xAA; 32], bob_id, alice_id,
    );

    // Receive message 5 first (before 0-4)
    let envelope_5 = MessageEnvelope::new(
        5, 1000, alice_id, 1, MessageType::Data,
        b"Msg 5".to_vec(), [0; 12], vec![],
    );

    bob.receive(envelope_5).expect("Receive 5 failed");

    let receiver = bob.receiver_state.read().expect("Lock failed");
    
    // Message 5 should be in out_of_order_buffer, not delivered yet
    assert!(receiver.last_seen_seq.is_none());  // Still waiting for seq 0
    assert!(receiver.out_of_order_buffer.contains_key(&5));
}

#[test]
fn test_atomic_transaction_single_message() {
    let alice_id = hash_identity("alice");
    let bob_id = hash_identity("bob");

    let bob = BidirectionalSession::new(
        [0xBB; 32], [0xAA; 32], bob_id, alice_id,
    );

    let txn_id = 42u128;
    let envelope = MessageEnvelope::new(
        0, 1000, alice_id, txn_id, MessageType::Data,
        b"Single msg transaction".to_vec(), [0; 12], vec![],
    );

    bob.receive(envelope).expect("Receive failed");

    // Transaction should be available for delivery
    if let Some((delivered_txn_id, messages)) = bob.get_next_complete_transaction() {
        assert_eq!(delivered_txn_id, txn_id);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], b"Single msg transaction");
    } else {
        panic!("Transaction not delivered");
    }
}

#[test]
fn test_atomic_transaction_multiple_messages() {
    let alice_id = hash_identity("alice");
    let bob_id = hash_identity("bob");

    let bob = BidirectionalSession::new(
        [0xBB; 32], [0xAA; 32], bob_id, alice_id,
    );

    let txn_id = 100u128;

    // Send 3 messages in same transaction
    let envelope1 = MessageEnvelope::new(
        0, 1000, alice_id, txn_id, MessageType::Data,
        b"Part 1".to_vec(), [0; 12], vec![],
    );

    let envelope2 = MessageEnvelope::new(
        1, 1001, alice_id, txn_id, MessageType::Data,
        b"Part 2".to_vec(), [0; 12], vec![],
    );

    let envelope3 = MessageEnvelope::new(
        2, 1002, alice_id, txn_id, MessageType::Data,
        b"Part 3".to_vec(), [0; 12], vec![],
    );

    bob.receive(envelope1).expect("Receive 1 failed");
    bob.receive(envelope2).expect("Receive 2 failed");
    bob.receive(envelope3).expect("Receive 3 failed");

    // All 3 messages should be delivered atomically
    if let Some((delivered_txn_id, messages)) = bob.get_next_complete_transaction() {
        assert_eq!(delivered_txn_id, txn_id);
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0], b"Part 1");
        assert_eq!(messages[1], b"Part 2");
        assert_eq!(messages[2], b"Part 3");
    } else {
        panic!("Transaction not delivered");
    }
}

#[test]
fn test_ack_mechanism() {
    let alice_id = hash_identity("alice");
    let bob_id = hash_identity("bob");

    let bob = BidirectionalSession::new(
        [0xBB; 32], [0xAA; 32], bob_id, alice_id,
    );

    // Receive messages 0, 1
    for seq in 0..2 {
        let envelope = MessageEnvelope::new(
            seq, 1000 + seq, alice_id, seq as u128, MessageType::Data,
            format!("Msg {}", seq).into_bytes(), [0; 12], vec![],
        );
        bob.receive(envelope).expect("Receive failed");
    }

    // Get ACK state
    let (ack_up_to, missing) = bob.get_ack_state().expect("Get ACK state failed");
    
    assert_eq!(ack_up_to, 1);  // Acknowledged up to seq 1
    assert!(missing.contains(&3));  // Missing seq 3 onwards
}

#[test]
fn test_process_ack_clears_unacked() {
    let alice_id = hash_identity("alice");
    let bob_id = hash_identity("bob");

    let alice = BidirectionalSession::new(
        [0xAA; 32], [0xBB; 32], alice_id, bob_id,
    );

    // Send 3 messages
    alice.send(b"Msg 1", 1).expect("Send 1 failed");
    alice.send(b"Msg 2", 2).expect("Send 2 failed");
    alice.send(b"Msg 3", 3).expect("Send 3 failed");

    // Check unacked count
    {
        let sender = alice.sender_state.lock().expect("Lock failed");
        // Messages are recorded with seq 0, 1, 2
        assert_eq!(sender.seq_counter, 3);  // seq_counter is incremented to 3
    }

    // Process ACK for seq 2 (all 3 messages)
    alice.process_ack(2).expect("Process ACK failed");

    // Unacked count should be 0 (all messages acked)
    {
        let sender = alice.sender_state.lock().expect("Lock failed");
        assert_eq!(sender.get_unacked_count(), 0);  // All messages cleared
    }
}

#[test]
fn test_replay_protection_timestamp_sequence() {
    let alice_id = hash_identity("alice");
    let bob_id = hash_identity("bob");

    let bob = BidirectionalSession::new(
        [0xBB; 32], [0xAA; 32], bob_id, alice_id,
    );

    // Receive message 0 first (in order)
    let envelope1 = MessageEnvelope::new(
        0, 1000, alice_id, 1, MessageType::Data,
        b"Msg".to_vec(), [0; 12], vec![],
    );

    bob.receive(envelope1.clone()).expect("First receive failed");

    let receiver_check = bob.receiver_state.read().expect("Lock failed");
    assert_eq!(receiver_check.last_seen_seq, Some(0));
    drop(receiver_check);

    // Second identical receive should be silently ignored (duplicate)
    bob.receive(envelope1).expect("Duplicate receive should not error");

    // Receive message 1 (next in sequence)
    let envelope3 = MessageEnvelope::new(
        1, 1001, alice_id, 2, MessageType::Data,
        b"Next".to_vec(), [0; 12], vec![],
    );
    bob.receive(envelope3).expect("Next receive failed");

    let receiver = bob.receiver_state.read().expect("Lock failed");
    assert_eq!(receiver.last_seen_seq, Some(1));  // Advanced to 1
}

#[test]
fn test_sender_state_unacked_management() {
    let mut sender = pq_core::bidirectional::SenderState::new();

    // Send 5 messages
    for i in 0..5 {
        sender.seq_counter = i;
        sender.record_message(i, 
            MessageEnvelope::new(i, 1000 + i, [0; 32], i as u128, MessageType::Data,
                vec![], [0; 12], vec![]),
            1000 + i);
    }

    assert_eq!(sender.get_unacked_count(), 5);

    // Acknowledge up to seq 2
    sender.mark_acked(2);
    assert_eq!(sender.get_unacked_count(), 2);  // Only 3 and 4 remain

    // Verify correct messages remain
    assert!(!sender.unacked_messages.contains_key(&0));
    assert!(!sender.unacked_messages.contains_key(&1));
    assert!(!sender.unacked_messages.contains_key(&2));
    assert!(sender.unacked_messages.contains_key(&3));
    assert!(sender.unacked_messages.contains_key(&4));
}

#[test]
fn test_receiver_state_gap_detection() {
    let mut receiver = pq_core::bidirectional::ReceiverState::new();
    receiver.last_seen_seq = Some(0);

    // Buffer seq 5 (gap between 1-4)
    receiver.out_of_order_buffer.insert(5, MessageEnvelope::new(
        5, 0, [0; 32], 0, MessageType::Data, vec![], [0; 12], vec![],
    ));

    // Buffer seq 10 (gap between 6-9)
    receiver.out_of_order_buffer.insert(10, MessageEnvelope::new(
        10, 0, [0; 32], 0, MessageType::Data, vec![], [0; 12], vec![],
    ));

    let missing = receiver.get_missing_seqs(10);
    
    // Should report all missing sequences
    assert!(missing.contains(&1));
    assert!(missing.contains(&2));
    assert!(missing.contains(&3));
    assert!(missing.contains(&4));
    assert!(missing.contains(&6));
    assert!(missing.contains(&7));
    assert!(missing.contains(&8));
    assert!(missing.contains(&9));
}
