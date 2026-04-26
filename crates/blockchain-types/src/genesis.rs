//! Genesis block creation and initial state setup.

use crate::block::Block;
use crate::transaction::{Transaction, TransactionType};
use crate::{AccountId, Hash256, Hash512, Amount, Signature};
use crate::constants::{GENESIS_BLOCK_ID, ONE_NRCS, MAX_BALANCE_NQT, INITIAL_BASE_TARGET};

pub const GENESIS_YEAR: i32 = 2020;
pub const GENESIS_MONTH: u32 = 1;
pub const GENESIS_DAY: u32 = 7;
pub const GENESIS_HOUR: u32 = 5;
pub const GENESIS_MINUTE: u32 = 40;
pub const GENESIS_SECOND: u32 = 0;

pub const CREATOR_ID: i64 = -80957052124787088;
pub const CREATOR_PUBLIC_KEY: [u8; 32] = [
    -73i32 as u8, -14i32 as u8, 35, 45, -38i32 as u8, -25i32 as u8, 117, 68, 105, 14, 20, 
    -105i32 as u8, -15i32 as u8, -75i32 as u8, -114i32 as u8, 39, 64, 57, -61i32 as u8, 
    -80i32 as u8, -7i32 as u8, -99i32 as u8, -101i32 as u8, 96, 120, -14i32 as u8, 82, 
    9, 38, 35, 11, 38
];

pub const GENESIS_BLOCK_SIGNATURE: [u8; 64] = [
    71, -79i32 as u8, -86i32 as u8, -128i32 as u8, 13, 101, 124, -54i32 as u8, -44i32 as u8, 
    -86i32 as u8, -88i32 as u8, -55i32 as u8, 70, -78i32 as u8, -80i32 as u8, -46i32 as u8, 
    -89i32 as u8, 51, 127, -45i32 as u8, -85i32 as u8, -114i32 as u8, -116i32 as u8, 
    -98i32 as u8, -42i32 as u8, -96i32 as u8, 106, 73, -73i32 as u8, 117, 110, 4, -93i32 as u8, 
    -1i32 as u8, 19, -79i32 as u8, 95, 100, 113, -81i32 as u8, -33i32 as u8, -13i32 as u8, 
    3, 19, -31i32 as u8, -60i32 as u8, 124, 76, 42, -80i32 as u8, -30i32 as u8, 9, -57i32 as u8, 
    -118i32 as u8, 6, 115, -92i32 as u8, 44, 37, 75, 116, -52i32 as u8, 2, 1
];

pub fn create_genesis_block() -> Result<Block, Box<dyn std::error::Error>> {
    let genesis_recipients: [i64; 2] = [
        -891382425467438890,
        2794603741293765856
    ];
    
    let genesis_amounts: [i64; 2] = [1, 999999999];
    
    let genesis_signatures: [[u8; 64]; 2] = [
        [
            -88i32 as u8, 116, 91, -49i32 as u8, 8, -93i32 as u8, 97, -90i32 as u8, -70i32 as u8, 
            -19i32 as u8, -106i32 as u8, 17, -33i32 as u8, 16, -70i32 as u8, -3i32 as u8, 3, 112, 
            81, -78i32 as u8, 108, -101i32 as u8, -127i32 as u8, 103, -65i32 as u8, 28, -69i32 as u8, 
            58, 53, 125, -37i32 as u8, 7, 42, -33i32 as u8, -47i32 as u8, -44i32 as u8, 59, 
            -7i32 as u8, 59, -71i32 as u8, 55, -108i32 as u8, 70, -19i32 as u8, 90, -89i32 as u8, 
            -20i32 as u8, -69i32 as u8, 33, -82i32 as u8, -57i32 as u8, 112, 108, 57, -8i32 as u8, 
            103, -122i32 as u8, -46i32 as u8, 81, -59i32 as u8, -32i32 as u8, -107i32 as u8, 
            -12i32 as u8, -86i32 as u8
        ],
        [
            24, 45, -47i32 as u8, -93i32 as u8, -85i32 as u8, -76i32 as u8, 86, -106i32 as u8, 
            26, -64i32 as u8, -41i32 as u8, -123i32 as u8, 42, 8, -110i32 as u8, -65i32 as u8, 
            -91i32 as u8, 126, 48, 111, 114, 127, 0, -89i32 as u8, -36i32 as u8, -26i32 as u8, 
            -64i32 as u8, -50i32 as u8, 123, -43i32 as u8, 61, 9, -16i32 as u8, -89i32 as u8, 
            -66i32 as u8, 123, -43i32 as u8, -62i32 as u8, 95, -15i32 as u8, 114, -31i32 as u8, 
            113, 51, 100, 111, -118i32 as u8, -6i32 as u8, -27i32 as u8, 29, 120, -33i32 as u8, 
            43, -91i32 as u8, -83i32 as u8, -44i32 as u8, 119, 116, 66, 63, 27, 40, -110i32 as u8, 46
        ]
    ];
    
    let mut transactions = Vec::new();
    for i in 0..genesis_recipients.len() {
        let tx = Transaction {
            id: 0,
            version: 0,
            type_id: TransactionType::Payment,
            subtype: 0,
            timestamp: 0,
            deadline: 0,
            sender_public_key: Hash256(CREATOR_PUBLIC_KEY),
            sender_id: CREATOR_ID as u64,
            recipient_id: Some(genesis_recipients[i] as u64),
            amount: (genesis_amounts[i] * ONE_NRCS as i64) as u64,
            fee: 0,
            height: 0,
            block_id: 0,
            block_timestamp: 0,
            transaction_index: i as u16,
            signature: Signature(genesis_signatures[i]),
            full_hash: Hash256([0u8; 32]),
            referenced_transaction_full_hash: None,
            attachment_bytes: vec![],
            phased: false,
            has_message: false,
            has_encrypted_message: false,
            has_public_key_announcement: false,
            has_prunable_attachment: false,
            ec_block_height: None,
            ec_block_id: None,
            has_encrypttoself_message: false,
            has_prunable_encrypted_message: false,
        };
        transactions.push(tx);
    }
    
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for tx in &transactions {
        hasher.update(&tx.serialize_for_signing());
    }
    let payload_hash = hasher.finalize();
    
    let block = Block {
        id: Some(GENESIS_BLOCK_ID),
        version: -1,
        timestamp: 0,
        height: 0,
        previous_block_id: None,
        previous_block_hash: Hash256([0u8; 32]),
        payload_hash: Hash256(payload_hash.into()),
        generator_id: Some(CREATOR_ID as u64),
        generator_public_key: Some(CREATOR_PUBLIC_KEY),
        nonce: 0,
        base_target: INITIAL_BASE_TARGET,
        cumulative_difficulty: vec![],
        total_amount: MAX_BALANCE_NQT,
        total_fee: 0,
        payload_length: (transactions.len() * 128) as u32,
        generation_signature: Hash256([0u8; 32]),
        block_signature: Hash512(GENESIS_BLOCK_SIGNATURE),
        transactions,
    };
    
    Ok(block)
}
