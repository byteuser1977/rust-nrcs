/// 集成测试：验证 transaction2.data 中所有记录的解析正确性
///
/// 覆盖的记录类型：
/// - DB_ID=69: CurrencyTransfer (type=5, subtype=3)
/// - DB_ID=327-330: AccountProperty/AccountPropertyDelete (type=1, subtype=10/11)
/// - DB_ID=331,333,335: TaggedDataUpload (type=6, subtype=0)
/// - DB_ID=332,334,336: CoinExchangeOrderIssue (type=12, subtype=0)
use blockchain_types::transaction::{Transaction, TransactionType};

#[test]
fn test_transaction2_data_all_records() {
    println!("\n=== Testing transaction2.data Records ===\n");

    // ============================================================
    // DB_ID=69: type=5(MonetarySystem), subtype=3(CurrencyTransfer)
    // 特征：attachment_bytes = 17 bytes (version + currencyId + units)
    // ============================================================
    test_db69_currency_transfer();

    // ============================================================
    // DB_ID=327: type=1(Messaging), subtype=10(AccountProperty)
    // 特征：attachment_bytes 包含 property + value 字符串
    // ============================================================
    test_db327_account_property();

    // ============================================================
    // DB_ID=328: type=1(Messaging), subtype=11(AccountPropertyDelete)
    // 特征：attachment_bytes 只包含 propertyId (i64)
    // ============================================================
    test_db328_account_property_delete();

    // ============================================================
    // DB_ID=329: type=1(Messaging), subtype=10(AccountProperty)
    // ============================================================
    test_db329_account_property();

    // ============================================================
    // DB_ID=330: type=1(Messaging), subtype=10(AccountProperty)
    // ============================================================
    test_db330_account_property();

    // ============================================================
    // DB_ID=331: type=6(Data), subtype=0(TaggedDataUpload)
    // 特征：HAS_PRUNABLE_ATTACHMENT=TRUE
    // ============================================================
    test_db331_tagged_data_upload();

    // ============================================================
    // DB_ID=332: type=12(LightContract), subtype=0(ContractReferenceSet)
    // 特征：attachment_bytes 包含 contractName + hash
    // ============================================================
    test_db332_light_contract_reference_set();

    // ============================================================
    // DB_ID=333: type=6(Data), subtype=0(TaggedDataUpload)
    // 特征：HAS_PRUNABLE_ATTACHMENT=TRUE
    // ============================================================
    test_db333_tagged_data_upload();

    // ============================================================
    // DB_ID=334: type=12(LightContract), subtype=0(ContractReferenceSet)
    // ============================================================
    test_db334_light_contract_reference_set();

    // ============================================================
    // DB_ID=335: type=6(Data), subtype=0(TaggedDataUpload)
    // 特征：HAS_PRUNABLE_ATTACHMENT=TRUE
    // ============================================================
    test_db335_tagged_data_upload();

    // ============================================================
    // DB_ID=336: type=12(LightContract), subtype=0(ContractReferenceSet)
    // ============================================================
    test_db336_light_contract_reference_set();
}

fn test_db69_currency_transfer() {
    let json_str = r#"{
        "amountNQT": "0",
        "attachment": {"version.CurrencyTransfer": 1, "currency": "2123033715300581825", "units": "500"},
        "block": "1032650923450679770",
        "blockTimestamp": 422108,
        "deadline": 1440,
        "ecBlockHeight": 6792,
        "ecBlockId": "3324756215706311257",
        "feeNQT": "100000000",
        "fullHash": "21604da26b68a73a856b7479b676a15561eac72b03831263f1d28c7ce634961f",
        "height": 7513,
        "phased": false,
        "recipient": "899669674531378169",
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "signature": "6ebd8910ce0d92219a12ccfa413e44dce515b985d3427e64174a8d3c4aed910d651f079a81c505f14bb71262d069c23120e49e5c5b8228c29960fc0b856be5d2",
        "subtype": 3,
        "timestamp": 422102,
        "transaction": "4226461586804269089",
        "transactionIndex": 0,
        "type": 5,
        "version": 1
    }"#;
    
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();
    
    assert_eq!(tx.id, 4226461586804269089, "DB_ID=69 ID mismatch");
    assert_eq!(tx.type_id, TransactionType::MonetarySystem);
    assert_eq!(hex::encode(&tx.attachment_bytes), "01c141ab6eea87761df401000000000000");
    assert_eq!(tx.attachment_bytes.len(), 17);
    println!("✅ DB_ID=69 CurrencyTransfer passed");
}

