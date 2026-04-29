#!/usr/bin/env python3
"""
直接分析用户提供的数据
"""

# 用户提供的原始数据
records = [
    {"db_id": 18, "type": 2, "subtype": 2, "att": "01f39e52171417e3df01000000000000000010a5d4e8000000", "full_hash": "f1a52f6cddf08dc8f51f4229e536826446b15119ccc2e013fa4ff286d4b25ecc"},
    {"db_id": 20, "type": 2, "subtype": 2, "att": "01f39e52171417e3dff300000000e1f50500000000", "full_hash": "67d4840098b46febad5f23b7f57813882f4e76226e9d2dd0f8f64de416f29b8a"},
    {"db_id": 23, "type": 1, "subtype": 0, "att": "0120a26df7b6c94b475f3c20ce9bdb1ef8409f3079a0a5dc1b3cf246f485309c87", "full_hash": "a546e8db59204007db3b0cb08312f001155a17140039b8aa51b8317d44248a82", "has_ppm": True},
    {"db_id": 43, "type": 6, "subtype": 1, "att": "0115930917a7e0eabd", "full_hash": "3240fc1df3b19d6b460d6c6026a732b5484b1cdb17ad45c81610b2570085e372"},
    {"db_id": 55, "type": 0, "subtype": 0, "att": "0179f3221c559eaabf7d96163188bc256a05fa0907e29b1a3a6e487e706b6e9d93", "full_hash": "ca5d746307ed0cf1945e49e626a9d9ff180084b54bc267b703080af320e1ef57", "has_ppm": True},
    {"db_id": 58, "type": 2, "subtype": 10, "att": "01f39e52171417e3df026e6f06313233343536012d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a93", "full_hash": "66d5bbc8f0f9b1f78d541cb97314e42374c8a170fb756d5d449604b3defd5189", "has_pka": True},
    {"db_id": 65, "type": 12, "subtype": 1, "att": "01e5c6099a6a80f8e0", "full_hash": "0b9f62178466df745e2b673016991327e5cc961ebd803491e22477c505669c50"},
]

import struct

print("=== Java 数据库记录分析 ===\n")

for rec in records:
    data = bytes.fromhex(rec["att"])
    print(f"DB_ID={rec['db_id']}, Type={rec['type']}:{rec['subtype']}")
    print(f"  FULL_HASH: {rec['full_hash']}")
    print(f"  ATTACHMENT_BYTES ({len(data)}B): {rec['att']}")
    
    if rec["type"] == 2 and rec["subtype"] == 2:
        # OrderPlacement
        if len(data) == 25:
            version = data[0]
            asset = struct.unpack_from('<q', data, 1)[0]
            qty = struct.unpack_from('<q', data, 9)[0]
            price = struct.unpack_from('<q', data, 17)[0]
            print(f"  ✅ 标准格式: version={version}, asset={asset}, qty={qty}, price={price}")
        else:
            print(f"  ❌ 异常长度: {len(data)} 字节 (标准应为 25 字节)")
            # 尝试解析
            version = data[0]
            asset = struct.unpack_from('<q', data, 1)[0]
            print(f"     version={version}, asset={asset}")
            remaining = data[9:]
            print(f"     剩余: {remaining.hex()}")
    
    elif rec["type"] == 1 and rec["subtype"] == 0:
        print(f"  ✅ PrunablePlainMessage hash: version={data[0]}, hash={data[1:].hex()}")
    
    elif rec["type"] == 6 and rec["subtype"] == 1:
        version = data[0]
        tagged_id = struct.unpack_from('<q', data, 1)[0]
        print(f"  ✅ TaggedDataExtend: version={version}, taggedDataId={tagged_id}")
    
    elif rec["type"] == 0 and rec["subtype"] == 0:
        print(f"  ✅ Payment with PrunablePlainMessage: version={data[0]}, hash={data[1:].hex()}")
    
    elif rec["type"] == 2 and rec["subtype"] == 10:
        version = data[0]
        asset = struct.unpack_from('<q', data, 1)[0]
        prop_len = data[9]
        prop = data[10:10+prop_len]
        val_len = data[10+prop_len]
        val = data[11+prop_len:11+prop_len+val_len]
        remaining = data[11+prop_len+val_len:]
        print(f"  ✅ AssetPropertySet: version={version}, asset={asset}")
        print(f"     property={prop}, value={val}")
        print(f"     剩余 (PublicKeyAnnouncement): {remaining.hex()}")
    
    elif rec["type"] == 12 and rec["subtype"] == 1:
        version = data[0]
        ref_id = struct.unpack_from('<q', data, 1)[0]
        print(f"  ✅ ContractReferenceDelete: version={version}, refId={ref_id}")
    
    print()

print("=== 结论 ===")
print("DB_ID=18: 25 字节，格式正确 ✅")
print("DB_ID=20: 21 字节，格式异常 ❌ (可能是数据库问题)")
print("其他记录: 格式正确 ✅")
