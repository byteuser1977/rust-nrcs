# NRCS区块链出块流程分析

## 一、出块核心组件

### 1. Generator类 (nrcs-main/src/main/java/com/bytechain/nrcs/service/generator/Generator.java)
- **职责**：管理锻造者、计算出块时间、触发区块生成
- **关键属性**：
  - `generators`：活跃锻造者集合
  - `activeGeneratorIds`：活跃锻造者ID集合
  - `lastBlockId`：最后一个区块ID

### 2. BlockchainProcessor类 (nrcs-main/src/main/java/com/bytechain/nrcs/service/block/BlockchainProcessor.java)
- **职责**：处理区块链、生成新区块、验证区块

## 二、出块完整流程

### 阶段1：锻造准备 (在Generator类中)

1. **启动锻造**
   ```java
   Generator.startForging(String secretPhrase);
   ```
   - 创建新的Generator实例
   - 验证锻造者数量不超过MAX_FORGERS
   - 计算初始有效余额

2. **出块检测线程** (generateBlocksThread)
   - 每500ms执行一次
   - 获取最新区块
   - 检查是否需要回滚最后一个区块
   - 计算并排序锻造者

### 阶段2：出块权限判断

1. **计算Hit值** (getHit方法)
   ```java
   public static BigInteger getHit(byte[] publicKey, IBlock block)
   ```
   - 使用前一个区块的generation signature
   - 与锻造者公钥进行哈希计算
   - 生成hit值（一个大整数）

2. **计算出块时间** (getHitTime方法)
   ```java
   public static long getHitTime(BigInteger effectiveBalance, BigInteger hit, IBlock block)
   ```
   - hit值 / (baseTarget × 有效余额)
   - 加上前一个区块的时间戳
   - 得到理论出块时间

3. **验证出块权限** (verifyHit方法)
   ```java
   public static boolean verifyHit(BigInteger hit, BigInteger effectiveBalance, IBlock previousBlock, int timestamp)
   ```
   - 检查当前时间是否在有效范围内
   - 验证hit值是否小于 (baseTarget × 有效余额 × 经过时间)
   - 确保区块时间戳有效

### 阶段3：区块生成 (在BlockchainProcessor.generateBlock方法中)

1. **准备交易**
   - 处理等待中的交易
   - 选择未确认交易 (selectUnconfirmedTransactions)
   - 验证交易有效性
   - 检查引用交易是否存在

2. **构建区块数据**
   - 计算totalAmountNQT（交易总金额）
   - 计算totalFeeNQT（交易总费用）
   - 计算payloadHash（交易的哈希）
   - 计算generationSignature（新区块的生成签名）
     ```java
     digest.update(previousBlock.getGenerationSignature());
     byte[] generationSignature = digest.digest(publicKey);
     ```
   - 计算previousBlockHash（前一个区块的哈希）

3. **创建新区块**
   ```java
   Block block = new Block(getBlockVersion(previousBlock.getHeight()), 
                          blockTimestamp, 
                          previousBlock.getId(), 
                          totalAmountNQT, 
                          totalFeeNQT, 
                          payloadLength,
                          payloadHash, 
                          publicKey, 
                          generationSignature, 
                          previousBlockHash, 
                          blockTransactions, 
                          secretPhrase);
   ```

### 阶段4：区块验证与推送 (pushBlock方法)

1. **区块验证**
   - 验证版本号
   - 验证时间戳（不超过当前时间+max_timedrift）
   - 验证前一个区块的哈希
   - 验证区块ID有效性
   - 验证generation signature
   - 验证区块签名
   - 验证交易数量
   - 验证payload长度

2. **交易验证**
   - 验证每个交易的签名
   - 验证交易时间戳
   - 验证交易引用
   - 检查交易是否重复
   - 计算并验证总金额和费用
   - 验证payload哈希

3. **区块添加与处理**
   - 应用未确认交易
   - 添加区块到区块链
   - 应用区块（更新账户余额等）
   - 处理阶段交易
   - 通知区块添加监听器
   - 广播区块到其他节点

### 阶段5：事件通知

