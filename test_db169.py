#!/usr/bin/env python3
"""
DB_ID=169 诊断脚本: type=4(subtype=1) SetPhasingOnly + Phasing

Java 数据:
  fullHash:     435a10defeb7797bd317f8ace4975ece824b086338dfc470a6f3a784de2cd608
  attachment:   01ff0000000000000000000000000000000000000000000000000000000000000000000000000000012c4800000001000000 (49 bytes)
"""

import hashlib, struct, json, sys

def u64_to_i64(u):
    """u64 -> i64 保持相同字节表示"""
    if u >= (1 << 63):
        return u - (1 << 64)
    return u

# === DB_ID=169 交易数据 (来自 Java API) ===
tx = {
    "type": 4,
    "subtype": 1,
    "version": 1,
    "timestamp": 472620,
    "deadline": 1440,
    "senderPublicKey": "cd70063e3017c782361323939c59c546217d3ec3249f1a5b9af1290ed4201073",
    # sender = "17712488668481078630" -> i64
    "recipient_id": None,  # null
    "amount": 0,
    "fee": 2100000000,
    "signature": bytes.fromhex("71a4133806ecf9a09eb634874e94dc3fc163ee36670cb738d267b0d0ead9fe0e0e5cfa46230c4b95b622e1948755e7ca481df162e66edb3b1b9f1a53eedbf3f1"),
    "ecBlockHeight": 0,
    "ecBlockId": 3488276486778630462,
    "attachment_json": {
        "phasingFinishHeight": 18476,
        "phasingHolding": "0",
        "phasingQuorum": "1",
        "version.Phasing": 1,
        "phasingControlParams": {
            "phasingHolding": "0",
            "phasingQuorum": 0,
            "phasingMinBalance": 0,
            "phasingMinBalanceModel": 0,
            "phasingVotingModel": -1
        },
        "version.SetPhasingOnly": 1,
        "controlMaxFees": "0",
        "controlMinDuration": 0,
        "controlMaxDuration": 0,
        "phasingMinBalance": "10000000000000",
        "phasingMinBalanceModel": 1,
        "phasingVotingModel": 0
    }
}

EXPECTED_FULLHASH = "435a10defeb7797bd317f8ace4975ece824b086338dfc470a6f3a784de2cd608"
EXPECTED_ATT_BYTES = "01ff0000000000000000000000000000000000000000000000000000000000000000000000000000012c4800000001000000"

print("=" * 80)
print("DB_ID=169 诊断: type=4:1 SetPhasingOnly + Phasing")
print("=" * 80)

# ============================================================
# Step 1: 构建 SetPhasingOnly attachment bytes
# Java: SetPhasingOnly.putMyBytes():
#   phasingParams.putMyBytes(buffer)
#   buffer.putLong(maxFees)           // 8 bytes
#   buffer.putShort(minDuration)      // 2 bytes  
#   buffer.putShort(maxDuration)      // 2 bytes
# Total: phasingParams_size + 12
#
# PhasingParams.putMyBytes():
#   buffer.put(voteWeighting.getVotingModel().getCode())  // 1 byte
#   buffer.putLong(quorum)                                  // 8 bytes
#   buffer.putLong(voteWeighting.getMinBalance())           // 8 bytes
#   buffer.put((byte) whitelist.length)                     // 1 byte
#   for account in whitelist: buffer.putLong(account)       // 8*N bytes
#   buffer.putLong(voteWeighting.getHoldingId())            // 8 bytes
#   buffer.put(voteWeighting.getMinBalanceModel().getCode())// 1 byte
# ============================================================

att = tx["attachment_json"]

# --- PhasingParams ---
voting_model = att["phasingVotingModel"]  # 0
quorum = int(att["phasingQuorum"])         # 1
min_balance = int(att["phasingMinBalance"])  # 10000000000000
holding_id = int(att["phasingHolding"])    # 0
min_balance_model = att["phasingMinBalanceModel"]  # 1
whitelist = []  # 无 whitelist 字段

buf = bytearray()
# votingModel code (1 byte)
buf.append(voting_model & 0xFF)
# quorum (8 bytes, long LE)
buf += struct.pack('<q', quorum)
# minBalance (8 bytes, long LE)
buf += struct.pack('<q', min_balance)
# whitelist length (1 byte)
buf.append(len(whitelist))
# holdingId (8 bytes, long LE)
buf += struct.pack('<q', holding_id)
# minBalanceModel (1 byte)
buf.append(min_balance_model & 0xFF)

phasing_params_bytes = bytes(buf)
print(f"\n[PhasingParams] {len(phasing_params_bytes)} bytes")
print(f"  hex: {phasing_params_bytes.hex()}")
print(f"  votingModel={voting_model}, quorum={quorum}, minBalance={min_balance}")
print(f"  holdingId={holding_id}, minBalanceModel={min_balance_model}, whitelistLen={len(whitelist)}")

