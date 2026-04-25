# 区块请求与响应日志

**测试日期**：2026-04-25  
**测试目标**：记录 Rust NRCS 节点与 Java NRCS 节点之间的区块请求和响应内容  
**测试地址**：192.168.110.87  

## 日志级别
- **配置**：debug 级别
- **目的**：详细记录区块请求和响应内容

## 连接过程

```
2026-04-25T09:00:59.741343Z  INFO nrcs_node: [BOOTSTRAP] Connecting to 192.168.110.87:17974...
2026-04-25T09:00:59.742189Z  INFO p2p::websocket: [CLIENT] Attempting to connect to 192.168.110.87:17974 via WebSocket
2026-04-25T09:00:59.742244Z  INFO p2p::websocket: [CLIENT] Connecting to WebSocket URL: ws://192.168.110.87:17974/nrcs
2026-04-25T09:00:59.764786Z  INFO p2p::websocket: [CLIENT] WebSocket connected to 192.168.110.87:17974
2026-04-25T09:00:59.764892Z DEBUG p2p::websocket: [CLIENT] Creating getInfo request
2026-04-25T09:00:59.765264Z DEBUG p2p::websocket: [CLIENT] Serialized getInfo request (38 bytes)
2026-04-25T09:00:59.765291Z DEBUG p2p::websocket: [CLIENT] Sending binary frame (58 bytes)
2026-04-25T09:00:59.765437Z DEBUG p2p::websocket: [CLIENT] Sent getInfo (binary) to 192.168.110.87:17974
2026-04-25T09:00:59.772690Z DEBUG p2p::websocket: [CLIENT] Received binary frame (243 bytes) from 192.168.110.87:17974
2026-04-25T09:00:59.772743Z DEBUG p2p::websocket: [CLIENT] Decoded frame: req_id=0, body_len=223
2026-04-25T09:00:59.772753Z DEBUG p2p::websocket: [CLIENT] Parsing PeerResponse from 192.168.110.87:17974 (223 bytes)
2026-04-25T09:00:59.772847Z DEBUG p2p::websocket: [CLIENT] Received PeerResponse: PeerResponse { error: None, data: Some(Object {"apiPort": Number(17976), "apiServerIdleTimeout": Number(30000), "application": String("NRcS"), "blockchainState": Number(0), "disabledAPIs": String(""), "platform": String("Linux aarch64"), "protocol": Number(1), "requestType": String("getInfo"), "services": String("20"), "shareAddress": Bool(true), "version": String("2.1.0")}) }
2026-04-25T09:00:59.773538Z  INFO p2p::websocket: Handshake successful with 192.168.110.87:17974
```

## 区块同步过程

### 区块请求示例

```
2026-04-25T09:01:28.280892Z DEBUG p2p::daemon::blockchain_sync: Raw block JSON: {
  "blockSignature": "5d085827c148c1c97b0bc6acb85b0f44e1f586dbb2decab31abbbe71e9ae58056a78eaa5c15d531e9dda4908bb3df933b2ab6a3c51139b85021ad893baeeecf6",
  "generationSignature": "69e144b6655fb4c6364754e7018716a884ab0396fa824325bdeb01f5f1ab1951",
  "generatorPublicKey": "21a908b060ca909e4f4f665aa31243f51817e9cbd32ddbb751fcef2373aeba1f",
  "payloadHash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "payloadLength": 0,
  "previousBlock": "10406956222173143494",
  "previousBlockHash": "c6e1abaf8bef6c90cbbca61f88329848ab8ecd626d66c8108a21b088889ebaaa",
  "timestamp": 40894,
  "totalAmountNQT": 0,
  "totalFeeNQT": 0,
  "transactions": [],
  "version": 3
}
```

### 区块处理过程

