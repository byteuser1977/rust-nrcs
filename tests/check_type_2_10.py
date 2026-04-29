#!/usr/bin/env python3
"""
检查所有 Type 2:10 的记录
"""
import struct

with open('/Volumes/DATA/data/develop/git/rust-nrcs/tests/transaction.data', 'r') as f:
    lines = f.readlines()

header = [h.strip() for h in lines[0].strip().split('|')]
idx = {h: i for i, h in enumerate(header)}

print("=== Type 2:10 (AssetPropertySet) 记录 ===\n")

for line in lines[1:]:
    fields = [f.strip() for f in line.strip().split('|')]
    db_id = int(fields[0])
    tx_type = int(fields[idx['TYPE']])
    tx_subtype = int(fields[idx['SUBTYPE']])
    
    if tx_type != 2 or tx_subtype != 10:
        continue
    
    att_hex = fields[idx['ATTACHMENT_BYTES']]
    has_pk = fields[idx['HAS_PUBLIC_KEY_ANNOUNCEMENT']]
    
    att_bytes = bytes.fromhex(att_hex)
    print(f"DB_ID={db_id}, HAS_PK={has_pk}, len={len(att_bytes)}B")
    print(f"  Hex: {att_hex}")
    
    # 解析 AssetPropertySet
    offset = 0
    version = att_bytes[offset]; offset += 1
    asset_id = struct.unpack_from('<q', att_bytes, offset)[0]; offset += 8
    prop_len = att_bytes[offset]; offset += 1
    prop = att_bytes[offset:offset+prop_len]; offset += prop_len
    val_len = att_bytes[offset]; offset += 1
    val = att_bytes[offset:offset+val_len]; offset += val_len
    remaining = att_bytes[offset:]
    
    print(f"  version={version}, assetId={asset_id}")
    print(f"  property=\"{prop.decode('utf-8', errors='replace')}\" ({prop_len}B)")
    print(f"  value=\"{val.decode('utf-8', errors='replace')}\" ({val_len}B)")
    print(f"  remaining ({len(remaining)}B) = {remaining.hex()}")
    print()
