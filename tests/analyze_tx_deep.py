#!/usr/bin/env python3
"""
深入分析 transaction.data 中特定类型记录的 attachment_bytes 结构
重点关注 trailing bytes 的模式
"""
import struct
import json

def analyze_type_1_9():
    """分析 Type 1:9 (PhasingVoteCasting) 的完整结构"""
    print("=" * 80)
    print("Type 1:9 (PhasingVoteCasting) 详细分析")
    print("=" * 80)

    records = [
        {
            "hex": "0101f0dfef7be95acd764476bca5913dcfb3a3e72f6eed6c26a638194558be760bbc00000000",
            "line": 1,
        },
        {
            "hex": "0101f0dfef7be95acd764476bca5913dcfb3a3e72f6eed6c26a638194558be760bbc00000000",
            "line": 2,
        },
    ]

    for rec in records:
        data = bytes.fromhex(rec["hex"])
        print(f"\n记录 #{rec['line']}: {len(data)} bytes")
        print(f"  Hex: {rec['hex']}")

        offset = 0
        version = data[offset]; offset += 1
        print(f"  [{offset-1}] version = {version}")

        count = data[offset]; offset += 1
        print(f"  [{offset-1}] count = {count}")

        for i in range(count):
            hash_val = data[offset:offset+32]
            offset += 32
            print(f"  [{offset-32}:{offset}] fullHash[{i}] = {hash_val.hex()}")

        # 剩余字节
        remaining = data[offset:]
        print(f"  [{offset}:{len(data)}] TRAILING = {remaining.hex()} ({len(remaining)} bytes)")

        # 尝试解释 trailing bytes
        if len(remaining) == 4:
            val_u32 = struct.unpack_from('<I', remaining, 0)[0]
            val_i32 = struct.unpack_from('<i', remaining, 0)[0]
            print(f"    作为 u32 LE: {val_u32} (0x{val_u32:08x})")
            print(f"    作为 i32 LE: {val_i32}")
            print(f"    作为 4 x u8: {list(remaining)}")


def analyze_type_12_0():
    """分析 Type 12:0 (AliasAssignment/LightContract) 的完整结构"""
    print("\n" + "=" * 80)
    print("Type 12:0 (AliasAssignment) 详细分析")
    print("=" * 80)

    records = [
        {"hex": "010a48656c6c6f576f726c6400000000000c63f641b8d467feec0aefc98b89313cf3570830ef95950214762fff4ad93b8b", "line": 20},
        {"hex": "011a4469737472696275746564416e646f6d47656e657261746f7200000000009193ff27cfae35775252126725132334af1397d58d784ee5064d636f244b425d", "line": 28},
        {"hex": "010a48656c6c6f576f726c6400000000008133d0b6dc69842183feeb8626ac37407470ef19ea330d8011a788d0b7929b80", "line": 60},
    ]

    for rec in records:
        data = bytes.fromhex(rec["hex"])
        print(f"\n记录 #{rec['line']}: {len(data)} bytes")
        print(f"  Hex: {rec['hex']}")

        offset = 0
        version = data[offset]; offset += 1
        print(f"  [{offset-1}] version = {version}")

        name_len = data[offset]; offset += 1
        name = data[offset:offset+name_len].decode('utf-8', errors='replace')
        offset += name_len
        print(f"  [{offset-name_len-1}] name_len={name_len}, alias=\"{name}\"")

        price = struct.unpack_from('<Q', data, offset)[0]; offset += 8
        print(f"  [{offset-8}:{offset}] price(u64 LE) = {price} (0x{price:016x})")

        # 剩余字节
        remaining = data[offset:]
        print(f"  [{offset}:{len(data)}] TRAILING ({len(remaining)} bytes) = {remaining.hex()}")

        if len(remaining) >= 32:
            print(f"    可能是 Hash256: {remaining[:32].hex()}")
        if len(remaining) > 32:
            print(f"    额外: {remaining[32:].hex()}")