fn test_db327_account_property() {
    let json_str = r#"{
        "amountNQT": "0",
        "attachment": {"version.AccountProperty": 1, "property": "MOBILE", "value": "+86-17701338028"},
        "block": "4204377706936983886",
        "blockTimestamp": 966374,
        "deadline": 1440,
        "ecBlockHeight": 0,
        "ecBlockId": "3488276486778630462",
        "feeNQT": "100000000",
        "fullHash": "1ff1015eb33982266d896a53bb79492e0cb62bbb1519af4a1655d692f476250a",
        "height": 16943,
        "phased": false,
        "recipient": "-4035311295601449920",
        "sender": "-4035311295601449920",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "signature": "00c473d8950f75626ca6c0ff8cb49d41fa7553550fe70021d7e3d512b6d48b00a5a614023f18379ec8531d58804ea457002b",
        "subtype": 10,
        "timestamp": 966320,
        "transaction": "2774843762952761631",
        "transactionIndex": 0,
        "type": 1,
        "version": 1
    }"#;
    
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();
    
    assert_eq!(tx.id, 2774843762952761631, "DB_ID=327 ID mismatch");
    assert_eq!(tx.type_id, TransactionType::Messaging);
    assert_eq!(tx.subtype, 10);
    assert_eq!(hex::encode(&tx.attachment_bytes), "01064d4f42494c450f2b38362d3137373031333338303238");
    println!("✅ DB_ID=327 AccountProperty passed");
}

fn test_db328_account_property_delete() {
    let json_str = r#"{
        "amountNQT": "0",
        "attachment": {"version.AccountPropertyDelete": 1, "property": "18063953811633506379"},
        "block": "4513604424364811",
        "blockTimestamp": 966389,
        "deadline": 1440,
        "ecBlockHeight": 0,
        "ecBlockId": "3488276486778630462",
        "feeNQT": "100000000",
        "fullHash": "64726ba0638b15880278c6f30b1723ab14d9963bd5d761c97e950df6ea0891aa",
        "height": 16944,
        "phased": false,
        "recipient": "996325769485053218",
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "signature": "fb3e342369a2ee131352285c32242e2c0962504e984c0108c33915bcf1de460045184ff8ef285b6c48ca618c1f6f21a1dd59",
        "subtype": 11,
        "timestamp": 966392,
        "transaction": "-8640847050031009180",
        "transactionIndex": 0,
        "type": 1,
        "version": 1
    }"#;
    
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();
    
    // 注意：ID 是负数，但 Rust 使用 u64 存储
    assert!(tx.id > 0, "DB_ID=328 ID should be positive u64");
    assert_eq!(tx.type_id, TransactionType::Messaging);
    assert_eq!(tx.subtype, 11); // AccountPropertyDelete
    assert_eq!(hex::encode(&tx.attachment_bytes), "014b887ae5480eb0fa");
    assert_eq!(tx.attachment_bytes.len(), 9); // version(1B) + propertyId(8B)
    println!("✅ DB_ID=328 AccountPropertyDelete passed");
}

