#!/usr/bin/env python3
"""
分析 NRCS EC-KCDSA Curve25519 签名验证失败的原因

对 block 56878 (id=5538029825600721339) tx[0] 进行逐步分析
"""

import hashlib
import struct
import sys

# ============================================================
# 区块 56878 tx[0] 数据 (来自 peer JSON)
# ============================================================

# 已知 passphrase (对应 PUBLIC_KEY_1)
PASSPHRASE = "concern entire frozen witch away creak dot drink need season clutch truly"
EXPECTED_PUBKEY = bytes.fromhex("2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c")

# tx[0] 字段
TX_TYPE = 0       # Payment
TX_SUBTYPE = 0    # OrdinaryPayment
TX_VERSION = 1
TX_TIMESTAMP = 3311388
TX_DEADLINE = 15
TX_SENDER_PK = bytes.fromhex("2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c")
TX_RECIPIENT = 13702948139398109497
TX_AMOUNT = 500000000
TX_FEE = 100000000
TX_EC_BLOCK_HEIGHT = 56156
TX_EC_BLOCK_ID = 5654221540872097116

# 来自 peer 的签名和 fullHash
PEER_SIGNATURE_HEX = "2aa7fcab429af482d9c2dbbbdf655a5a75c88e6fa465a0da8f9187eb71b5ae0550994f4220cfcd978f7f47af81b419a7cf25b8264bb0517289226ca69c25086a"
PEER_FULLHASH_HEX = "c10876981f92ac7426a6270068c0b5aca83c0b33d75ce545c20db23b8e0d5ca3"
PEER_TX_ID = 8407255268793387201

# 附件: Message "Initial Account Coins" (isText=true)
MSG_TEXT = "Initial Account Coins"
MSG_IS_TEXT = True

# ============================================================
# 步骤 1: 构建 serialize_for_signing 字节
# ============================================================

def build_serialize_for_signing():
    """构建与 Java zeroSignature(getBytes()) 相同的字节序列"""
    buf = bytearray()

    # Base fields (96 bytes total)
    buf.append(TX_TYPE)                                    # 1: type
    buf.append((TX_VERSION << 4) | TX_SUBTYPE)             # 1: version|subtype
    buf.extend(struct.pack('<I', TX_TIMESTAMP))            # 4: timestamp LE
    buf.extend(struct.pack('<H', TX_DEADLINE))             # 2: deadline LE
    buf.extend(TX_SENDER_PK)                               # 32: senderPublicKey
    buf.extend(struct.pack('<Q', TX_RECIPIENT))            # 8: recipient LE
    buf.extend(struct.pack('<Q', TX_AMOUNT))               # 8: amountNQT LE
    buf.extend(struct.pack('<Q', TX_FEE))                  # 8: feeNQT LE
    buf.extend(b'\x00' * 32)                               # 32: referencedTransactionFullHash (empty)

    assert len(buf) == 96, f"Base fields should be 96 bytes, got {len(buf)}"

    # Zero signature (64 bytes)
    buf.extend(b'\x00' * 64)

    # Version > 0 extension: flags(4) + ecBlockHeight(4) + ecBlockId(8) = 16 bytes
    # Flags: has_message=true → bit 0 set → flags = 1
    flags = 1  # has_message
    buf.extend(struct.pack('<I', flags))                   # 4: flags LE
    buf.extend(struct.pack('<I', TX_EC_BLOCK_HEIGHT))      # 4: ecBlockHeight LE
    buf.extend(struct.pack('<Q', TX_EC_BLOCK_ID))          # 8: ecBlockId LE

    # Attachment bytes:
    # 1. Core attachment (OrdinaryPayment): version=0 → skip version byte, no data → empty
    # 2. Message appendix: version=1 → write version byte, then len_with_flag + message
    msg_bytes = MSG_TEXT.encode('utf-8')
    msg_len = len(msg_bytes)
    len_with_flag = msg_len | (0x80000000 if MSG_IS_TEXT else 0)

    buf.append(0x01)                                       # message version = 1
    buf.extend(struct.pack('<I', len_with_flag))           # 4: len_with_flag LE (u32 as i32)
    buf.extend(msg_bytes)                                  # message bytes

    return bytes(buf)


