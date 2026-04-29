#!/usr/bin/env python3
"""
检查特定 db_id 记录的 appendix 标志
"""

target_ids = [46, 54, 56, 57, 58, 60, 62, 65]

with open('/Volumes/DATA/data/develop/git/rust-nrcs/tests/transaction.data', 'r') as f:
    lines = f.readlines()

header = [h.strip() for h in lines[0].strip().split('|')]
idx = {h: i for i, h in enumerate(header)}

print("=== 检查 Type 12:0 和 Type 2:10 的 appendix 标志 ===\n")

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
    has_pk = fields[idx['HAS_PUBLIC_KEY_ANNOUNCEMENT']]
    has_enc = fields[idx['HAS_ENCRYPTED_MESSAGE']]
    has_pem = fields[idx['HAS_PRUNABLE_ENCRYPTED_MESSAGE']]
    
    print(f"DB_ID={db_id}, Type={tx_type}:{tx_subtype}")
    print(f"  HAS_MESSAGE={has_msg}")
    print(f"  HAS_ENCRYPTED_MESSAGE={has_enc}")
    print(f"  HAS_PUBLIC_KEY_ANNOUNCEMENT={has_pk}")
    print(f"  HAS_PRUNABLE_MESSAGE={has_ppm}")
    print(f"  HAS_PRUNABLE_ATTACHMENT={has_ppa}")
    print(f"  HAS_PRUNABLE_ENCRYPTED_MESSAGE={has_pem}")
    print(f"  ATTACHMENT_BYTES={att_hex}")
    print()