# --- SetPhasingOnly ---
max_fees = int(att["controlMaxFees"])      # 0
min_duration = att["controlMinDuration"]    # 0 (short)
max_duration = att["controlMaxDuration"]    # 0 (short)

buf2 = bytearray()
buf2 += phasing_params_bytes
buf2 += struct.pack('<q', max_fees)          # 8 bytes
buf2 += struct.pack('<h', min_duration)      # 2 bytes (short!)
buf2 += struct.pack('<h', max_duration)      # 2 bytes (short!)

att_bytes_no_version = bytes(buf2)

# 加上 version byte (version.SetPhasingOnly = 1 > 0, 所以有 version prefix)
full_att = bytearray()
full_att.append(1)  # version = 1
full_att += att_bytes_no_version
full_att = bytes(full_att)

print(f"\n[SetPhasingOnly] total: {len(full_att)} bytes")
print(f"  hex: {full_att.hex()}")
print(f"  expected: {EXPECTED_ATT_BYTES}")
print(f"  MATCH: {full_att.hex() == EXPECTED_ATT_BYTES}")

if full_att.hex() != EXPECTED_ATT_BYTES:
    print(f"\n  !!! ATTACHMENT BYTES MISMATCH !!!")
    print(f"  expected len: {len(bytes.fromhex(EXPECTED_ATT_BYTES))}, got: {len(full_att)}")
    for i in range(min(len(full_att), len(bytes.fromhex(EXPECTED_ATT_BYTES)))):
        exp = bytes.fromhex(EXPECTED_ATT_BYTES)[i]
        got = full_att[i]
        if exp != got:
            print(f"  diff @ offset {i}: expected 0x{exp:02x} ({exp}), got 0x{got:02x} ({got})")

# ============================================================
# Step 2: 计算 flags
# Java getFlags(): bit4(phased)=16 | bit5(prunablePlainMessage)=32 | bit6(prunableEncrypted)=64
# DB_ID=169: phased=true, 无 prunablePlainMessage, 无 prunableEncryptedMessage
# => flags = 16
# ============================================================
flags = 16  # only phased (bit 4)
print(f"\n[Flags] = {flags} (0x{flags:x})")

# ============================================================
# Step 3: 完整序列化 + 计算 fullHash
# ============================================================
sender_pk = bytes.fromhex(tx["senderPublicKey"])
sig = tx["signature"]
recipient = 0  # null recipient

# serialize_for_signing (unsigned bytes per Java Transaction.bytes())
signing = bytearray()
signing.append(tx["type"])                              # type: 1B
signing.append((tx["version"] << 4) | tx["subtype"])    # version<<4|subtype: 1B
signing += struct.pack('<i', tx["timestamp"])            # timestamp: 4B (int)
signing += struct.pack('<h', tx["deadline"])             # deadline: 2B (short)
signing += sender_pk                                     # senderPublicKey: 32B
signing += struct.pack('<q', recipient)                  # recipient: 8B (long, 0 for null)
signing += struct.pack('<q', tx["amount"])               # amount: 8B (long)
signing += struct.pack('<q', tx["fee"])                 # fee: 8B (long)
signing += b'\x00' * 32                                 # referencedTransactionFullHash: 32B zeros

signing = bytes(signing)
print(f"\n[Signing] {len(signing)} bytes")

# serialize_for_full_hash = signing + zeroSignature(64B) + flags + ecBlockHeight + ecBlockId + attachment
full_hash_buf = bytearray(signing)
full_hash_buf += b'\x00' * 64                            # zero signature: 64B
if tx["version"] > 0:
    full_hash_buf += struct.pack('<I', flags)             # flags: 4B (int)
    full_hash_buf += struct.pack('<I', tx["ecBlockHeight"]) # ecBlockHeight: 4B (int)
    full_hash_buf += struct.pack('<q', tx["ecBlockId"])   # ecBlockId: 8B (long)
full_hash_buf += full_att                                # attachmentBytes

full_hash_input = bytes(full_hash_buf)
print(f"[FullHash Input] {len(full_hash_input)} bytes")

# fullHash = SHA256(unsigned_bytes || SHA256(signature))
sig_hash = hashlib.sha256(sig).digest()
final_input = full_hash_input + sig_hash
calculated_fullhash = hashlib.sha256(final_input).digest()

print(f"[Result]")
print(f"  calculated: {calculated_fullhash.hex()}")
print(f"  expected:   {EXPECTED_FULLHASH}")
print(f"  MATCH: {calculated_fullhash.hex() == EXPECTED_FULLHASH}")

if calculated_fullhash.hex() != EXPECTED_FULLHASH:
    print(f"\n  !!! FULLHASH MISMATCH !!!")
    print(f"\n  Debug - full hash input hex:")
    print(f"  {final_input.hex()}")

sys.exit(0 if calculated_fullhash.hex() == EXPECTED_FULLHASH else 1)
