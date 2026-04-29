#!/usr/bin/env python3
"""
精确分析 transaction.data 中每条记录的 attachment_bytes 二进制结构
"""
import struct

def decode_attachment_bytes(hex_str, type_byte, subtype):
    """解码 attachment_bytes 的二进制结构"""
    data = bytes.fromhex(hex_str)
    parts = []
    offset = 0
    
    def read_bytes(n):
        nonlocal offset
        val = data[offset:offset+n]
        offset += n
        return val
    
    def read_u8():
        nonlocal offset
        val = data[offset]
        offset += 1
        return val
    
    def read_u32_le():
        nonlocal offset
        val = struct.unpack_from('<I', data, offset)[0]
        offset += 4
        return val
    
    def read_i32_le():
        nonlocal offset
        val = struct.unpack_from('<i', data, offset)[0]
        offset += 4
        return val
    
    def read_u64_le():
        nonlocal offset
        val = struct.unpack_from('<Q', data, offset)[0]
        offset += 8
        return val
    
    def read_i64_le():
        nonlocal offset
        val = struct.unpack_from('<q', data, offset)[0]
        offset += 8
        return val
    
    total_len = len(data)
    
    # === Attachment 部分 ===
    if offset < total_len:
        version = read_u8()
        parts.append(f"ver={version}")
        
        if type_byte == 1 and subtype == 9:
            # PhasingVoteCasting
            count = read_u8()
            parts.append(f"count={count}")
            for i in range(count):
                hash_val = read_bytes(32)
                parts.append(f"hash[{i}]={hash_val[:16].hex()}...")
            
            # 检查剩余字节
            remaining = total_len - offset
            if remaining > 0:
                trail = read_bytes(remaining)
                parts.append(f"TRAIL[{remaining}B]={trail.hex()}")
                
        elif type_byte == 1 and subtype == 0:
            # ArbitraryMessage (Message appendix)
            len_field = read_i32_le()
            is_text = (len_field & 0x80000000) != 0
            actual_len = len_field & 0x7FFFFFFF
            parts.append(f"msg_len={len_field}(raw) isText={is_text} actual={actual_len}")
            if actual_len > 0 and offset + actual_len <= total_len:
                msg_data = read_bytes(actual_len)
                try:
                    msg_str = msg_data.decode('utf-8')
                    if all(32 <= ord(c) < 127 or c in '\n\r\t' for c in msg_str):
                        parts.append(f'msg="{msg_str[:50]}"')
                    else:
                        parts.append(f'msg({actual_len}B)={msg_data[:20].hex()}...')
                except:
                    parts.append(f'msg({actual_len}B)={msg_data[:20].hex()}...')
            elif actual_len > 0:
                parts.append(f"MSG_OVERFLOW: need {actual_len}B but only {total_len-offset}B left")
            
            # 检查是否有 trailing bytes after message
            if offset < total_len:
                remaining = total_len - offset
                trail = read_bytes(remaining)
                parts.append(f"extra_after_msg[{remaining}B]={trail.hex()}")
                
        elif type_byte == 6 and subtype == 0:
            # PrunablePlainMessage (only hash stored)
            hash_data = read_bytes(total_len - offset)
            parts.append(f"prunable_hash({len(hash_data)}B)={hash_data.hex()}")
            
        elif type_byte == 12 and subtype == 0:
            # AliasAssignment
            name_len = read_u8()
            name = read_bytes(name_len).decode('utf-8', errors='replace')
            parts.append(f'name_len={name_len} alias="{name}"')
            price = read_u64_le()
            parts.append(f'price={price}')
            
            # 检查 trailing
            if offset < total_len:
                remaining = total_len - offset
                trail = read_bytes(remaining)
                parts.append(f"extra_after_price[{remaining}B]={trail.hex()}")
                
        elif type_byte == 2 and subtype == 10:
            # PublicKeyAnnouncement
            pk = read_bytes(32)
            parts.append(f'pk={pk[:16].hex()}...')
            
            # 检查 trailing
            if offset < total_len:
                remaining = total_len - offset
                trail = read_bytes(remaining)
                parts.append(f"extra_after_pk[{remaining}B]={trail.hex()}")
                
        elif type_byte == 5 and subtype == 0:
            # AssetTransfer
            asset = read_u64_le()
            qty = read_u64_le()
            parts.append(f'asset={asset} qty={qty}')
            
            # 检查 trailing
            if offset < total_len:
                remaining = total_len - offset
                trail = read_bytes(remaining)
                parts.append(f"extra_after_qty[{remaining}B]={trail.hex()}")
                
        elif type_byte == 0 and subtype == 0:
            # Ordinary Payment - should be empty or have message appendix
            # If has data after version, it's a Message appendix
            if offset < total_len:
                len_field = read_i32_le()
                is_text = (len_field & 0x80000000) != 0
                actual_len = len_field & 0x7FFFFFFF
                parts.append(f"PAYMENT_MSG: len={len_field}(raw) isText={is_text} actual={actual_len}")
                if actual_len > 0 and offset + actual_len <= total_len:
                    msg_data = read_bytes(actual_len)
                    try:
                        msg_str = msg_data.decode('utf-8')
                        parts.append(f'content="{msg_str[:50]}"')
                    except:
                        parts.append(f'content({actual_len}B)={msg_data[:20].hex()}...')
                        
        else:
            # Unknown type - dump raw remaining
            remaining = total_len - offset
            raw = read_bytes(remaining)
            parts.append(f"RAW_DATA[{remaining}B]={raw.hex()}")
    
    return parts, total_len