1. **BlockchainProcessorEvent**
   - BEFORE_BLOCK_ACCEPT：区块接受前
   - AFTER_BLOCK_ACCEPT：区块接受后
   - BEFORE_BLOCK_APPLY：区块应用前
   - AFTER_BLOCK_APPLY：区块应用后
   - BLOCK_PUSHED：区块已推送
   - BLOCK_GENERATED：区块已生成

2. **GeneratorEvent**
   - START_FORGING：开始锻造
   - STOP_FORGING：停止锻造
   - GENERATION_DEADLINE：生成截止时间

## 三、关键算法说明

### 1. 权益证明 (PoS) 机制
- **有效余额**：账户的有效余额影响出块概率
- **Hit值计算**：基于前一个区块的generation signature和账户公钥
- **出块时间**：hit值 / (baseTarget × 有效余额) + 前一个区块时间戳
- **验证条件**：hit < (baseTarget × 有效余额 × 经过时间)

### 2. 交易选择策略
- 按到达时间排序
- 验证交易有效性
- 检查引用交易
- 防止交易重复
- 限制区块大小和交易数量

### 3. BaseTarget调整机制

BaseTarget 是 NRCS 区块链中控制出块难度的关键参数，它在每个新区块创建时进行动态调整。

#### 3.1 BaseTarget的产生时机

在 `Block` 类的 `setPrevious` 方法中调用 `calculateBaseTarget` 方法，当一个新区块被链接到前一个区块时进行计算：

```java
public void setPrevious(Block block) {
    if (block != null) {
        if (block.getId() != getPreviousBlockId()) {
            throw new IllegalStateException("Previous block id doesn't match");
        }
        this.setHeight(block.getHeight() + 1);
        this.calculateBaseTarget(block);  // 计算新的 baseTarget
    } else {
        this.setHeight(0);
    }
    // ...
}
```

#### 3.2 初始值与常量定义

在 `Constant` 类中定义了与 baseTarget 相关的常量：

```java
public static final int BLOCK_TIME = 60;  // 目标出块时间：60秒

// 初始 baseTarget 值
public static final long INITIAL_BASE_TARGET = 
    BigInteger.valueOf(2).pow(63)
    .divide(BigInteger.valueOf(BLOCK_TIME * MAX_BALANCE_FXT))
    .longValue();

// 不同阶段的 baseTarget 范围
public static final long MAX_BASE_TARGET = MAX_BALANCE_NRCS * INITIAL_BASE_TARGET;
public static final long MAX_BASE_TARGET_2 = isTestnet ? MAX_BASE_TARGET : INITIAL_BASE_TARGET * 50;
public static final long MIN_BASE_TARGET = INITIAL_BASE_TARGET * 9 / 10;

// 时间调整相关参数
public static final int MIN_BLOCKTIME_LIMIT = 53;
public static final int MAX_BLOCKTIME_LIMIT = 67;
public static final int BASE_TARGET_GAMMA = 64;
```

#### 3.3 计算算法（calculateBaseTarget 方法）

BaseTarget 的计算分为两个阶段，根据 `SHUFFLING_BLOCK` 高度进行区分：

**阶段一（SHUFFLING_BLOCK 之前）：**

```java
private void calculateBaseTarget(Block previousBlock) {
    long baseTarget;
    long prevBaseTarget = previousBlock.getBaseTarget();
    
    if (previousBlock.getHeight() < Constant.SHUFFLING_BLOCK || 
        previousBlock.getId() == Genesis.GENESIS_BLOCK_ID) {
        
        // 简单调整算法
        baseTarget = BigInteger.valueOf(prevBaseTarget)
            .multiply(BigInteger.valueOf(this.getTimestamp() - previousBlock.getTimestamp()))
            .divide(BigInteger.valueOf(60))
            .longValue();
        
        // 边界约束
        if (baseTarget < 0 || baseTarget > Constant.MAX_BASE_TARGET) {
            baseTarget = Constant.MAX_BASE_TARGET;
        }
        if (baseTarget < prevBaseTarget / 2) {
            baseTarget = prevBaseTarget / 2;
        }
        if (baseTarget == 0) {
            baseTarget = 1;
        }
        long twofoldCurBaseTarget = prevBaseTarget * 2;
        if (twofoldCurBaseTarget < 0) {
            twofoldCurBaseTarget = Constant.MAX_BASE_TARGET;
        }
        if (baseTarget > twofoldCurBaseTarget) {
            baseTarget = twofoldCurBaseTarget;
        }
    }
    // ...
}
```

