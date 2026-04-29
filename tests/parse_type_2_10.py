#!/usr/bin/env python3
"""
精确解析 Type 2:10 (AssetPropertySet) 的 attachment_bytes
"""
import struct

def parse_asset_property(hex_str):
    """解析 AssetPropertySet attachment"""
    data = bytes.fromhex(hex_str)
    offset = 0
    
    version = data[offset]; offset += 1
    print(f"  version = {version}")
    
    # assetId (8 bytes)
    asset_id = struct.unpack_from('<q', data, offset)[0]; offset += 8
    print(f"  assetId = {asset_id} (0x{asset_id:016x})")
    
    # property (BYTE prefix)
    prop_len = data[offset]; offset += 1
    prop = data[offset:offset+prop_len].decode('utf-8', errors='replace')
    offset += prop_len
    print(f"  property_len = {prop_len}, property = \"{prop}\"")
    
    # value (UBYTE prefix)
    val_len = data[offset]; offset += 1
    val = data[offset:offset+val_len].decode('utf-8', errors='replace')
    offset += val_len
    print(f"  value_len = {val_len}, value = \"{val}\"")
    
    # 剩余数据
    remaining = data[offset:]
    print(f"  remaining ({len(remaining)}B) = {remaining.hex()}")


print("=== Type 2:10 (AssetPropertySet) ===")
print("\nDB_ID=56:")
parse_asset_property("01f39e52171417e3df026e6f06313233343536012d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a93")

print("\n=== 验证解析 ===")
# 手动解析
data = bytes.fromhex("01f39e52171417e3df026e6f06313233343536012d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a93")
print(f"Total length: {len(data)} bytes")
print(f"  [0] version = {data[0]}")
print(f"  [1:9] assetId = {data[1:9].hex()}")
print(f"  [9] property_len = {data[9]}")
print(f"  [10:{10+data[9]}] property = {data[10:10+data[9]]}")
offset = 10 + data[9]
print(f"  [{offset}] value_len = {data[offset]}")
print(f"  [{offset+1}:{offset+1+data[offset]}] value = {data[offset+1:offset+1+data[offset]]}")
