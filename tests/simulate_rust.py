#!/usr/bin/env python3
"""
模拟 Rust 的 build_attachment_bytes_from_json 函数
"""
import struct
import json

def put_byte(buf, val):
    buf.append(val & 0xFF)

def put_i64(buf, val):
    buf.extend(struct.pack('<q', val))

def put_bytes(buf, data):
    buf.extend(data)

def put_version_and_data(buf, version, data):
    if version > 0:
        buf.append(version)
    buf.extend(data)

def serialize_order_placement(att_map, version):
    """模拟 Rust 的 serialize_colored_coins_attachment(OrderPlacement)"""
    buf = bytearray()
    
    # asset
    asset = att_map.get("asset", "0")
    if isinstance(asset, str):
        asset_val = int(asset)
        # 转换为有符号 i64
        if asset_val > 9223372036854775807:
            asset_val = asset_val - 18446744073709551616
    else:
        asset_val = int(asset)
    put_i64(buf, asset_val)
    
    # quantityQNT (注意: Rust 代码检查 quantityQQT 或 quantityQNT)
    qty = att_map.get("quantityQNT", "0")
    if isinstance(qty, str):
        qty_val = int(qty)
    else:
        qty_val = int(qty)
    put_i64(buf, qty_val)
    
    # priceNQT
    price = att_map.get("priceNQT", "0")
    if isinstance(price, str):
        price_val = int(price)
    else:
        price_val = int(price)
    put_i64(buf, price_val)
    
    return bytes(buf)

def build_attachment_bytes_from_json(type_byte, subtype, version, att_map):
    """模拟 Rust 的 build_attachment_bytes_from_json"""
    result = bytearray()
    
    # 1. 序列化 Attachment 部分
    if type_byte == 2 and subtype == 2:  # OrderPlacement
        att_bytes = serialize_order_placement(att_map, version)
        if len(att_bytes) > 0:
            put_version_and_data(result, version, att_bytes)
    
    # 2. 序列化各 Appendix 部分
    # ... (省略其他 appendix)
    
    return bytes(result)

# 测试用例
print("=== 模拟 Rust build_attachment_bytes_from_json ===\n")

# DB_ID=18 的数据
java_data_18 = bytes.fromhex("01f39e52171417e3df01000000000000000010a5d4e8000000")
print(f"DB_ID=18 Java 数据 ({len(java_data_18)}B): {java_data_18.hex()}")

# 解析 Java 数据
version_18 = java_data_18[0]
asset_18 = struct.unpack_from('<q', java_data_18, 1)[0]
qty_18 = struct.unpack_from('<q', java_data_18, 9)[0]
price_18 = struct.unpack_from('<q', java_data_18, 17)[0]
print(f"  解析: version={version_18}, asset={asset_18}, qty={qty_18}, price={price_18}")

# 模拟 Rust 从 JSON 生成
att_map_18 = {
    "asset": str(asset_18 & 0xFFFFFFFFFFFFFFFF),  # 无符号字符串
    "quantityQNT": qty_18,
    "priceNQT": price_18,
}
rust_18 = build_attachment_bytes_from_json(2, 2, version_18, att_map_18)
print(f"DB_ID=18 Rust 模拟 ({len(rust_18)}B): {rust_18.hex()}")
print(f"  匹配: {rust_18 == java_data_18}")
print()

# DB_ID=20 的数据
java_data_20 = bytes.fromhex("01f39e52171417e3dff300000000e1f50500000000")
print(f"DB_ID=20 Java 数据 ({len(java_data_20)}B): {java_data_20.hex()}")

# 解析 Java 数据
version_20 = java_data_20[0]
asset_20 = struct.unpack_from('<q', java_data_20, 1)[0]
print(f"  解析: version={version_20}, asset={asset_20}")

# 剩余数据
remaining_20 = java_data_20[9:]
print(f"  剩余: {remaining_20.hex()}")

# 尝试解析为 i32 + i64
if len(remaining_20) == 12:
    qty_i32 = struct.unpack_from('<i', remaining_20, 0)[0]
    price = struct.unpack_from('<q', remaining_20, 4)[0]
    print(f"  尝试解析为 qty(i32)={qty_i32}, price={price}")
    
    # 模拟 Rust 从 JSON 生成（假设使用这些值）
    att_map_20 = {
        "asset": str(asset_20 & 0xFFFFFFFFFFFFFFFF),
        "quantityQNT": qty_i32,
        "priceNQT": price,
    }
    rust_20 = build_attachment_bytes_from_json(2, 2, version_20, att_map_20)
    print(f"DB_ID=20 Rust 模拟 ({len(rust_20)}B): {rust_20.hex()}")
    print(f"  注意: Rust 生成 25 字节，但 Java 数据只有 21 字节！")