def main():
    with open('/Volumes/DATA/data/develop/git/rust-nrcs/tests/transaction.data', 'r') as f:
        lines = f.readlines()
    
    header = lines[0].strip().split('|')
    # 清理字段名中的前后空格
    header = [h.strip() for h in header]
    print("=== HEADER ===")
    for i, h in enumerate(header):
        print(f"  [{i}] {h}")
    print()
    
    # 找到关键字段的索引
    idx = {h: i for i, h in enumerate(header)}
    TYPE_IDX = idx['TYPE']
    SUBTYPE_IDX = idx['SUBTYPE']
    ATTACHMENT_BYTES_IDX = idx['ATTACHMENT_BYTES']
    HAS_MESSAGE_IDX = idx['HAS_MESSAGE']
    HAS_PRUNABLE_MESSAGE_IDX = idx['HAS_PRUNABLE_MESSAGE']
    HAS_PK_ANNOUNCEMENT_IDX = idx['HAS_PUBLIC_KEY_ANNOUNCEMENT']
    
    print("=" * 120)
    print(f"{'#':>3} {'TYPE':>5} {'HAS_MSG':>7} {'HAS_PPM':>7} {'HAS_PK':>6} | {'LEN':>6} | DECODED STRUCTURE")
    print("=" * 120)
    
    # 统计各类型
    type_stats = {}
    
    for line_num, line in enumerate(lines[1:], start=1):
        fields = line.strip().split('|')
        # 清理字段值中的前后空格
        fields = [f.strip() for f in fields]
        if len(fields) < ATTACHMENT_BYTES_IDX + 1:
            continue
        
        db_id = fields[0]
        tx_type = int(fields[TYPE_IDX])
        tx_subtype = int(fields[SUBTYPE_IDX])
        att_hex = fields[ATTACHMENT_BYTES_IDX]
        has_msg = fields[HAS_MESSAGE_IDX]
        has_ppm = fields[HAS_PRUNABLE_MESSAGE_IDX]
        has_pk = fields[HAS_PK_ANNOUNCEMENT_IDX]
        
        type_key = f"{tx_type}:{tx_subtype}"
        if type_key not in type_stats:
            type_stats[type_key] = []
        type_stats[type_key].append(line_num)
        
        try:
            parts, total_len = decode_attachment_bytes(att_hex, tx_type, tx_subtype)
            decoded = " | ".join(parts)
        except Exception as e:
            decoded = f"ERROR: {e}"
        
        print(f"#{line_num:>3} {type_key:>5} {has_msg:>7} {has_ppm:>7} {has_pk:>6} | {total_len:>6}B | {decoded}")
    
    print("\n" + "=" * 120)
    print("=== TYPE SUMMARY ===")
    for tk, lines_list in sorted(type_stats.items()):
        print(f"  Type {tk}: {len(lines_list)} records (lines: {lines_list})")


if __name__ == '__main__':
    main()