def analyze_type_2_10():
    """分析 Type 2:10 (PublicKeyAnnouncement) 的完整结构"""
    print("\n" + "=" * 80)
    print("Type 2:10 (PublicKeyAnnouncement) 详细分析")
    print("=" * 80)

    records = [
        {"hex": "01f39e52171417e3df026e6f06313233343536012d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a93", "line": 30},
    ]

    for rec in records:
        data = bytes.fromhex(rec["hex"])
        print(f"\n记录 #{rec['line']}: {len(data)} bytes")
        print(f"  Hex: {rec['hex']}")

        offset = 0
        version = data[offset]; offset += 1
        print(f"  [{offset-1}] version = {version}")

        pk = data[offset:offset+32]; offset += 32
        print(f"  [{offset-32}:{offset}] publicKey(32B) = {pk.hex()}")

        # 剩余字节
        remaining = data[offset:]
        print(f"  [{offset}:{len(data)}] TRAILING ({len(remaining)} bytes) = {remaining.hex()}")

        # 尝试解释
        if len(remaining) >= 17:
            # 可能是 ASCII 文本?
            try:
                text = remaining.decode('ascii')
                if all(32 <= ord(c) < 127 for c in text):
                    print(f"    作为 ASCII: \"{text}\"")
                else:
                    print(f"    非纯ASCII")
            except:
                pass

            # 可能是 sender public key 的某种 hash?
            print(f"    前16字节: {remaining[:16].hex()}")


def analyze_type_5_0():
    """分析 Type 5:0 (AssetTransfer/MonetarySystem) 的完整结构"""
    print("\n" + "=" * 80)
    print("Type 5:0 (MonetarySystem?) 详细分析")
    print("=" * 80)

    records = [
        {"hex": "01044e555344044e5553441b00e4b88e555344e4bbb7e6a0bce9949ae5ae9ae79a84e7a7afe588863340420f000000000000", "line": 39},
    ]

    for rec in records:
        data = bytes.fromhex(rec["hex"])
        print(f"\n记录 #{rec['line']}: {len(data)} bytes")
        print(f"  Hex: {rec['hex']}")

        offset = 0
        version = data[offset]; offset += 1
        print(f"  [{offset-1}] version = {version}")

        asset = struct.unpack_from('<Q', data, offset)[0]; offset += 8
        print(f"  [{offset-8}:{offset}] asset(u64 LE) = {asset} (0x{asset:016x})")

        qty = struct.unpack_from('<Q', data, offset)[0]; offset += 8
        print(f"  [{offset-8}:{offset}] quantity(u64 LE) = {qty} (0x{qty:016x})")

        # 剩余字节
        remaining = data[offset:]
        print(f"  [{offset}:{len(data)}] TRAILING ({len(remaining)} bytes) = {remaining.hex()}")

        # 尝试作为字符串解读
        try:
            # 检查是否是 UTF-8 编码的文本
            text = remaining.decode('utf-8', errors='replace')
            printable = ''.join(c if 32 <= ord(c) < 127 else '.' for c in text)
            print(f"    UTF-8 尝试: \"{printable}\"")
        except:
            pass


def analyze_type_1_0_and_6_0():
    """对比分析 Type 1:0 和 Type 6:0（都显示为 ver + 32B hash）"""
    print("\n" + "=" * 80)
    print("Type 1:0 vs Type 6:0 对比分析")
    print("=" * 80)

    type_1_0_records = [
        {"hex": "0120a26df7b6c94b475f3c20ce9bdb1ef8409f3079a0a5dc1b3cf246f485309c87", "line": 3, "has_ppm": False},
        {"hex": "01729aad0f1430eed237754913c708f90863310b799b44a1916ddbf76de45668fa", "line": 8, "has_ppm": True},
    ]

    type_6_0_records = [
        {"hex": "01749cdbc29f1d2ee7f3e3fc65f64b97912b699e31324ebed4c9fdeb84eb42c51c", "line": 4},
        {"hex": "0118240be5d898ad3734cf2ec1569c864c7ad14749563d0c076443d779b43263ac", "line": 5},
    ]

    print("\n--- Type 1:0 (Messaging.ArbitraryMessage) ---")
    for rec in type_1_0_records:
        data = bytes.fromhex(rec["hex"])
        print(f"\n记录 #{rec['line']} (HAS_PRUNABLE_MESSAGE={rec.get('has_ppm', '?')}): {len(data)} bytes")
        print(f"  ver={data[0]}, data(32B)={data[1:].hex()}")

    print("\n--- Type 6:0 (Data.TaggedDataUpload/PrunablePlainMessage?) ---")
    for rec in type_6_0_records:
        data = bytes.fromhex(rec["hex"])
        print(f"\n记录 #{rec['line']}: {len(data)} bytes")
        print(f"  ver={data[0]}, data(32B)={data[1:].hex()}")


if __name__ == '__main__':
    analyze_type_1_9()
    analyze_type_12_0()
    analyze_type_2_10()
    analyze_type_5_0()
    analyze_type_1_0_and_6_0()