```
2026-04-25T09:01:28.280912Z DEBUG p2p::daemon::blockchain_sync: Converting account_id field: generator_id = 21a908b060ca909e4f4f665aa31243f51817e9cbd32ddbb751fcef2373aeba1f
2026-04-25T09:01:28.280915Z DEBUG p2p::daemon::blockchain_sync: Converted generator_id to account_id: 11425854771257977121
2026-04-25T09:01:28.280917Z DEBUG p2p::daemon::blockchain_sync: Converted previous_block to u64: 10406956222173143494
2026-04-25T09:01:28.280924Z DEBUG p2p::daemon::blockchain_sync: Normalized block JSON keys: Some(["base_target:0", "block_signature:64", "cumulative_difficulty:0", "generation_signature:32", "generator_id:0", "height:0", "nonce:0", "payload_hash:32", "payload_length:0", "previous_block:0", "previous_block_hash:32", "timestamp:0", "total_amount:0", "total_amount_n_q_t:0", "total_fee:0", "total_fee_n_q_t:0", "transactions:0", "version:0"])
2026-04-25T09:01:28.280929Z DEBUG p2p::daemon::blockchain_sync: Processing downloaded block: height=0
2026-04-25T09:01:28.280981Z DEBUG p2p::verifier: Previous block 10406956222173143494 not found, but block has height info, using it
2026-04-25T09:01:28.281114Z  WARN p2p::daemon::blockchain_sync: Block verification/processing failed: database error: database error: error returned from database: (code: 2067) UNIQUE constraint failed: BLOCK.HEIGHT
2026-04-25T09:01:28.281118Z  WARN p2p::daemon::blockchain_sync: Failed to process downloaded block: database error: database error: error returned from database: (code: 2067) UNIQUE constraint failed: BLOCK.HEIGHT
```

### 另一个区块示例

```
2026-04-25T09:01:28.281329Z DEBUG p2p::daemon::blockchain_sync: Raw block JSON: {
  "blockSignature": "7a1f7447829165dc9991ce90def80c712b6cf429f5049218ac216269787c5f019eb0e33dfb9d6fb4d0b6786ca3ce29737d70692dc10ecf0a81316177f45284b2",
  "generationSignature": "213b17f3b93cc61a4e24d889aae71e9a5e68f8e414a12371d3b3939d0c745eee",
  "generatorPublicKey": "21a908b060ca909e4f4f665aa31243f51817e9cbd32ddbb751fcef2373aeba1f",
  "payloadHash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "payloadLength": 0,
  "previousBlock": "729687304856484507",
  "previousBlockHash": "9b3a15dbbc5e200a2f5185131c790c063dbdff3d5957115277f61ac2b7618d2b",
  "timestamp": 40985,
  "totalAmountNQT": 0,
  "totalFeeNQT": 0,
  "transactions": [],
  "version": 3
}
```

## 错误信息

```
2026-04-25T09:01:28.281521Z  WARN p2p::daemon::blockchain_sync: Block verification/processing failed: database error: database error: error returned from database: (code: 2067) UNIQUE constraint failed: BLOCK.HEIGHT
2026-04-25T09:01:28.281526Z  WARN p2p::daemon::blockchain_sync: Failed to process downloaded block: database error: database error: error returned from database: (code: 2067) UNIQUE constraint failed: BLOCK.HEIGHT
```

## 同步统计

```
2026-04-25T09:01:28.281545Z  INFO p2p::daemon::blockchain_sync: Downloaded 36 blocks total
```

## 日志分析

1. **连接成功**：Rust 节点成功连接到 Java 节点（192.168.110.87:17974），握手完成

2. **区块请求**：节点成功从 Java 节点下载区块数据，每个区块包含以下字段：
   - blockSignature：区块签名
   - generationSignature：生成签名
   - generatorPublicKey：生成者公钥
   - payloadHash：负载哈希
   - payloadLength：负载长度
   - previousBlock：前一个区块ID
   - previousBlockHash：前一个区块哈希
   - timestamp：时间戳
   - totalAmountNQT：总金额
   - totalFeeNQT：总费用
   - transactions：交易列表
   - version：版本

3. **处理过程**：
   - 转换生成者ID为账户ID
   - 转换前一个区块ID为u64类型
   - 规范化区块JSON键
   - 处理下载的区块

4. **错误**：
   - 数据库约束错误：UNIQUE constraint failed: BLOCK.HEIGHT
   - 原因：区块高度重复，可能是同步过程中重复处理同一区块

5. **同步结果**：
   - 总共下载了36个区块
   - 但由于数据库约束错误，无法成功处理这些区块

## 结论

Rust NRCS 节点能够成功连接到 Java NRCS 节点并下载区块数据，但在处理区块时遇到了数据库约束错误。日志中详细记录了区块请求和响应的内容，以及处理过程中的错误信息，这对于调试和解决问题非常有帮助。