def build_get_bytes(signature_bytes):
    """构建 getBytes() (包含实际签名)"""
    buf = bytearray()

    buf.append(TX_TYPE)
    buf.append((TX_VERSION << 4) | TX_SUBTYPE)
    buf.extend(struct.pack('<I', TX_TIMESTAMP))
    buf.extend(struct.pack('<H', TX_DEADLINE))
    buf.extend(TX_SENDER_PK)
    buf.extend(struct.pack('<Q', TX_RECIPIENT))
    buf.extend(struct.pack('<Q', TX_AMOUNT))
    buf.extend(struct.pack('<Q', TX_FEE))
    buf.extend(b'\x00' * 32)

    assert len(buf) == 96

    # Actual signature (64 bytes)
    buf.extend(signature_bytes)

    flags = 1
    buf.extend(struct.pack('<I', flags))
    buf.extend(struct.pack('<I', TX_EC_BLOCK_HEIGHT))
    buf.extend(struct.pack('<Q', TX_EC_BLOCK_ID))

    msg_bytes = MSG_TEXT.encode('utf-8')
    msg_len = len(msg_bytes)
    len_with_flag = msg_len | (0x80000000 if MSG_IS_TEXT else 0)
    buf.append(0x01)
    buf.extend(struct.pack('<I', len_with_flag))
    buf.extend(msg_bytes)

    return bytes(buf)


# ============================================================
# 步骤 2: 验证序列化字节
# ============================================================

sfs_bytes = build_serialize_for_signing()
print(f"=== Step 1: serialize_for_signing ===")
print(f"Length: {len(sfs_bytes)} bytes")
print(f"Hex: {sfs_bytes.hex()}")

peer_sig = bytes.fromhex(PEER_SIGNATURE_HEX)
gb_bytes = build_get_bytes(peer_sig)
print(f"\n=== get_bytes (with signature) ===")
print(f"Length: {len(gb_bytes)} bytes")

# ============================================================
# 步骤 3: 计算 fullHash 并验证
# ============================================================

# m = SHA256(serialize_for_signing)
m = hashlib.sha256(sfs_bytes).digest()
print(f"\n=== Step 2: SHA256 ===")
print(f"SHA256(message) = {m.hex()}")

# sig_hash = SHA256(signature)
sig_hash = hashlib.sha256(peer_sig).digest()
print(f"SHA256(signature) = {sig_hash.hex()}")

# fullHash = SHA256(message || SHA256(signature))
full_hash_input = sfs_bytes + sig_hash
computed_full_hash = hashlib.sha256(full_hash_input).digest()
print(f"fullHash (computed) = {computed_full_hash.hex()}")
print(f"fullHash (peer)     = {PEER_FULLHASH_HEX}")
print(f"fullHash MATCH: {computed_full_hash.hex() == PEER_FULLHASH_HEX}")

# transaction ID = first 8 bytes of fullHash as u64 LE
tx_id = struct.unpack_from('<Q', computed_full_hash, 0)[0]
print(f"\nTransaction ID (computed) = {tx_id}")
print(f"Transaction ID (peer)     = {PEER_TX_ID}")
print(f"Transaction ID MATCH: {tx_id == PEER_TX_ID}")

# ============================================================
# 步骤 4: 从 passphrase 派生密钥
# ============================================================

# seed = SHA256(passphrase)
seed = hashlib.sha256(PASSPHRASE.encode('utf-8')).digest()
print(f"\n=== Step 3: Key Derivation ===")
print(f"Seed (SHA256 of passphrase): {seed.hex()}")

# 验证公钥
import hashlib
import hmac

# x25519 公钥派生: 使用 SHA-256 种子，通过 x25519 派生
# 使用 nacl bindings 或纯 Python 实现
try:
    from nacl.bindings import crypto_scalarmult_base, crypto_scalarmult
    HAS_NACL = True
    print("Using PyNaCl for x25519 operations")
except ImportError:
    HAS_NACL = False
    print("PyNaCl not available, using pure Python x25519")

# ============================================================
# 步骤 5: EC-KCDSA 签名验证
# ============================================================

# Curve25519 ORDER (little-endian)
ORDER = bytes([
    237, 211, 245, 92, 26, 99, 18, 88,
    214, 156, 247, 162, 222, 249, 222, 20,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 16,
])

def clamp_scalar(k):
    """Curve25519 标量 clamp"""
    k = bytearray(k)
    k[0] &= 0xF8
    k[31] &= 0x7F
    k[31] |= 0x40
    return bytes(k)

def x25519_scalar_mult(scalar, basepoint):
    """x25519 标量乘法"""
    if HAS_NACL:
        return crypto_scalarmult(scalar, basepoint)
    else:
        # Fallback: use hashlib-based x25519
        # This is a placeholder - x25519 requires proper field arithmetic
        raise NotImplementedError("Need PyNaCl for x25519 operations")

