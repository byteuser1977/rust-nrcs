#!/usr/bin/env python3
"""分析 transaction.data - 正确解析 attachment_bytes 结构（修正字段索引）"""
import struct

with open('tests/transaction.data') as f:
    lines = f.readlines()

print(f'Total records: {len(lines)-1}')
print()
print("=" * 200)
print(f'{"#":>3} {"T:S":>5} {"VER":>4} {"PHASED":>6} {"PRUN_MSG":>8} {"PRUN_ATT":>8} | {"ATT_LEN":>6} | DECODED_STRUCTURE')
print("=" * 200)

for i, line in enumerate(lines[1:], 1):
    fields = [f.strip() for f in line.strip().split('|')]
    if len(fields) < 29:
        continue
    
    tx_type = int(fields[11])
    subtype = int(fields[12])
    version = int(fields[19])
    full_hash = fields[6][:24]
    att_hex = fields[18]  # ATTACHMENT_BYTES is at index 18!
    phased = fields[17]
    has_prunable_msg = fields[23]
    has_prunable_att = fields[24]
    
    try:
        if not att_hex or att_hex.upper() == 'FALSE' or att_hex.upper() == 'NULL':
            print(f'#{i:>3} {tx_type}:{subtype:<2} {version:>4} {phased:>6} {has_prunable_msg:>8} {has_prunable_att:>8} | {"EMPTY":>6} | *** NO BINARY DATA ***')
            continue
            
        att_bytes = bytes.fromhex(att_hex)
        off = 0
        parts = []
        
        if len(att_bytes) > 0 and version == 1:
            parts.append(f'ver={att_bytes[0]:02x}')
            off = 1
        
        if tx_type == 1 and subtype == 9:  # PhasingVoteCasting
            if off < len(att_bytes):
                cnt = att_bytes[off]; off += 1
                parts.append(f'count={cnt}')
                for c in range(min(cnt, 3)):
                    if off + 32 <= len(att_bytes):
                        h = att_bytes[off:off+32].hex()
                        parts.append(f'hash[{c}]={h[:20]}..')
                        off += 32
                if off < len(att_bytes):
                    trailing = att_bytes[off:].hex()
                    tlen = len(att_bytes) - off
                    parts.append(f'TRAIL[{tlen}B]={trailing}')
        
        elif tx_type == 1 and subtype == 0:  # ArbitraryMessage (may have Message appendix)
            if off + 4 <= len(att_bytes):
                lf = struct.unpack_from('<i', att_bytes, off)[0]
                is_text = lf < 0
                msg_len = lf & 0x7fffffff
                parts.append(f'msg_len={msg_len} txt={is_text}')
                off += 4
                if off + msg_len <= len(att_bytes):
                    raw = att_bytes[off:off+msg_len]
                    try:
                        preview = raw.decode('utf8', errors='replace')[:25]
                        parts.append(f'msg="{preview}"')
                    except:
                        parts.append(f'msg_hex={raw.hex()[:30]}')
                    off += msg_len
            if off < len(att_bytes):
                parts.append(f'extra={att_bytes[off:].hex()[:30]}')
        
        elif tx_type == 12 and subtype == 0:  # AliasAssignment
            if off < len(att_bytes):
                nl = att_bytes[off]; off += 1; parts.append(f'name_len={nl}')
                if off + nl <= len(att_bytes):
                    name = att_bytes[off:off+nl].decode('utf8', errors='replace')
                    parts.append(f'alias="{name}"'); off += nl
            if off + 8 <= len(att_bytes):
                price = struct.unpack_from('<Q', att_bytes, off)[0]; off += 8
                parts.append(f'price={price}')
            if off < len(att_bytes):
                parts.append(f'extra={att_bytes[off:].hex()[:30]}')
        
        elif tx_type == 2 and subtype == 10:  # PublicKeyAnnouncement
            if off + 32 <= len(att_bytes):
                pk = att_bytes[off:off+32].hex(); off += 32
                parts.append(f'pk={pk[:28]}..')
            if off < len(att_bytes):
                parts.append(f'extra={att_bytes[off:].hex()[:30]}')
        
        elif tx_type == 5 and subtype == 0:  # AssetTransfer
            if off + 16 <= len(att_bytes):
                aid = struct.unpack_from('<Q', att_bytes, off)[0]; off += 8
                qty = struct.unpack_from('<Q', att_bytes, off)[0]; off += 8
                parts.append(f'asset={aid} qty={qty}')
            if off < len(att_bytes):
                parts.append(f'extra={att_bytes[off:].hex()[:30]}')
        
        elif tx_type == 2 and subtype == 0:  # AssetIssuance
            if off < len(att_bytes):
                nl = att_bytes[off]; off += 1; parts.append(f'name_len={nl}')
                if off + nl <= len(att_bytes):
                    name = att_bytes[off:off+nl].decode('utf8', errors='replace')
                    parts.append(f'name="{name}"'); off += nl
            if off + 2 <= len(att_bytes):
                dl = struct.unpack_from('<H', att_bytes, off)[0]; off += 2; parts.append(f'desc_len={dl}')
                if off + dl <= len(att_bytes):
                    desc = att_bytes[off:off+dl].decode('utf8', errors='replace')[:15]
                    parts.append(f'desc="{desc}"'); off += dl
            if off + 8 <= len(att_bytes):
                qty = struct.unpack_from('<Q', att_bytes, off)[0]; off += 8; parts.append(f'qty={qty}')
            if off < len(att_bytes):
                dec = att_bytes[off]; off += 1; parts.append(f'decimals={dec}')
            if off < len(att_bytes):
                parts.append(f'extra={att_bytes[off:].hex()[:30]}')
        
        elif tx_type == 6 and subtype == 0:  # Some type with prunable attachment
            if off < len(att_bytes):
                remaining = att_bytes[off:]
                parts.append(f'data={len(remaining)}B:{remaining.hex()[:40]}')
        
        else:
            if off < len(att_bytes):
                parts.append(f'raw={att_bytes[off:].hex()[:45]}')
        
        print(f'#{i:>3} {tx_type}:{subtype:<2} {version:>4} {phased:>6} {has_prunable_msg:>8} {has_prunable_att:>8} | {len(att_bytes):>6}B | {" | ".join(parts)}')
        
    except Exception as e:
        print(f'#{i:>3} {tx_type}:{subtype:<2} {version:>4} | ERR: {e}')

print("\n" + "=" * 200)
