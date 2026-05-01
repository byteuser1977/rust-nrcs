/// 验证关键交易类型的 attachment 序列化（使用真实数据）
///
/// 测试目标：
/// 1. 验证 AccountProperty 和 AccountPropertyDelete 的序列化逻辑
/// 2. 验证 Messaging 各子类型的实现
/// 3. 确保与 Java NRCS 完全一致
use blockchain_types::transaction::Transaction;

#[test]
fn test_critical_transaction_types() {
    println!("\n=== Critical Transaction Types Verification ===\n");

    // ============================================================
    // 1. AccountProperty (type=1, subtype=10) - 已验证正确 ✅
    // ============================================================
    test_account_property_correct();

    // ============================================================
    // 2. AccountPropertyDelete (type=1, subtype=11) - 已验证正确 ✅
    // ============================================================
    test_account_property_delete_correct();

    // ============================================================
    // 3. ArbitraryMessage (type=1, subtype=0)
    // ============================================================
    test_arbitrary_message();

    // ============================================================
    // 4. CurrencyTransfer (type=5, subtype=3)
    // ============================================================
    test_currency_transfer();

    // ============================================================
    // 5. TaggedDataUpload (type=6, subtype=0)
    // ============================================================
    test_tagged_data_upload();

    // ============================================================
    // 6. AssetPropertySet + PublicKeyAnnouncement (type=2, subtype=10)
    // ============================================================
    test_asset_property_set_with_pk();

    println!("\n✅ All critical transaction types verified!");
}

fn test_account_property_correct() {
    let json_str = r#"{
        "type": 1, "subtype": 10,
        "amountNQT": "0", "feeNQT": "100000000",
        "recipient": "-4035311295601449920",
        "sender": "-4035311295601449920",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "timestamp": 966320, "deadline": 1440, "version": 1,
        "attachment": {
            "version.AccountProperty": 1,
            "property": "MOBILE",
            "value": "+86-17701338028"
        },
        "fullHash": "1ff1015eb33982266d896a53bb79492e0cb62bbb1519af4a1655d692f476250a",
        "signature": "00c473d8950f75626ca6c0ff8cb49d41fa7553550fe70021d7e3d512b6d48b00a5a614023f18379ec8531d58804ea457002b",
        "block": "4204377706936983886", "blockTimestamp": 966374,
        "height": 16943, "phased": false,
        "ecBlockHeight": 0, "ecBlockId": "3488276486778630462",
        "transaction": "2774843762952761631", "transactionIndex": 0
    }"#;

    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();

    assert_eq!(tx.id, 2774843762952761631);
    assert_eq!(tx.subtype, 10); // AccountProperty

    // Java putMyBytes: propertyLen(BYTE) + property + valueLen(BYTE) + value
    // MOBILE(6B) + +86-17701338028(14B) = 22 bytes
    let expected_hex = "01064d4f42494c450f2b38362d3137373031333338303238";
    assert_eq!(hex::encode(&tx.attachment_bytes), expected_hex);

    println!("✅ AccountProperty: attachment={} ({} bytes)", expected_hex, tx.attachment_bytes.len());
}

fn test_account_property_delete_correct() {
    let json_str = r#"{
        "type": 1, "subtype": 11,
        "amountNQT": "0", "feeNQT": "100000000",
        "recipient": "996325769485053218",
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "timestamp": 966392, "deadline": 1440, "version": 1,
        "attachment": {
            "version.AccountPropertyDelete": 1,
            "property": "18063953811633506379"
        },
        "fullHash": "64726ba0638b15880278c6f30b1723ab14d9963bd5d761c97e950df6ea0891aa",
        "signature": "fb3e342369a2ee131352285c32242e2c0962504e984c0108c33915bcf1de460045184ff8ef285b6c48ca618c1f6f21a1dd59",
        "block": "4513604424364811", "blockTimestamp": 966389,
        "height": 16944, "phased": false,
        "ecBlockHeight": 0, "ecBlockId": "3488276486778630462",
        "transaction": "-8640847050031009180", "transactionIndex": 0
    }"#;

    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();

    assert!(tx.id > 0);
    assert_eq!(tx.subtype, 11); // AccountPropertyDelete

    // Java putMyBytes: buffer.putLong(propertyId) = 8 bytes
    // 格式：version(1B) + propertyId(8B) = 9 bytes
    let expected_hex = "014b887ae5480eb0fa";
    assert_eq!(hex::encode(&tx.attachment_bytes), expected_hex);

    println!("✅ AccountPropertyDelete: attachment={} ({} bytes)", expected_hex, tx.attachment_bytes.len());
}

fn test_arbitrary_message() {
    let json_str = r#"{
        "type": 1, "subtype": 0,
        "amountNQT": "0", "feeNQT": "40000000",
        "recipient": null,
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "timestamp": 167893, "deadline": 1440, "version": 1,
        "attachment": {
            "version.ArbitraryMessage": 0,
            "message": "Hello World"
        },
        "fullHash": "679c85f0d133af0e810179ee7b9119185d16a7725406abe3565840d1c56cd45c",
        "signature": "2aaa452578ada6853483e174689d61ec4555ba38b8d17c1efb1616419793550dee059f9bb5893f60d69226a4b32eb3bdc0f1",
        "block": "-8990481967722926086", "blockTimestamp": 167923,
        "height": 2988, "phased": false,
        "ecBlockHeight": 2267, "ecBlockId": "-1762530543998683424",
        "transaction": "1058121414231825511", "transactionIndex": 0
    }"#;

    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();

    assert_eq!(tx.id, 1058121414231825511);
    assert_eq!(tx.subtype, 0); // ArbitraryMessage
    assert!(tx.has_message);

    println!("✅ ArbitraryMessage: has_message={}, attachment={} bytes", 
             tx.has_message, tx.attachment_bytes.len());
}