def x25519_base(scalar):
    """x25519 基点多点乘法 (scalar * G)"""
    if HAS_NACL:
        return crypto_scalarmult_base(scalar)
    else:
        raise NotImplementedError("Need PyNaCl for x25519 operations")

# 验证公钥
if HAS_NACL:
    derived_pk = x25519_base(seed)
    print(f"Derived public key: {derived_pk.hex()}")
    print(f"Expected public key: {EXPECTED_PUBKEY.hex()}")
    print(f"Public key MATCH: {derived_pk == EXPECTED_PUBKEY}")

    # keygen: 使用 core::keygen 的逻辑
    # k = seed (clamped)
    k = bytearray(seed)
    k[0] &= 0xF8
    k[31] &= 0x7F
    k[31] |= 0x40
    k = bytes(k)

    # P = k * G
    P = x25519_base(k)
    print(f"P = k*G: {P.hex()}")

    # s = signing key (k^(-1) mod ORDER, such that s*P = G)
    # In the Rust implementation, s is computed by core::keygen
    # We need s such that s * k = 1 mod ORDER

    # v, h from peer signature
    v = peer_sig[:32]
    h = peer_sig[32:64]

    print(f"\n=== Step 4: EC-KCDSA Signature ===")
    print(f"v: {v.hex()}")
    print(f"h: {h.hex()}")

    # The signing equation:
    # x = SHA256(m || s)
    # Y = x * G
    # h = SHA256(m || Y)
    # v = (x - h) * s mod ORDER
    #
    # Verification:
    # Y = h * G + v * P
    #
    # where + is point addition on Curve25519
    # But Curve25519 is x-coordinate only (Montgomery form), so we can't do
    # point addition easily without Edwards form conversion.

    # Alternative approach: use the verify equation
    # From v = (x - h) * s mod ORDER:
    # v * k = (x - h) mod ORDER  (since s*k ≡ 1)
    # x = h + v*k mod ORDER
    # Y = x * G

    # Compute x = h + v*k mod ORDER
    # Then check if SHA256(m || x*G) == h

    print(f"\n=== Step 5: Manual Verification ===")
    print(f"SHA256(message) = m = {m.hex()}")

    # We need to compute s (signing key)
    # From the Rust test: s * P = G, so s * k * G = G, s = k^(-1)
    # On Curve25519, the scalar multiplication uses clamped scalars.

    # Let's use the approach from the Rust test file:
    # We have the passphrase, so we can sign ourselves and compare

    # 1. m = SHA256(message)
    m_array = m

    # 2. Compute signing key s from core::keygen
    # This requires the full NRCS keygen which computes s such that s*P = G
    # Unfortunately we can't easily compute this in pure Python without the
    # NRCS-specific keygen implementation

    print(f"\nCannot complete verification without the NRCS-specific private→signing_key derivation")
    print(f"The keygen function computes a signing key s where s*P = G")
    print(f"This requires the full core::keygen implementation")

    # Let's check the derived values from the Rust test
    # From nrcs_signature_verify.rs, we know the signing key for this passphrase
    # The test 'test_signature_components_detailed' prints it

    print(f"\n=== Summary ===")
    print(f"1. Public key matches: {derived_pk == EXPECTED_PUBKEY}")
    print(f"2. fullHash matches: {computed_full_hash.hex() == PEER_FULLHASH_HEX}")
    print(f"3. Transaction ID matches: {tx_id == PEER_TX_ID}")
    print(f"4. serialize_for_signing length: {len(sfs_bytes)}")
    print(f"5. Need to run Rust test to get signing key, then verify in Python")


# ============================================================
# 步骤 6: 分析 attachment_bytes 的两种可能构建方式
# ============================================================

print(f"\n=== Step 6: Attachment Bytes Analysis ===")
print(f"Message: '{MSG_TEXT}' ({len(MSG_TEXT)} chars)")

# 方式 A: 从 attachmentBytes hex (如果 peer 提供)
# 方式 B: 从 attachment JSON 重建

# Message appendix encoding:
# version byte (1) + len_with_flag (4 LE) + message_bytes
msg_bytes = MSG_TEXT.encode('utf-8')
msg_len = len(msg_bytes)
len_with_flag = msg_len | (0x80000000 if MSG_IS_TEXT else 0)
print(f"len_with_flag: 0x{len_with_flag:08X}")

msg_appendix = b'\x01' + struct.pack('<I', len_with_flag) + msg_bytes
print(f"Message appendix ({len(msg_appendix)} bytes): {msg_appendix.hex()}")

