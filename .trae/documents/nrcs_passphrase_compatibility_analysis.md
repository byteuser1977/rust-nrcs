# NRCS 助记词验证问题修复计划（优化版）

## 问题分析

经过对 Java 原始实现（位于 `/mnt/d/workspace/git/nrcs`）和 Rust 实现的深入对比分析，以及使用提供的测试数据进行验证，发现 NRCS 项目中存在以下关键问题：

### 测试数据
- **PassPhrase**: `confusion flirt teeth story crawl dear shove screw decay flood cover warrior`
- **Account Id**: `NRCS-P5FE-QYES-PLHK-HWTJM`
- **PublicKey**: `5aafe59365b988d73aa15424a1b83c24fcada07e7cd73fcfade612c5bf32fc71`

### 1. 前端问题

**文件**: [frontend/src/views/account/Register.vue](file:///mnt/d/workspace/git/rust-nrcs/frontend/src/views/account/Register.vue)

- **问题**: 第 125 行引用了 `cryptoUtils.generateKeypair()`，但 [frontend/src/utils/crypto.ts](file:///mnt/d/workspace/git/rust-nrcs/frontend/src/utils/crypto.ts) 中并没有定义这个函数
- **影响**: 前端无法正常生成 NRCS 助记词，会导致运行时错误

### 2. 后端助记词生成问题

**文件**: [crates/crypto/src/crypto.rs](file:///mnt/d/workspace/git/rust-nrcs/crates/crypto/src/crypto.rs)

- **问题**: 第 256-281 行的 `generate_mnemonic` 函数使用了错误的单词列表
  - 当前使用约 70 个单词的简化列表
  - NRCS 标准应使用 1626 个单词（定义在 [words.rs](file:///mnt/d/workspace/git/rust-nrcs/crates/crypto/src/passphrase/words.rs)）
- **Java 原始实现**: [SecretSharingGenerator.java](file:///mnt/d/workspace/git/nrcs/nrcs-crypto/src/main/java/com/bytechain/nrcs/crypto/SecretSharingGenerator.java#L176-L185)
  - 生成 16 字节随机数
  - 转换为 BigInteger
  - 调用 `from128bit()` 转换为 12 个单词
- **影响**: 生成的助记词不符合 NRCS 标准，无法通过 `validate_passphrase` 验证

### 3. 验证逻辑与 Java 一致 ✅

**文件**: [crates/crypto/src/passphrase/mod.rs](file:///mnt/d/workspace/git/rust-nrcs/crates/crypto/src/passphrase/mod.rs)

- **Java 原始实现**: [SecretSharingGenerator.java](file:///mnt/d/workspace/git/nrcs/nrcs-crypto/src/main/java/com/bytechain/nrcs/crypto/SecretSharingGenerator.java#L218-L220)
  - `is12WordsSecret()` 只检查单词数量和是否在列表中
  - **没有校验和验证**
- **Rust 实现**: 与 Java 完全一致
- **结论**: 无需修改

### 4. 密钥派生问题 ❌ 严重

**文件**: [crates/crypto/src/crypto.rs](file:///mnt/d/workspace/git/rust-nrcs/crates/crypto/src/crypto.rs#L250-L253)

- **问题**: `generate_keypair_from_passphrase` 函数直接对任意字符串进行 SHA-256 哈希
- **Java 原始实现**: 没有直接的密钥派生函数，而是通过助记词转换为数字后再处理
- **缺失**: 没有验证输入是否为有效的 NRCS 助记词
- **影响**: 用户可能使用非 NRCS 标准的助记词，导致兼容性问题

### 5. Account ID 生成问题 ❌ 严重

**文件**: [crates/crypto/src/crypto.rs](file:///mnt/d/workspace/git/rust-nrcs/crates/crypto/src/crypto.rs#L284-L300)

- **问题 1**: 密钥派生错误
  - 第 285 行使用 `sha256(mnemonic.as_bytes())` 来派生密钥
  - 应该使用 `passphrase_to_keypair` 函数

- **问题 2**: Account ID 计算错误
  - **Java 实现**: [Convert.java](file:///mnt/d/workspace/git/nrcs/nrcs-common/src/main/java/com/bytechain/nrcs/common/utils/Convert.java#L190-L196)
    ```java
    public static long fullHashToId(byte[] hash) {
        BigInteger bigInteger = new BigInteger(1, new byte[]{
            hash[7], hash[6], hash[5], hash[4], 
            hash[3], hash[2], hash[1], hash[0]
        });
        return bigInteger.longValue();
    }
    ```
    - 使用公钥 SHA-256 哈希的**前 8 个字节**（hash[0-7]）
    - **小端序**（Little Endian）

  - **Rust 实现**:
    ```rust
    let account_id = u64::from_be_bytes([
        hash[24], hash[25], hash[26], hash[27],
        hash[28], hash[29], hash[30], hash[31],
    ]);
    ```
    - 使用公钥 SHA-256 哈希的**后 8 个字节**（hash[24-31]）
    - **大端序**（Big Endian）

  - **结果**: 完全不兼容！生成的 Account ID 完全不同

### 6. 单词列表对比结果 ✅

经过对比，Rust 实现的单词列表与 Java 原始实现完全一致：
- Java: `Constant.ALL_SECRET_PHRASE_WORDS` (1626 个单词)
- Rust: `NRCS_WORDS` (1626 个单词)
- ✅ 单词列表一致，无需修改

## 修复方案

### 阶段 1: 修复后端核心函数

#### 1.1 修复 `generate_mnemonic` 函数

**文件**: [crates/crypto/src/crypto.rs](file:///mnt/d/workspace/git/rust-nrcs/crates/crypto/src/crypto.rs#L256-L281)

**修改内容**:
- 删除当前的简化实现
- 调用 `passphrase::generate_passphrase()` 函数
- 确保生成的助记词符合 NRCS 标准（与 Java 实现一致）

**修改后代码**:
```rust
/// 生成 NRCS 标准助记词
/// 
/// 对应 Java: SecretSharingGenerator.generatePassPhrase()
pub fn generate_mnemonic(word_count: usize) -> CryptoResult<String> {
    if word_count != 12 {
        return Err(CryptoError::ConfigurationError(
            "NRCS only supports 12-word passphrases".to_string()
        ));
    }
    
    crate::passphrase::generate_passphrase()
        .map_err(|e| CryptoError::ConfigurationError(e.to_string()))
}
```

#### 1.2 修复 `derive_account_id` 函数

**文件**: [crates/crypto/src/crypto.rs](file:///mnt/d/workspace/git/rust-nrcs/crates/crypto/src/crypto.rs#L284-L300)

**修改内容**:
- 使用 `passphrase_to_keypair` 派生密钥对
- 使用前 8 个字节，小端序（与 Java 一致）

**修改后代码**:
```rust
/// 从助记词派生账户ID
/// 
/// 对应 Java: Account.getId(publicKey) -> Convert.fullHashToId(hash)
pub fn derive_account_id(passphrase: &str) -> CryptoResult<String> {
    // 使用 passphrase_to_keypair 派生密钥对
    let keypair = passphrase_to_keypair(passphrase)?;
    let pub_key = keypair.public_key();
    
    let pub_key_bytes = match pub_key {
        crate::PublicKey::Ed25519(bytes) => bytes,
    };
    
    // 对公钥进行 SHA-256 哈希
    let hash = sha256(&pub_key_bytes);
    
    // Java 实现：使用前 8 个字节，小端序
    let account_id = u64::from_le_bytes([
        hash[0], hash[1], hash[2], hash[3],
        hash[4], hash[5], hash[6], hash[7],
    ]);
    
    Ok(format!("NRCS-{}", encode_reed_solomon(&account_id)))
}
```

#### 1.3 修复 `derive_public_key` 函数

**文件**: [crates/crypto/src/crypto.rs](file:///mnt/d/workspace/git/rust-nrcs/crates/crypto/src/crypto.rs#L302-L311)

**修改内容**:
- 使用 `passphrase_to_keypair` 派生密钥对

**修改后代码**:
```rust
/// 从助记词派生公钥
/// 
/// 对应 Java: Crypto.getPublicKey(secretPhrase)
pub fn derive_public_key(passphrase: &str) -> CryptoResult<Vec<u8>> {
    let keypair = passphrase_to_keypair(passphrase)?;
    let pub_key = keypair.public_key();
    
    Ok(match pub_key {
        crate::PublicKey::Ed25519(bytes) => bytes.to_vec(),
    })
}
```

#### 1.4 增强 `generate_keypair_from_passphrase` 函数

**文件**: [crates/crypto/src/crypto.rs](file:///mnt/d/workspace/git/rust-nrcs/crates/crypto/src/crypto.rs#L250-L253)

**修改内容**:
- 添加助记词验证
- 支持两种模式：严格验证（NRCS 助记词）和宽松模式（任意密码）
- 保持与 Java 实现的兼容性

**修改后代码**:
```rust
/// 从密码短语生成密钥对（支持 NRCS 助记词和任意密码）
/// 
/// 对应 Java: 从 secretPhrase 生成密钥对
pub fn generate_keypair_from_passphrase(passphrase: &str) -> CryptoResult<KeyPair> {
    // 尝试验证是否为 NRCS 助记词
    if crate::validate_passphrase(passphrase).is_ok() {
        // 使用 NRCS 标准方法：先转换为数字，再派生密钥
        let number = crate::secret_to_number(passphrase)
            .map_err(|e| CryptoError::ConfigurationError(e.to_string()))?;
        let bytes = number.to_bytes_be().1;
        
        // 如果字节数不足 32，填充前导零
        let mut seed = [0u8; 32];
        let start = 32 - bytes.len().min(32);
        seed[start..].copy_from_slice(&bytes[..bytes.len().min(32)]);
        
        Ok(keypair_from_seed(&seed))
    } else {
        // 非 NRCS 助记词，使用 SHA-256 派生（向后兼容）
        let seed = sha256(passphrase.as_bytes());
        Ok(keypair_from_seed(&seed))
    }
}
```

### 阶段 2: 修复前端助记词生成

#### 2.1 添加前端助记词生成函数

**文件**: [frontend/src/utils/crypto.ts](file:///mnt/d/workspace/git/rust-nrcs/frontend/src/utils/crypto.ts)

**修改内容**:
- 添加 NRCS 单词列表
- 实现 `generateKeypair` 函数
- 实现助记词验证函数

**新增代码**:
```typescript
/**
 * NRCS 助记词单词列表（1626 个单词）
 * 来源: crates/crypto/src/passphrase/words.rs
 */
const NRCS_WORDS = [
  "like", "just", "love", "know", "never", "want", "time",
  // ... 完整的 1626 个单词列表（从 Rust 实现复制）
];

/**
 * 生成 NRCS 助记词密钥对
 */
export function generateKeypair(): { 
  mnemonic: string; 
  publicKey: string; 
  privateKey: string;
} {
  // 生成 12 个随机单词
  const words: string[] = [];
  for (let i = 0; i < 12; i++) {
    const idx = Math.floor(Math.random() * NRCS_WORDS.length);
    words.push(NRCS_WORDS[idx]);
  }
  
  const mnemonic = words.join(' ');
  
  // 注意：前端只生成助记词，实际的密钥派生应该在后端进行
  // 这里返回空值，实际使用时应该调用后端 API
  return {
    mnemonic,
    publicKey: '',
    privateKey: '',
  };
}

/**
 * 验证 NRCS 助记词
 */
export function validatePassphrase(passphrase: string): boolean {
  const words = passphrase.trim().split(/\s+/);
  
  if (words.length !== 12) {
    return false;
  }
  
  return words.every(word => NRCS_WORDS.includes(word));
}
```

#### 2.2 更新 Register.vue

**文件**: [frontend/src/views/account/Register.vue](file:///mnt/d/workspace/git/rust-nrcs/frontend/src/views/account/Register.vue)

**修改内容**:
- 确保正确导入和使用 `cryptoUtils`
- 添加助记词验证提示

**修改位置**: 第 125 行附近

**修改后代码**:
```typescript
import { generateKeypair, validatePassphrase } from '@/utils/crypto'

// 在 generateKeypair 函数中
const generateKeypair = async () => {
  try {
    generating.value = true

    if (!form.name || !form.email || !form.wallet_address) {
      ElMessage.warning('请先填写用户名、邮箱和钱包地址')
      return
    }

    // 使用加密工具生成密钥对
    const keypair = generateKeypair()
    generatedKeypair.value = keypair
    generatedMnemonic.value = keypair.mnemonic || ''

    ElMessage.success('密钥对生成成功，请保存助记词')
  } catch (error: any) {
    console.error('Failed to generate keypair:', error)
    ElMessage.error('密钥对生成失败：' + error.message)
  } finally {
    generating.value = false
  }
}
```

### 阶段 3: 添加测试和文档

#### 3.1 添加单元测试

**文件**: [crates/crypto/src/passphrase/mod.rs](file:///mnt/d/workspace/git/rust-nrcs/crates/crypto/src/passphrase/mod.rs#L209-L270)

**新增测试**:
```rust
#[test]
fn test_generate_mnemonic_compatibility() {
    // 测试生成的助记词符合 NRCS 标准
    let mnemonic = generate_mnemonic(12).unwrap();
    let words: Vec<&str> = mnemonic.split(' ').collect();
    
    assert_eq!(words.len(), 12);
    assert!(validate_passphrase(&mnemonic).is_ok());
}

#[test]
fn test_known_passphrase_validation() {
    // 测试已知助记词
    let passphrase = "confusion flirt teeth story crawl dear shove screw decay flood cover warrior";
    assert!(validate_passphrase(passphrase).is_ok());
}

#[test]
fn test_account_id_generation() {
    // 测试 Account ID 生成
    let passphrase = "confusion flirt teeth story crawl dear shove screw decay flood cover warrior";
    let expected_account_id = "NRCS-P5FE-QYES-PLHK-HWTJM";
    
    let account_id = derive_account_id(passphrase).unwrap();
    assert_eq!(account_id, expected_account_id);
}

#[test]
fn test_public_key_derivation() {
    // 测试公钥派生
    let passphrase = "confusion flirt teeth story crawl dear shove screw decay flood cover warrior";
    let expected_public_key = "5aafe59365b988d73aa15424a1b83c24fcada07e7cd73fcfade612c5bf32fc71";
    
    let public_key = derive_public_key(passphrase).unwrap();
    let public_key_hex = hex::encode(&public_key);
    
    assert_eq!(public_key_hex, expected_public_key);
}

#[test]
fn test_java_compatibility() {
    // 测试与 Java 实现的完整兼容性
    let passphrase = "confusion flirt teeth story crawl dear shove screw decay flood cover warrior";
    
    // 验证助记词
    assert!(validate_passphrase(passphrase).is_ok());
    
    // 转换为数字
    let number = secret_to_number(passphrase).unwrap();
    
    // 转换回助记词
    let recovered = number_to_secret(&number, true).unwrap();
    assert_eq!(passphrase, recovered);
    
    // 派生公钥
    let public_key = derive_public_key(passphrase).unwrap();
    assert_eq!(
        hex::encode(&public_key),
        "5aafe59365b988d73aa15424a1b83c24fcada07e7cd73fcfade612c5bf32fc71"
    );
    
    // 派生 Account ID
    let account_id = derive_account_id(passphrase).unwrap();
    assert_eq!(account_id, "NRCS-P5FE-QYES-PLHK-HWTJM");
}
```

#### 3.2 更新文档

**文件**: [crates/crypto/README.md](file:///mnt/d/workspace/git/rust-nrcs/crates/crypto/README.md)

**新增内容**:
```markdown
## NRCS 助记词系统

NRCS 使用自定义的 12 词助记词系统（非 BIP39 标准）：

### 特性
- 1626 个单词列表（与 Java 实现完全一致）
- 12 个单词组合
- 基于 128 位随机数生成

### 使用示例

\`\`\`rust
use crypto::{generate_passphrase, validate_passphrase, derive_account_id, derive_public_key};

// 生成助记词
let passphrase = generate_passphrase()?;
println!("助记词: {}", passphrase);

// 验证助记词
validate_passphrase(&passphrase)?;

// 派生公钥
let public_key = derive_public_key(&passphrase)?;
println!("公钥: {}", hex::encode(&public_key));

// 派生 Account ID
let account_id = derive_account_id(&passphrase)?;
println!("Account ID: {}", account_id);
\`\`\`

### 与 Java 实现的兼容性

Rust 实现与 Java 原始实现完全兼容：
- 单词列表一致（1626 个单词）
- 生成算法一致（16 字节随机数 → BigInteger → 12 个单词）
- 验证逻辑一致（只检查单词数量和是否在列表中）
- Account ID 生成一致（公钥 SHA-256 哈希前 8 字节，小端序）

### 注意事项
- NRCS 助记词与 BIP39 不兼容
- 助记词不包含校验和（与 Java 实现一致）
- 请安全保存助记词，丢失无法恢复
- 支持非 NRCS 标准的密码短语（向后兼容）
```

## 实施步骤

### 第一步：修复后端核心函数
1. 修改 `crypto.rs` 中的 `generate_mnemonic` 函数
2. 修复 `crypto.rs` 中的 `derive_account_id` 函数（关键）
3. 修复 `crypto.rs` 中的 `derive_public_key` 函数
4. 增强 `crypto.rs` 中的 `generate_keypair_from_passphrase` 函数

### 第二步：运行测试验证
1. 运行现有单元测试：`cargo test --package crypto --lib passphrase`
2. 使用提供的测试数据进行验证
3. 确保所有测试通过

### 第三步：修复前端代码
1. 在 `frontend/src/utils/crypto.ts` 中添加完整的 NRCS 单词列表
2. 实现 `generateKeypair` 函数
3. 更新 `Register.vue` 确保正确使用

### 第四步：集成测试
1. 测试前端生成助记词
2. 测试后端验证助记词
3. 测试密钥派生流程
4. 验证 Account ID 和 Public Key 是否匹配

### 第五步：更新文档
1. 更新 `crates/crypto/README.md`
2. 添加使用示例
3. 添加注意事项

## 风险评估

### 高风险
- **Account ID 生成逻辑**: 当前实现与 Java 完全不兼容，会导致用户无法访问账户
- **Public Key 派生**: 当前实现使用错误的密钥派生方法

### 中风险
- 前端助记词生成：需要确保单词列表完整且正确

### 低风险
- 后端函数修改：有完整的测试覆盖
- 文档更新：不影响功能

### 缓解措施
- 使用提供的测试数据进行完整验证
- 保留宽松模式：`generate_keypair_from_passphrase` 支持非 NRCS 助记词
- 完整测试：添加边界情况测试
- 向后兼容：确保现有助记词仍然有效

## 预期结果

修复完成后：
1. ✅ 前端可以正确生成 NRCS 标准助记词
2. ✅ 后端可以验证助记词的有效性（与 Java 实现一致）
3. ✅ 密钥派生流程完整且安全
4. ✅ Account ID 生成与 Java 完全一致
5. ✅ Public Key 派生与 Java 完全一致
6. ✅ 所有测试通过
7. ✅ 文档完善，使用清晰
8. ✅ 与 Java 原始实现完全兼容

## 关键发现

### 1. 单词列表完全一致 ✅
- Rust 实现的 `NRCS_WORDS` 与 Java 的 `ALL_SECRET_PHRASE_WORDS` 完全一致
- 都是 1626 个单词
- 无需修改

### 2. 验证逻辑与 Java 一致 ✅
- Java 原始实现的 `is12WordsSecret()` 只检查单词数量和是否在列表中
- **没有校验和验证**
- Rust 实现应该保持当前逻辑，以保持兼容性

### 3. 生成逻辑需要修复 ⚠️
- `generate_mnemonic` 函数使用了错误的单词列表
- 需要调用 `passphrase::generate_passphrase()` 来生成标准 NRCS 助记词

### 4. 密钥派生需要增强 ⚠️
- `generate_keypair_from_passphrase` 需要支持两种模式
- NRCS 助记词：转换为数字后派生密钥
- 非 NRCS 密码：使用 SHA-256 派生（向后兼容）

### 5. Account ID 生成严重错误 ❌
- **Java**: 使用公钥 SHA-256 哈希的**前 8 个字节**，**小端序**
- **Rust**: 使用公钥 SHA-256 哈希的**后 8 个字节**，**大端序**
- **结果**: 完全不兼容！必须修复

### 6. Public Key 派生错误 ❌
- 当前使用 SHA-256 直接哈希助记词
- 应该使用 `passphrase_to_keypair` 函数

### 7. 前端缺失功能 ❌
- `crypto.ts` 中没有 `generateKeypair` 函数
- 需要添加完整的 NRCS 单词列表和生成函数

## 时间估算

- 后端核心修复：2-3 小时
- 前端修复：2-3 小时
- 测试和文档：1-2 小时
- **总计**：5-8 小时
