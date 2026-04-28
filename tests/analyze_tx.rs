//! 分析 transaction.data 中的差异记录
//!
//! 从 tests/transaction.data 文件中读取每笔交易，
//! 解析 attachment_bytes 的二进制结构，与当前 Rust 实现对比

use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let file = File::open("tests/transaction.data").expect("Failed to open transaction.data");
    let reader = BufReader::new(file);
    
    println!("=== Transaction Data Analysis ===\n");
    println!("{:<6} {:<22} {:<4} {:<4} {:<10} {:<70} {:<8} ATTACHMENT_BYTES_LEN | PARSED_STRUCTURE",
        "DB_ID", "ID", "TYPE", "SUBTYPE", "VERSION", "FULL_HASH", "ATT_BYTES_HEX");
    println!("{}", "-".repeat(200));
    
    for (line_num, line) in reader.lines().enumerate() {
        if line_num == 0 { continue; } // skip header
        
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 25 { continue; }
        
        let db_id = fields[0].trim();
        let id = fields[1].trim();
        let tx_type = fields[11].trim();
        let subtype = fields[12].trim();
        let version = fields[19].trim();
        let full_hash = fields[6].trim();
        let att_bytes_hex = fields[17].trim();
        
        let att_bytes = match hex::decode(att_bytes_hex) {
            Ok(b) => b,
            Err(_) => continue,
        };
        
        // Parse attachment_bytes structure
        let structure = parse_attachment_structure(tx_type.parse().unwrap_or(0), 
                                                   subtype.parse().unwrap_or(0),
                                                   version.parse().unwrap_or(0),
                                                   &att_bytes);
        
        println!("{:<6} {:<22} {:<4} {:<4} {:<10} {:<70} {:<8} {:<4} | {}",
            db_id, id, tx_type, subtype, version, full_hash,
            &att_bytes_hex[..att_bytes_hex.len().min(40)],
            att_bytes.len(),
            structure);
    }
}

fn parse_attachment_structure(tx_type: u8, subtype: u8, version: u8, bytes: &[u8]) -> String {
    if bytes.is_empty() { return "(empty)".to_string(); }
    
    let mut parts = Vec::new();
    let mut offset = 0;
    
    // First byte is usually attachment version (if version > 0)
    if bytes.len() >= 1 && version > 0 {
        parts.push(format!("att_ver={}", bytes[0]));
        offset = 1;
    }
    
    match (tx_type, subtype) {
        (1, 9) => {
            // PhasingVoteCasting
            if offset < bytes.len() {
                let count = bytes[offset];
                parts.push(format!("count={}", count));
                offset += 1;
                for i in 0..count.min(3) as usize {
                    if offset + 32 <= bytes.len() {
                        parts.push(format!("hash[{}]={}", i, hex::encode(&bytes[offset..offset+32])));
                        offset += 32;
                    }
                }
                if offset < bytes.len() {
                    parts.push(format!("trailing={}", hex::encode(&bytes[offset..])));
                }
            }
        }
        (1, 0) => {
            // ArbitraryMessage - may have Message appendix
            if offset + 4 <= bytes.len() {
                let len_field = i32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
                let is_text = len_field < 0;
                let msg_len = (len_field as u32) & 0x7FFFFFFF;
                parts.push(format!("msg_len={} isText={}", msg_len, is_text));
                offset += 4;
                if offset + msg_len as usize <= bytes.len() {
                    let msg_data = &bytes[offset..offset + msg_len as usize];
                    let preview = String::from_utf8_lossy(&msg_data[..msg_data.len().min(16)]);
                    parts.push(format!("msg=\"{}{}\"", preview, if msg_data.len() > 16 { "..." } else { "" }));
                    offset += msg_len as usize;
                }
            }
            if offset < bytes.len() {
                parts.push(format!("extra={}", hex::encode(&bytes[offset..])));
            }
        }
        (12, 0) => {
            // AliasAssignment
            if offset < bytes.len() {
                let name_len = bytes[offset] as usize;
                parts.push(format!("name_len={}", name_len));
                offset += 1;
                if offset + name_len <= bytes.len() {
                    let name = String::from_utf8_lossy(&bytes[offset..offset+name_len]);
                    parts.push(format!("alias=\"{}\"", name));
                    offset += name_len;
                }
            }
            // priceNQT (8 bytes)
            if offset + 8 <= bytes.len() {
                let price = u64::from_le_bytes([
                    bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
                    bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7],
                ]);
                parts.push(format!("price={}", price));
                offset += 8;
            }
            // May have Message appendix following
            if offset + 4 <= bytes.len() {
                let len_field = i32::from_le_bytes([bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]]);
                let msg_len = (len_field as u32) & 0x7FFFFFFF;
                if msg_len > 0 && msg_len <= 1000 && offset + 4 + msg_len as usize <= bytes.len() {
                    parts.push(format!("appended_msg_len={}", msg_len));
                }
            }
            if offset < bytes.len() {
                parts.push(format!("extra={}", hex::encode(&bytes[offset..])));
            }
        }
        (2, 10) => {
            // PublicKeyAnnouncement
            if offset + 32 <= bytes.len() {
                let pk = hex::encode(&bytes[offset..offset+32]);
                parts.push(format!("pubkey={}", &pk[..16]));
                offset += 32;
            }
            if offset < bytes.len() {
                parts.push(format!("extra={}", hex::encode(&bytes[offset..])));
            }
        }
        (5, 0) => {
            // AssetTransfer or similar
            if offset + 8 <= bytes.len() {
                let asset_id = u64::from_le_bytes([
                    bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
                    bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7],
                ]);
                parts.push(format!("asset_id={}", asset_id));
                offset += 8;
            }
            if offset + 8 <= bytes.len() {
                let quantity = u64::from_le_bytes([
                    bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3],
                    bytes[offset+4], bytes[offset+5], bytes[offset+6], bytes[offset+7],
                ]);
                parts.push(format!("quantity={}", quantity));
                offset += 8;
            }
            if offset < bytes.len() {
                parts.push(format!("extra={}", hex::encode(&bytes[offset..])));
            }
        }
        _ => {
            // Generic: show first few bytes and total length
            if offset < bytes.len() {
                let remaining = &bytes[offset..];
                parts.push(format!("data={}B:{}", remaining.len(), hex::encode(&remaining[..remaining.len().min(20)])));
            }
        }
    }
    
    parts.join(" | ")
}
