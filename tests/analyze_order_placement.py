#!/usr/bin/env python3
"""
详细分析 OrderPlacement 记录
"""
import struct

# 用户提供的 Java 数据库数据
records = [
    {"db_id": 18, "type": 2, "subtype": 2, "att": "01f39e52171417e3df01000000000000000010a5d4e8000000"},
    {"db_id": 20, "type": 2, "subtype": 2, "att": "01f39e52171417e3dff300000000e1f50500000000"},
]

print("=== Java 数据库中的 OrderPlacement 记录分析 ===\n")

for rec in records:
    data = bytes.fromhex(rec["att"])
    print(f"DB_ID={rec['db_id']}: {len(data)} bytes")
    print(f"  Hex: {rec['att']}")
    
    # OrderPlacement 格式: version(1) + assetId(8) + quantityQNT(8) + priceNQT(8) = 25 bytes
    if len(data) >= 25:
        version = data[0]
        asset_id = struct.unpack_from('<q', data, 1)[0]
        qty = struct.unpack_from('<q', data, 9)[0]
        price = struct.unpack_from('<q', data, 17)[0]
        print(f"  version={version}, assetId={asset_id}, quantityQNT={qty}, priceNQT={price}")
    else:
        # 数据不完整，尝试部分解析
        print(f"  数据长度不足 25 字节！")
        version = data[0]
        asset_id = struct.unpack_from('<q', data, 1)[0]
        print(f"  version={version}, assetId={asset_id}")
        remaining = data[9:]
        print(f"  剩余 {len(remaining)} 字节: {remaining.hex()}")
        
        # 尝试不同的解析方式
        # 可能是 version(1) + assetId(8) + quantityQNT(4) + priceNQT(8) = 21 bytes?
        if len(data) == 21:
            qty_i32 = struct.unpack_from('<i', data, 9)[0]
            price = struct.unpack_from('<q', data, 13)[0]
            print(f"  尝试解析为 assetId(8) + quantityQNT(4) + priceNQT(8):")
            print(f"    quantityQNT(i32)={qty_i32}, priceNQT={price}")
    print()

# 模拟 Rust 当前实现生成的数据
print("=== Rust 当前实现模拟 ===\n")

def simulate_rust_order_placement(asset_str, qty_str, price_str):
    """模拟 Rust 当前的序列化实现"""
    buf = bytearray()
    # version
    buf.append(1)
    # assetId (i64)
    asset = int(asset_str) if asset_str else 0
    buf.extend(struct.pack('<q', asset))
    # quantityQNT (i64)
    qty = int(qty_str) if qty_str else 0
    buf.extend(struct.pack('<q', qty))
    # priceNQT (i64)
    price = int(price_str) if price_str else 0
    buf.extend(struct.pack('<q', price))
    return bytes(buf)

# 假设的 JSON 数据
test_cases = [
    {"asset": "-2313980408480227597", "quantityQNT": "1", "priceNQT": "1000000000000"},
    {"asset": "-2313980408480227597", "quantityQNT": "429496729600000243", "priceNQT": "0"},
]

for i, tc in enumerate(test_cases):
    result = simulate_rust_order_placement(tc["asset"], tc["quantityQNT"], tc["priceNQT"])
    print(f"测试用例 {i+1}: {len(result)} bytes")
    print(f"  输入: {tc}")
    print(f"  输出: {result.hex()}")
    print()