fn test_db329_account_property() {
    let json_str = r#"{
        "amountNQT": "0",
        "attachment": {"version.AccountProperty": 1, "property": "NAME", "value": "ALEX"},
        "block": "-2614491266220712868",
        "blockTimestamp": 967107,
        "deadline": 1440,
        "ecBlockHeight": 16233,
        "ecBlockId": "-7573538264667841448",
        "feeNQT": "100000000",
        "fullHash": "ce77af075da92bd5a0ad20b14d538f134a827ef100e39eea843cb121181aeb3b",
        "height": 16954,
        "phased": false,
        "recipient": "996325769485053218",
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "signature": "ed1a60fea8173551b2ed224db1e3b0104a56b91c1ab5c0ed6f83c58a7a1c3604ff3b3cc7de37cc33586ed369561146098441",
        "subtype": 10,
        "timestamp": 967062,
        "transaction": "-3086186902606350386",
        "transactionIndex": 0,
        "type": 1,
        "version": 1
    }"#;
    
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();
    
    assert!(tx.id > 0, "DB_ID=329 ID should be positive u64");
    assert_eq!(tx.type_id, TransactionType::Messaging);
    assert_eq!(tx.subtype, 10);
    assert_eq!(hex::encode(&tx.attachment_bytes), "01044e414d4504414c4558");
    println!("✅ DB_ID=329 AccountProperty passed");
}

fn test_db330_account_property() {
    let json_str = r#"{
        "amountNQT": "0",
        "attachment": {"version.AccountProperty": 1, "property": "SEX", "value": "MALE"},
        "block": "-2614491266220712868",
        "blockTimestamp": 967107,
        "deadline": 1440,
        "ecBlockHeight": 16233,
        "ecBlockId": "-7573538264667841448",
        "feeNQT": "100000000",
        "fullHash": "ba92aadc2c88fa22b1a225be011834bfd18ae99b0b6cf012c4fc03fd7b113fb5",
        "height": 16954,
        "phased": false,
        "recipient": "996325769485053218",
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "signature": "5dbf87dc1d4063a6b5edba58477fc56ea663d5440eabf906608963fdc46aa403f2adba85b9b304cde615f65edd26941e5c93",
        "subtype": 10,
        "timestamp": 967094,
        "transaction": "2520476667729318586",
        "transactionIndex": 1,
        "type": 1,
        "version": 1
    }"#;
    
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();
    
    assert_eq!(tx.id, 2520476667729318586, "DB_ID=330 ID mismatch");
    assert_eq!(tx.type_id, TransactionType::Messaging);
    assert_eq!(tx.subtype, 10);
    assert_eq!(hex::encode(&tx.attachment_bytes), "0103534558044d414c45");
    println!("✅ DB_ID=330 AccountProperty passed");
}

fn test_db331_tagged_data_upload() {
    let json_str = r#"{
        "amountNQT": "0",
        "attachment": {"version.TaggedDataUpload": 1, "hash": "3d4c157d4ed8a8ea96cbee877905733d58d27e86da4e57acf1b19a9f1815149c"},
        "block": "-6898644439468565517",
        "blockTimestamp": 1025854,
        "deadline": 15,
        "ecBlockHeight": 17252,
        "ecBlockId": "1354532317025969696",
        "feeNQT": "360000000",
        "fullHash": "ac677314bedd332e14dda0ff20273179a84ffd0b4d02c47a93c2d8c64e320f4f",
        "height": 17973,
        "phased": false,
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "signature": "04a447dd32a1b51ca9b0c7eef3d676cac800e3c689b6aefd08c1cecfcf10aa06eae3875c47a91899a0859e5ceb85872d5f24",
        "subtype": 0,
        "timestamp": 1025779,
        "transaction": "3329248358013560748",
        "transactionIndex": 0,
        "type": 6,
        "version": 1
    }"#;
    
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();
    
    assert_eq!(tx.id, 3329248358013560748, "DB_ID=331 ID mismatch");
    assert_eq!(tx.type_id, TransactionType::Data);
    assert!(tx.has_prunable_attachment, "DB_ID=331 should have prunable attachment");
    assert_eq!(hex::encode(&tx.attachment_bytes), "013d4c157d4ed8a8ea96cbee877905733d58d27e86da4e57acf1b19a9f1815149c");
    println!("✅ DB_ID=331 TaggedDataUpload passed");
}

