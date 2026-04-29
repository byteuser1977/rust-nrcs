#!/usr/bin/env python3
"""
详细解析 DB_ID=56 的 attachment_bytes
"""
import struct

hex_str = "01f39e52171417e3df026e6f06313233343536012d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a93"
data = bytes.fromhex(hex_str)
print(f"Total length: {len(data)} bytes")
print(f"Hex: {hex_str}")
print()

# 逐字节解析
print("逐字节解析:")
for i, b in enumerate(data):
    print(f"  [{i:2d}] 0x{b:02x} ({b:3d})")

print()
print("=== 尝试不同解析方式 ===")

# 方式1: AssetPropertySet
print("\n方式1: AssetPropertySet")
offset = 0
version = data[offset]; offset += 1
print(f"  version = {version}")
asset_id = struct.unpack_from('<q', data, offset)[0]; offset += 8
print(f"  assetId = {asset_id}")
prop_len = data[offset]; offset += 1
prop = data[offset:offset+prop_len]
print(f"  property_len = {prop_len}, property = {prop}")
offset += prop_len
val_len = data[offset]; offset += 1
val = data[offset:offset+val_len]
print(f"  value_len = {val_len}, value = {val}")
offset += val_len
remaining = data[offset:]
print(f"  remaining ({len(remaining)}B) = {remaining.hex()}")

# 方式2: 检查是否有 PublicKeyAnnouncement
print("\n方式2: 检查 PublicKeyAnnouncement")
# 如果 HAS_PUBLIC_KEY_ANNOUNCEMENT=TRUE，那么 remaining 应该是 version + publicKey(32B)
if len(remaining) >= 1:
    pk_version = remaining[0]
    print(f"  pk_version = {pk_version}")
    if len(remaining) >= 33:
        pk = remaining[1:33]
        print(f"  publicKey = {pk.hex()}")
    else:
        print(f"  Not enough bytes for publicKey (need 32, have {len(remaining)-1})")
