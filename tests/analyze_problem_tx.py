#!/usr/bin/env python3
"""
分析有问题的 transaction 记录
"""
import struct

records = [
    {"db_id": 18, "type": 2, "subtype": 2, "att": "01f39e52171417e3df01000000000000000010a5d4e8000000"},
    {"db_id": 20, "type": 2, "subtype": 2, "att": "01f39e52171417e3dff300000000e1f50500000000"},
    {"db_id": 23, "type": 1, "subtype": 0, "att": "0120a26df7b6c94b475f3c20ce9bdb1ef8409f3079a0a5dc1b3cf246f485309c87"},
    {"db_id": 43, "type": 6, "subtype": 1, "att": "0115930917a7e0eabd"},
    {"db_id": 55, "type": 0, "subtype": 0, "att": "0179f3221c559eaabf7d96163188bc256a05fa0907e29b1a3a6e487e706b6e9d93"},
    {"db_id": 58, "type": 2, "subtype": 10, "att": "01f39e52171417e3df026e6f06313233343536012d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a93"},
    {"db_id": 65, "type": 12, "subtype": 1, "att": "01e5c6099a6a80f8e0"},
]

for rec in records:
    data = bytes.fromhex(rec["att"])
    print(f"\n=== DB_ID={rec['db_id']}, Type={rec['type']}:{rec['subtype']} ===")
    print(f"  Total: {len(data)} bytes")
    print(f"  Hex: {rec['att']}")
    
    if rec["type"] == 2 and rec["subtype"] == 2:
        # ColoredCoins OrderPlacement (AskOrder)
        # Java: buffer.putLong(assetId) + buffer.putLong(quantityQNT) + buffer.putLong(priceNQT)
        offset = 0
        version = data[offset]; offset += 1
        print(f"  version = {version}")
        
        asset_id = struct.unpack_from('<q', data, offset)[0]; offset += 8
        print(f"  assetId = {asset_id}")
        
        qty = struct.unpack_from('<q', data, offset)[0]; offset += 8
        print(f"  quantityQNT = {qty}")
        
        price = struct.unpack_from('<q', data, offset)[0]; offset += 8
        print(f"  priceNQT = {price}")
        
        remaining = data[offset:]
        if remaining:
            print(f"  REMAINING ({len(remaining)}B) = {remaining.hex()} <- 问题！应该没有剩余字节")
    
    elif rec["type"] == 1 and rec["subtype"] == 0:
        # Messaging ArbitraryMessage - 应该是 PrunablePlainMessage hash
        print(f"  version = {data[0]}")
        print(f"  hash(32B) = {data[1:].hex()}")
        print(f"  -> 这是 PrunablePlainMessage hash 格式")
    
    elif rec["type"] == 6 and rec["subtype"] == 1:
        # Data TaggedDataExtend
        print(f"  version = {data[0]}")
        tagged_data_id = struct.unpack_from('<q', data, 1)[0]
        print(f"  taggedDataId = {tagged_data_id}")
    
    elif rec["type"] == 0 and rec["subtype"] == 0:
        # Payment with PrunablePlainMessage
        print(f"  version = {data[0]}")
        print(f"  hash(32B) = {data[1:].hex()}")
        print(f"  -> Payment with PrunablePlainMessage hash")
    
    elif rec["type"] == 2 and rec["subtype"] == 10:
        # ColoredCoins AssetPropertySet
        offset = 0
        version = data[offset]; offset += 1
        print(f"  version = {version}")
        
        asset_id = struct.unpack_from('<q', data, offset)[0]; offset += 8
        print(f"  assetId = {asset_id}")
        
        prop_len = data[offset]; offset += 1
        prop = data[offset:offset+prop_len]; offset += prop_len
        print(f"  property = {prop}")
        
        val_len = data[offset]; offset += 1
        val = data[offset:offset+val_len]; offset += val_len
        print(f"  value = {val}")
        
        remaining = data[offset:]
        print(f"  REMAINING ({len(remaining)}B) = {remaining.hex()}")
        print(f"  -> 这应该是 PublicKeyAnnouncement appendix")
    
    elif rec["type"] == 12 and rec["subtype"] == 1:
        # LightContract ContractReferenceDelete
        print(f"  version = {data[0]}")
        ref_id = struct.unpack_from('<q', data, 1)[0]
        print(f"  contractReferenceId = {ref_id}")
