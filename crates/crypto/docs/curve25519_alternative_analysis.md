# Curve25519 签名验证问题分析与 Ed25519 替代方案评估

## 问题现状

### 已完成的功能
- ✅ 公钥派生 - 使用 x25519-dalek 标准库，与 NRCS Java 实现完全一致
- ✅ Account ID 生成 - Reed-Solomon 编码正确实现
- ✅ Passphrase 验证 - 12 词助记词系统正常工作
- ✅ 签名生成 - sign 函数可以生成签名

### 待解决的问题
- ❌ 签名验证 - verify 函数计算出的 Y 值与预期不匹配

测试输出显示：
```
Original h: 75fb17835e31ec8959eb3c5c87bcac41a91799b71f2032b2553f87c20e049981
Computed h: c77f7de8d9017b7f3ef4dc5127d36c0f57a9d9677cf42bbfc474ed819dcdf7ba
```

## 问题根因分析

NRCS 使用的是自定义的 EC-KCDSA 变体签名算法，与标准 Ed25519 有以下区别：

1. **签名私钥派生不同**
   - NRCS: `keygen(P, s, k)` 生成公钥 P 和签名私钥 s
   - Ed25519: 直接使用种子派生密钥对

2. **签名计算不同**
   - NRCS: 使用自定义的 `sign(v, h, x, s)` 函数
   - Ed25519: 使用标准的 Schnorr 签名

3. **验证计算不同**
   - NRCS: 使用复杂的 `verify(Y, v, h, P)` 椭圆曲线点运算
   - Ed25519: 使用标准的签名验证

## 替代方案评估

### 方案 1: 继续调试 NRCS Curve25519 实现

**优点：**
- 与现有 NRCS 区块链完全兼容
- 已有大部分代码实现

**缺点：**
- 调试复杂，涉及大量数学运算
- 缺乏参考实现和测试向量
- 可能存在边界条件和特殊情况

**工作量估计：** 中等偏高

### 方案 2: 使用 Ed25519 替代

**优点：**
- 标准算法，有成熟的 Rust 实现 (ed25519-dalek)
- 广泛使用，安全审计充分
- 性能优异

**缺点：**
- 与 NRCS Java 实现不兼容
- 需要修改现有区块链协议

**工作量估计：** 低

### 方案 3: 混合方案

保留 Curve25519 公钥派生（用于账户 ID），使用 Ed25519 进行签名验证：

**优点：**
- 公钥派生与 NRCS 兼容
- 签名使用标准算法，安全可靠

**缺点：**
- 签名格式与 NRCS 不兼容
- 需要协议升级

**工作量估计：** 中等

## 建议

### 短期建议（推荐）

采用**方案 3 混合方案**：
1. 保留 x25519-dalek 用于公钥派生（与 NRCS 兼容）
2. 使用 ed25519-dalek 进行签名验证（标准安全）
3. 在配置中支持算法选择

### 长期建议

1. 继续调试 NRCS Curve25519 签名验证，作为可选功能
2. 添加更多测试向量，逐步验证每个数学函数
3. 考虑与 NRCS 团队沟通，获取更多测试数据

## 实现计划

### 阶段 1: 混合方案实现（当前）
- [x] 公钥派生使用 x25519-dalek
- [ ] 签名验证使用 ed25519-dalek
- [ ] 更新配置支持算法选择

### 阶段 2: NRCS 兼容性（后续）
- [ ] 继续调试 verify 函数
- [ ] 添加更多单元测试
- [ ] 与 Java 实现对比测试

## 代码示例

### 混合方案实现

```rust
use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Signature};
use x25519_dalek::StaticSecret;
use sha2::{Digest, Sha256};

pub struct HybridCrypto {
    // 用于公钥派生 (NRCS 兼容)
    x25519_secret: StaticSecret,
    // 用于签名 (标准 Ed25519)
    ed25519_signing_key: SigningKey,
}

impl HybridCrypto {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        // 公钥派生使用 x25519
        let x25519_secret = StaticSecret::from(seed);
        
        // 签名使用 ed25519
        let ed25519_signing_key = SigningKey::from_bytes(&seed);
        
        Self {
            x25519_secret,
            ed25519_signing_key,
        }
    }
    
    pub fn public_key(&self) -> [u8; 32] {
        // 返回 x25519 公钥 (NRCS 兼容)
        *x25519_dalek::PublicKey::from(&self.x25519_secret).as_bytes()
    }
    
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        // 使用 ed25519 签名
        self.ed25519_signing_key.sign(message).to_bytes()
    }
    
    pub fn verify(public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
        // 使用 ed25519 验证
        let verifying_key = VerifyingKey::from_bytes(public_key);
        if verifying_key.is_err() {
            return false;
        }
        let sig = Signature::from_bytes(signature);
        verifying_key.unwrap().verify(message, &sig).is_ok()
    }
}
```

## 结论

建议采用混合方案，在保证 NRCS 公钥派生兼容性的同时，使用标准的 Ed25519 进行签名验证。这样既能保持账户 ID 的一致性，又能利用成熟安全的签名算法。
