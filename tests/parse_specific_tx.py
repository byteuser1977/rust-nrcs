#!/usr/bin/env python3
"""
精确解析 Type 12:0 和 Type 2:10 的 attachment_bytes
"""
import struct

def parse_alias_assignment(hex_str):
    """解析 AliasAssignment attachment"""
    data = bytes.fromhex(hex_str)
    offset = 0
    
    version = data[offset]; offset += 1
    print(f"  version = {version}")
    
    # aliasName (BYTE prefix)
    name_len = data[offset]; offset += 1
    name = data[offset:offset+name_len].decode('utf-8', errors='replace')
    offset += name_len
    print(f"  name_len = {name_len}, name = \"{name}\"")
    
    # aliasURI (SHORT prefix)
    uri_len = struct.unpack_from('<H', data, offset)[0]; offset += 2
    print(f"  uri_len = {uri_len}")
    if uri_len > 0:
        uri = data[offset:offset+uri_len].decode('utf-8', errors='replace')
        offset += uri_len
        print(f"  uri = \"{uri}\"")
    
    # 剩余数据
    remaining = data[offset:]
    print(f"  remaining ({len(remaining)}B) = {remaining.hex()}")
    
    # 尝试解析剩余数据
    if len(remaining) >= 8:
        val1 = struct.unpack_from('<Q', remaining, 0)[0]
        print(f"    作为 u64 LE: {val1} (0x{val1:016x})")
        if len(remaining) >= 40:
            hash_val = remaining[8:40]
            print(f"    作为 hash(32B): {hash_val.hex()}")


def parse_public_key_announcement(hex_str):
    """解析 PublicKeyAnnouncement attachment"""
    data = bytes.fromhex(hex_str)
    offset = 0
    
    version = data[offset]; offset += 1
    print(f"  version = {version}")
    
    # recipientPublicKey (32 bytes)
    pk = data[offset:offset+32]; offset += 32
    print(f"  publicKey(32B) = {pk.hex()}")
    
    # 剩余数据
    remaining = data[offset:]
    print(f"  remaining ({len(remaining)}B) = {remaining.hex()}")
    
    # 尝试解析剩余数据
    if len(remaining) >= 1:
        print(f"    第一个字节: {remaining[0]:02x}")
        if len(remaining) >= 2:
            len_field = struct.unpack_from('<H', remaining, 0)[0]
            print(f"    作为 u16 LE: {len_field}")


print("=== Type 12:0 (AliasAssignment) ===")
print("\nDB_ID=46:")
parse_alias_assignment("010a48656c6c6f576f726c6400000000000c63f641b8d467feec0aefc98b89313cf3570830ef95950214762fff4ad93b8b")

print("\nDB_ID=54:")
parse_alias_assignment("011a446973747269627574656452616e646f6d47656e657261746f7200000000009193ff27cfae35775252126725132334af")

print("\n=== Type 2:10 (PublicKeyAnnouncement) ===")
print("\nDB_ID=56:")
parse_public_key_announcement("01f39e52171417e3df026e6f06313233343536012d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a93")

print("\n=== Type 12:1 (AliasSell?) ===")
print("\nDB_ID=65:")
data = bytes.fromhex("01e5c6099a6a80f8e0")
print(f"  version = {data[0]}")
val = struct.unpack_from('<Q', data, 1)[0]
print(f"  value(u64 LE) = {val} (0x{val:016x})")
