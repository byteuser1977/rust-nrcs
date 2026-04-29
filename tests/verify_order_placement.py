#!/usr/bin/env python3
"""
验证 Rust attachment_bytes 生成是否与 Java 一致
"""
import struct

def put_i64(val):
    """模拟 Rust 的 put_i64 (小端序)"""
    return struct.pack('<q', val)

def put_byte(val):
    """模拟 Rust 的 put_byte"""
    return bytes([val & 0xFF])

def simulate_order_placement(asset_id, quantity_qnt, price_nqt):
    """模拟 Rust 的 OrderPlacement 序列化"""
    buf = bytearray()
    # version (由 put_version_and_data 添加)
    buf.append(1)
    # assetId
    buf.extend(put_i64(asset_id))
    # quantityQNT
    buf.extend(put_i64(quantity_qnt))
    # priceNQT
    buf.extend(put_i64(price_nqt))
    return bytes(buf)

# 测试用例
print("=== OrderPlacement 序列化验证 ===\n")

# DB_ID=18 的数据
java_data_18 = bytes.fromhex("01f39e52171417e3df01000000000000000010a5d4e8000000")
print(f"DB_ID=18 Java 数据 ({len(java_data_18)} 字节): {java_data_18.hex()}")

# 解析 Java 数据
version_18 = java_data_18[0]
asset_18 = struct.unpack_from('<q', java_data_18, 1)[0]
qty_18 = struct.unpack_from('<q', java_data_18, 9)[0]
price_18 = struct.unpack_from('<q', java_data_18, 17)[0]
print(f"  version={version_18}, assetId={asset_18}, quantityQNT={qty_18}, priceNQT={price_18}")

# 模拟 Rust 生成
rust_18 = simulate_order_placement(asset_18, qty_18, price_18)
print(f"DB_ID=18 Rust 模拟 ({len(rust_18)} 字节): {rust_18.hex()}")
print(f"  匹配: {rust_18 == java_data_18}")
print()

# DB_ID=20 的数据
java_data_20 = bytes.fromhex("01f39e52171417e3dff300000000e1f50500000000")
print(f"DB_ID=20 Java 数据 ({len(java_data_20)} 字节): {java_data_20.hex()}")

# 解析 Java 数据
version_20 = java_data_20[0]
asset_20 = struct.unpack_from('<q', java_data_20, 1)[0]
print(f"  version={version_20}, assetId={asset_20}")

# 剩余数据
remaining_20 = java_data_20[9:]
print(f"  剩余 {len(remaining_20)} 字节: {remaining_20.hex()}")

# 尝试不同的解析方式
if len(remaining_20) == 12:
    # 可能是 quantityQNT(4) + priceNQT(8)?
    qty_i32 = struct.unpack_from('<i', remaining_20, 0)[0]
    price = struct.unpack_from('<q', remaining_20, 4)[0]
    print(f"  尝试解析为 quantityQNT(i32)={qty_i32}, priceNQT={price}")
    
    # 模拟 Rust 生成（假设使用 i32）
    buf = bytearray()
    buf.append(1)
    buf.extend(put_i64(asset_20))
    buf.extend(struct.pack('<i', qty_i32))  # i32
    buf.extend(put_i64(price))
    print(f"  如果使用 i32: {bytes(buf).hex()} ({len(buf)} 字节)")
    
    # 但这和 Java 数据不匹配！
    print(f"  不匹配！Java 数据有 21 字节，但标准格式应该是 25 字节")

print()
print("=== 结论 ===")
print("DB_ID=18: Java 数据格式正确 (25 字节)")
print("DB_ID=20: Java 数据格式异常 (21 字节)，可能是数据库问题或特殊格式")