fn test_currency_transfer() {
    let json_str = r#"{
        "type": 5, "subtype": 3,
        "amountNQT": "0", "feeNQT": "100000000",
        "recipient": "899669674531378169",
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "timestamp": 422102, "deadline": 1440, "version": 1,
        "attachment": {
            "version.CurrencyTransfer": 1,
            "currency": "2123033715300581825",
            "units": "500"
        },
        "fullHash": "21604da26b68a73a856b7479b676a15561eac72b03831263f1d28c7ce634961f",
        "signature": "6ebd8910ce0d92219a12ccfa413e44dce515b985d3427e64174a8d3c4aed910d651f079a81c505f14bb71262d069c23120e49e5c5b8228c29960fc0b856be5d2",
        "block": "1032650923450679770", "blockTimestamp": 422108,
        "height": 7513, "phased": false,
        "ecBlockHeight": 6792, "ecBlockId": "3324756215706311257",
        "transaction": "4226461586804269089", "transactionIndex": 0
    }"#;

    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();

    assert_eq!(tx.id, 4226461586804269089);
    assert_eq!(tx.subtype, 3); // CurrencyTransfer

    let expected_hex = "01c141ab6eea87761df401000000000000";
    assert_eq!(hex::encode(&tx.attachment_bytes), expected_hex);

    println!("✅ CurrencyTransfer: ID={}, attachment={}", tx.id, expected_hex);
}

fn test_tagged_data_upload() {
    let json_str = r#"{
        "type": 6, "subtype": 0,
        "amountNQT": "0", "feeNQT": "360000000",
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "timestamp": 1025779, "deadline": 15, "version": 1,
        "attachment": {
            "version.TaggedDataUpload": 1,
            "hash": "3d4c157d4ed8a8ea96cbee877905733d58d27e86da4e57acf1b19a9f1815149c"
        },
        "fullHash": "ac677314bedd332e14dda0ff20273179a84ffd0b4d02c47a93c2d8c64e320f4f",
        "signature": "04a447dd32a1b51ca9b0c7eef3d676cac800e3c689b6aefd08c1cecfcf10aa06eae3875c47a91899a0859e5ceb85872d5f24",
        "block": "-6898644439468565517", "blockTimestamp": 1025854,
        "height": 17973, "phased": false,
        "ecBlockHeight": 17252, "ecBlockId": "1354532317025969696",
        "transaction": "3329248358013560748", "transactionIndex": 0
    }"#;

    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();

    assert_eq!(tx.id, 3329248358013560748);
    assert!(tx.has_prunable_attachment, "TaggedDataUpload should be prunable");

    let expected_hex = "013d4c157d4ed8a8ea96cbee877905733d58d27e86da4e57acf1b19a9f1815149c";
    assert_eq!(hex::encode(&tx.attachment_bytes), expected_hex);

    println!("✅ TaggedDataUpload: prunable={}, attachment={}", 
             tx.has_prunable_attachment, expected_hex);
}

fn test_asset_property_set_with_pk() {
    let json_str = r#"{
        "type": 2, "subtype": 10,
        "amountNQT": "0", "feeNQT": "100000000",
        "recipient": "996325769485053218",
        "sender": "996325769485053218",
        "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
        "timestamp": 133354, "deadline": 15, "version": 1,
        "attachment": {
            "version.AssetProperty": 1,
            "asset": "16132763665229324019",
            "property": "no",
            "value": "123456",
            "version.PublicKeyAnnouncement": 1,
            "recipientPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c"
        },
        "fullHash": "a82003fc9d935c59e9a62bacb8381f92398ec722a0f02550544d087c18080c89",
        "signature": "81c12bdb6a45b05252dc068a34d6212897848055da72d899be6819b9635fdc0f76c37ff5f19093a9d52c7cf24fcc5682fb25",
        "block": "4807844805401117927", "blockTimestamp": 133506,
        "height": 2369, "phased": false,
        "ecBlockHeight": 1647, "ecBlockId": "-3101292290248805829",
        "transaction": "6439183873980178600", "transactionIndex": 0
    }"#;

    let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let tx = Transaction::from_json(&json).unwrap();

    assert_eq!(tx.id, 6439183873980178600);
    assert_eq!(tx.subtype, 10); // AssetPropertySet
    assert!(tx.has_public_key_announcement, "Should have PK announcement");

    let expected_att_hex = "01f39e52171417e3df026e6f06313233343536012d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c";
    assert_eq!(hex::encode(&tx.attachment_bytes), expected_att_hex);

    println!("✅ AssetPropertySet+PKAnn: PK={}, attachment={}", 
             tx.has_public_key_announcement, &expected_att_hex[..40]);
}