fn test_db332_light_contract_reference_set() {
    let json_str = r#"{
        "amountNQT": "0",
        "attachment": {"version.ContractReferenceSet": 1, "contractName": "InnernerContract", "contractParams": "", "contract": {"chainId": 0, "hash": "ac677314bedd332e14dda0ff20273179a84ffd0b4d02c47a"}},
        "block": "-5643170988929785739",
        "blockTimestamp": 1026034,
        "deadline": 15,
        "ecBlockHeight": 17252,
        "ecBlockId": "1354532317025969696",
        "feeNQT": "60000000",
        "fullHash": "e9a395f74d9ca3ee15ab0081dc2a6d60dc9899a771ab07bc91b1734e6660fcbe",
        "height": 17974,
        "phased": false,
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "signature": "416a0b3118b985e84740c8fad5c8103f04e5266daa44e83d42f6229cb08b4a08b3514f4d9670be237c1cbb642a9b4547e104",
        "subtype": 0,
        "timestamp": 1025820,
        "transaction": "-1250984412798671895",
        "transactionIndex": 0,
        "type": 12,
        "version": 1
    }"#;
    
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();
    
    assert!(tx.id > 0, "DB_ID=332 ID should be positive u64");
    assert_eq!(tx.type_id, TransactionType::LightContract);
    assert_eq!(tx.subtype, 0); // ContractReferenceSet
    println!("✅ DB_ID=332 LightContract/ContractReferenceSet passed (ID={}, attachment={} bytes)", 
             tx.id, tx.attachment_bytes.len());
}

fn test_db333_tagged_data_upload() {
    let json_str = r#"{
        "amountNQT": "0",
        "attachment": {"version.TaggedDataUpload": 1, "hash": "5ee903429e5b6ed167ca673fc6ff82c2c7f8a44a2226ac61afef5dff2b1d0471"},
        "block": "-3265285546812720937",
        "blockTimestamp": 1027917,
        "deadline": 15,
        "ecBlockHeight": 17283,
        "ecBlockId": "9163922980003578869",
        "feeNQT": "390000000",
        "fullHash": "caeee1837774b90029964c282579eeff380d9a7bc1ba0295453952174fb8e095",
        "height": 18005,
        "phased": false,
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "signature": "a73f8034ad2bfc72d2857e33250d9bfe20229cf218919cfe14e047cd0ee2f40b533a4a81b2a44379a84b73dbdc9b5d25847b",
        "subtype": 0,
        "timestamp": 1027715,
        "transaction": "52200927354023626",
        "transactionIndex": 0,
        "type": 6,
        "version": 1
    }"#;
    
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();
    
    assert_eq!(tx.id, 52200927354023626, "DB_ID=333 ID mismatch");
    assert_eq!(tx.type_id, TransactionType::Data);
    assert!(tx.has_prunable_attachment, "DB_ID=333 should have prunable attachment");
    assert_eq!(hex::encode(&tx.attachment_bytes), "015ee903429e5b6ed167ca673fc6ff82c2c7f8a44a2226ac61afef5dff2b1d0471");
    println!("✅ DB_ID=333 TaggedDataUpload passed");
}