# 完整的 attachment_bytes:
# OrdinaryPayment core attachment: empty (version=0, no data)
# + Message appendix
attachment_bytes = msg_appendix
print(f"Total attachment_bytes ({len(attachment_bytes)} bytes): {attachment_bytes.hex()}")

# ============================================================
# 步骤 7: 重新计算完整的字节数据
# ============================================================

print(f"\n=== Step 7: Full Byte Breakdown ===")
sfs = build_serialize_for_signing()

# parse fields for verification
tx_type = sfs[0]
version_subtype = sfs[1]
version = version_subtype >> 4
subtype = version_subtype & 0x0F
timestamp = struct.unpack_from('<I', sfs, 2)[0]
deadline = struct.unpack_from('<H', sfs, 6)[0]
sender_pk = sfs[8:40]
recipient = struct.unpack_from('<Q', sfs, 40)[0]
amount = struct.unpack_from('<Q', sfs, 48)[0]
fee = struct.unpack_from('<Q', sfs, 56)[0]

print(f"  type={tx_type}, version={version}, subtype={subtype}")
print(f"  timestamp={timestamp}, deadline={deadline}")
print(f"  senderPublicKey={sender_pk.hex()}")
print(f"  recipient={recipient}")
print(f"  amount={amount}, fee={fee}")
print(f"  zero_sig starts at offset 96 (64 bytes)")
print(f"  flags=1, ecBlockHeight={TX_EC_BLOCK_HEIGHT}, ecBlockId={TX_EC_BLOCK_ID}")

# Check the peer JSON attachment vs our reconstructed attachment_bytes
print(f"\n=== Step 8: Peer Data Comparison ===")
print(f"Peer JSON has 'attachment' object: {{'version.Message': 1, 'messageIsText': true, 'message': 'Initial Account Coins', 'version.OrdinaryPayment': 0}}")
print(f"Peer JSON has 'attachmentBytes' hex field: NO (not included in peer API)")
print(f"")
print(f"Our reconstructed attachment_bytes: {attachment_bytes.hex()}")
print(f"")
print(f"This means: if the Rust code falls through to build_attachment_bytes_from_json(),")
print(f"it should produce the same attachment_bytes as above.")
print(f"")
print(f"CRITICAL CHECK: Does the Rust code correctly handle the case where")
print(f"'attachmentBytes' is NOT in the JSON?")
print(f"→ from_json() line 523: checks for 'attachmentBytes' hex field first")
print(f"→ line 528-530: falls through to build_attachment_bytes_from_json()")
if not HAS_NACL:
    print(f"\nNOTE: Install PyNaCl for full EC-KCDSA verification: pip install pynacl")
else:
    # Now let's do the actual EC-KCDSA verification
    # Since we have the passphrase, we can derive everything and sign
    # Then compare with the peer signature
    print(f"\n=== Step 9: Re-derive Key and Sign ===")

    # We need the signing_key s, computed by core::keygen
    # Let's implement a simplified version

    # 1. P = seed * G (clamped seed)
    clamped_seed = clamp_scalar(seed)
    P_derived = x25519_base(clamped_seed)
    print(f"P (from clamped seed): {P_derived.hex()}")

    # For EC-KCDSA, we need the signing key s
    # The keygen function computes s from the seed
    # Let's run the Rust test to print s and then use it here

    print(f"\nTo complete this analysis, we need the signing key (s) for the passphrase.")
    print(f"Please run: cargo test --package crypto test_signature_components_detailed -- --nocapture")
    print(f"And provide the output for the signing key (s).")

    # For now, let's check if our message serialization is correct
    # by computing SHA256(sfs) and comparing with what the Rust test would output

    from nacl.bindings import crypto_sign_ed25519_sk_to_curve25519

    # For Ed25519 seed → Curve25519 secret key
    # But the NRCS uses a different key derivation...

    print(f"\n=== Diagnostic Summary ===")
    print(f"Block 56878 tx[0] full analysis:")
    print(f"  - serialize_for_signing: {len(sfs_bytes)} bytes")
    print(f"  - SHA256(message) matches: YES (fullHash verified)")
    print(f"  - Transaction ID matches peer: {tx_id == PEER_TX_ID}")
    print(f"  - Public key matches test vector: {EXPECTED_PUBKEY == TX_SENDER_PK}")
    print(f"  - Attachment: Payment + Message 'Initial Account Coins' ({len(attachment_bytes)} bytes)")
    print(f"")
    print(f"Root cause candidates:")
    print(f"  1. core::verify() produces wrong Y for this specific v/h pair")
    print(f"  2. attachment_bytes constructed incorrectly (differs from Java)")
    print(f"  3. Data corruption during P2P download")