**阶段二（SHUFFLING_BLOCK 之后，每隔一个区块计算）：**

```java
else if (previousBlock.getHeight() % 2 == 0) {
    // 查找前前区块
    Block block = BlockService.findBlockAtHeight(previousBlock.getHeight() - 2);
    // 计算3个区块的平均出块时间
    int blocktimeAverage = (this.getTimestamp() - block.getTimestamp()) / 3;
    
    // 根据平均出块时间调整 baseTarget
    if (blocktimeAverage > 60) {
        // 出块太慢，提高 baseTarget（降低难度）
        baseTarget = (prevBaseTarget * Math.min(blocktimeAverage, Constant.MAX_BLOCKTIME_LIMIT)) / 60;
    } else {
        // 出块太快，降低 baseTarget（提高难度）
        baseTarget = prevBaseTarget - prevBaseTarget * Constant.BASE_TARGET_GAMMA * 
            (60 - Math.max(blocktimeAverage, Constant.MIN_BLOCKTIME_LIMIT)) / 6000;
    }
    
    // 边界约束
    if (baseTarget < 0 || baseTarget > Constant.MAX_BASE_TARGET_2) {
        baseTarget = Constant.MAX_BASE_TARGET_2;
    }
    if (baseTarget < Constant.MIN_BASE_TARGET) {
        baseTarget = Constant.MIN_BASE_TARGET;
    }
} else {
    // 奇数次区块保持与前一个相同的 baseTarget
    baseTarget = prevBaseTarget;
}
```

#### 3.4 计算原理

1. **目标**：保持平均出块时间稳定在 60 秒
2. **调整频率**：
   - 前期：每块都调整
   - 后期：每隔一个区块调整（偶数高度）
3. **调整方向**：
   - 出块平均时间 > 60秒 → 提高 baseTarget（降低难度，加快出块）
   - 出块平均时间 < 60秒 → 降低 baseTarget（提高难度，减慢出块）
4. **边界保护**：
   - 最低限制：MIN_BASE_TARGET = 初始值的90%
   - 最高限制：MAX_BASE_TARGET_2 = 初始值的50倍
   - 单次调整限制：不超过前一个值的2倍或1/2

#### 3.5 BaseTarget在出块中的作用

BaseTarget 影响出块概率和验证条件：

- **出块时间计算**：`hitTime = hit / (baseTarget × effectiveBalance) + previousBlockTimestamp`
- **验证条件**：`hit < (baseTarget × effectiveBalance × elapsedTime)`
- **难度理解**：baseTarget 越高，难度越低；baseTarget 越低，难度越高

#### 3.6 累积难度计算

除了 baseTarget 外，还计算累积难度：

```java
this.setCumulativeDifficulty(
    previousBlock.getCumulativeDifficulty().add(
        Convert.two64.divide(BigInteger.valueOf(baseTarget))
    )
);
```

累积难度用于比较不同链的难度，确定哪条链是主链。

## 四、流程图概览

```
开始锻造
   ↓
启动出块检测线程
   ↓
获取最新区块
   ↓
计算所有锻造者的hit值和出块时间
   ↓
排序锻造者
   ↓
检查是否有锻造者满足出块条件
   ↓
选择未确认交易
   ↓
构建新区块
   ↓
验证区块和交易
   ↓
添加区块到区块链
   ↓
应用区块更新
   ↓
广播区块到网络
   ↓
完成！
```

## 五、安全机制

1. **签名验证**：所有区块和交易都需要签名验证
2. **时间限制**：防止区块时间戳超前或过时
3. **引用验证**：交易引用的交易必须存在
4. **重复检查**：防止重复交易和区块
5. **难度调整**：通过baseTarget控制出块速度
6. **余额检查**：确保锻造者有足够权益

## 六、总结

NRCS区块链采用基于权益证明(PoS)的出块机制，核心流程包括：
1. 锻造者注册与管理
2. 持续计算出块时间和验证出块权限
3. 选择和验证交易
4. 生成、签名并验证新区块
5. 将区块添加到区块链并广播到网络
6. 通知监听器并更新状态

整个过程设计有完善的安全验证机制，确保区块链的安全性和一致性。