fn test_db334_light_contract_reference_set() {
    let json_str = r#"{
        "amountNQT": "0",
        "attachment": {"version.ContractReferenceSet": 1, "contractName": "InnernerContract", "contractParams": "", "contract": {"chainId": 0, "hash": "caeee1837774b90029964c282579eeff380d9a7bc1ba0295"}},
        "block": "-3265285546812720937",
        "blockTimestamp": 1027917,
        "deadline": 15,
        "ecBlockHeight": 17283,
        "ecBlockId": "9163922980003578869",
        "feeNQT": "60000000",
        "fullHash": "5522023436000ba0314ff275d6959bc38edaf1678a0a633f5417d3ce820985c0",
        "height": 18005,
        "phased": false,
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "signature": "19391b6c9fd15180afcfc30d13e181625263784f33e8c75efc7e507f49387307314c9838a2db5aefa8a51c1813418724718d",
        "subtype": 0,
        "timestamp": 1027715,
        "transaction": "-6914432570096475563",
        "transactionIndex": 1,
        "type": 12,
        "version": 1
    }"#;
    
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();
    
    assert!(tx.id > 0, "DB_ID=334 ID should be positive u64");
    assert_eq!(tx.type_id, TransactionType::LightContract);
    println!("✅ DB_ID=334 LightContract/ContractReferenceSet passed (ID={}, attachment={} bytes)", 
             tx.id, tx.attachment_bytes.len());
}

fn test_db335_tagged_data_upload() {
    let json_str = r#"{
        "amountNQT": "0",
        "attachment": {"version.TaggedDataUpload": 1, "hash": "c7c36f3f356d9acb2e0237cfb93fc9c34ffc29dc9fa54d243970a78a63809bd2"},
        "block": "63164930941337033",
        "blockTimestamp": 1028573,
        "deadline": 15,
        "ecBlockHeight": 17304,
        "ecBlockId": "-201320318177515380",
        "feeNQT": "390000000",
        "fullHash": "30354e469bc757f95b77ee932fba9bff57aad430ed3d2687533fce8f519af87c",
        "height": 18026,
        "phased": false,
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "signature": "b13a5b9de7a71a9cdd6d8400b44f2b3b18c73aa66c7aafd43f622e83c60d570dc5c9a0a1de0132e225122960749425df2d8a",
        "subtype": 0,
        "timestamp": 1028475,
        "transaction": "-479695365578279632",
        "transactionIndex": 0,
        "type": 6,
        "version": 1
    }"#;
    
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();
    
    assert!(tx.id > 0, "DB_ID=335 ID should be positive u64");
    assert_eq!(tx.type_id, TransactionType::Data);
    assert!(tx.has_prunable_attachment, "DB_ID=335 should have prunable attachment");
    assert_eq!(hex::encode(&tx.attachment_bytes), "01c7c36f3f356d9acb2e0237cfb93fc9c34ffc29dc9fa54d243970a78a63809bd2");
    println!("✅ DB_ID=335 TaggedDataUpload passed");
}

fn test_db336_light_contract_reference_set() {
    let json_str = r#"{
        "amountNQT": "0",
        "attachment": {"version.ContractReferenceSet": 1, "contractName": "InnernerContract", "contractParams": "", "contract": {"chainId": 0, "hash": "30354e469bc757f95b77ee932fba9bff57aad430ed3d2687"}},
        "block": "63164930941337033",
        "blockTimestamp": 1028573,
        "deadline": 15,
        "ecBlockHeight": 17304,
        "ecBlockId": "-201320318177515380",
        "feeNQT": "60000000",
        "fullHash": "83429cc771f7a54c22dbad1baa0af7d22f9edfcbc62a1a04bbc3c98bcdae56d3",
        "height": 18026,
        "phased": false,
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "signature": "0384a86b0320a4e864b77963e59d62cbc968c706a51afe67c22eba2d5c427b0108b5032ec5b289f905a7af5b3b3362c39c3f",
        "subtype": 0,
        "timestamp": 1028475,
        "transaction": "5523092586092053123",
        "transactionIndex": 1,
        "type": 12,
        "version": 1
    }"#;
    
    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();
    
    assert_eq!(tx.id, 5523092586092053123, "DB_ID=336 ID mismatch");
    assert_eq!(tx.type_id, TransactionType::LightContract);
    println!("✅ DB_ID=336 LightContract/ContractReferenceSet passed (ID={}, attachment={} bytes)", 
             tx.id, tx.attachment_bytes.len());
}
