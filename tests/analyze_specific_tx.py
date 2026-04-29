#!/usr/bin/env python3
"""
分析指定 db_id 的 transaction 记录
"""
import struct

target_ids = [23,29,30,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,64,65]

with open('/Volumes/DATA/data/develop/git/rust-nrcs/tests/transaction.data', 'r') as f:
    lines = f.readlines()

header = [h.strip() for h in lines[0].strip().split('|')]
idx = {h: i for i, h in enumerate(header)}

print("=" * 140)
print(f"{'DB_ID':>6} | {'TYPE':>5} | {'SUBTYPE':>7} | {'HAS_MSG':>7} | {'HAS_PPM':>7} | {'HAS_PPA':>7} | {'ATT_LEN':>7} | ATTACHMENT_BYTES (前40字节)")
print("=" * 140)

for line in lines[1:]:
    fields = [f.strip() for f in line.strip().split('|')]
    db_id = int(fields[0])
    
    if db_id not in target_ids:
        continue
    
    tx_type = int(fields[idx['TYPE']])
    tx_subtype = int(fields[idx['SUBTYPE']])
    att_hex = fields[idx['ATTACHMENT_BYTES']]
    has_msg = fields[idx['HAS_MESSAGE']]
    has_ppm = fields[idx['HAS_PRUNABLE_MESSAGE']]
    has_ppa = fields[idx['HAS_PRUNABLE_ATTACHMENT']]
    
    att_bytes = bytes.fromhex(att_hex)
    att_len = len(att_bytes)
    
    # 解析 attachment 结构
    version = att_bytes[0] if att_len > 0 else 0
    
    # 判断是否是 prunable hash (ver=1 + 32B data)
    is_prunable_hash = (att_len == 33 and version == 1)
    
    print(f"{db_id:>6} | {tx_type:>5}:{tx_subtype:<2} | {has_msg:>7} | {has_ppm:>7} | {has_ppa:>7} | {att_len:>7}B | {att_hex[:80]}...")
    
    # 详细分析
    if is_prunable_hash:
        print(f"       → Prunable Hash: ver={version}, hash={att_bytes[1:].hex()}")
    elif tx_type == 12 and tx_subtype == 0:
        # AliasAssignment
        name_len = att_bytes[1]
        name = att_bytes[2:2+name_len].decode('utf-8', errors='replace')
        remaining = att_bytes[2+name_len:]
        print(f"       → AliasAssignment: ver={version}, name=\"{name}\", remaining={remaining.hex()[:40]}...")
    elif tx_type == 2 and tx_subtype == 10:
        # PublicKeyAnnouncement
        pk = att_bytes[1:33]
        remaining = att_bytes[33:]
        print(f"       → PublicKeyAnnouncement: ver={version}, pk={pk.hex()[:20]}..., remaining={remaining.hex()[:30]}...")
    elif tx_type == 5 and tx_subtype == 0:
        # MonetarySystem
        print(f"       → MonetarySystem: ver={version}, data={att_hex[2:]}")
    elif tx_type == 0 and tx_subtype == 0:
        # Payment with prunable message
        print(f"       → Payment with prunable: ver={version}, hash={att_bytes[1:].hex()[:40]}...")
    elif tx_type == 6 and tx_subtype == 1:
        # Data subtype 1
        print(f"       → Data subtype 1: ver={version}, data={att_hex[2:40]}...")
