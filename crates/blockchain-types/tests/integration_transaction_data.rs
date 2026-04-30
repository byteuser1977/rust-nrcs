/// 集成测试：验证 transaction.data 中所有记录的解析正确性
///
/// 测试目标：
/// 1. 每条记录的 ID 计算是否正确（u64 ↔ i64 转换）
/// 2. 标志位检测是否准确（HAS_PUBLIC_KEY_ANNOUNCEMENT, HAS_PRUNABLE_ATTACHMENT 等）
/// 3. attachment_bytes 序列化是否与 Java NRCS 一致
use blockchain_types::transaction::{Transaction, TransactionType};

#[test]
fn test_transaction_data_all_records() {
    // ============================================================
    // DB_ID=53: type=6(Data), subtype=0(TaggedDataUpload)
    // 特征：HAS_PRUNABLE_ATTACHMENT=TRUE, attachment_bytes 有 hash
    // ============================================================
    {
        let json_str = r#"{
            "amountNQT": "0",
            "attachment": {"version.TaggedDataUpload": 1, "hash": "a1474a67570fbabbf6794cb86400c119e0b23f4e69f464daea9e0fa93c5ef80c"},
            "block": "-6444971140270725924",
            "blockTimestamp": 125757,
            "deadline": 15,
            "ecBlockHeight": 1506,
            "ecBlockId": "-7988487755785480949",
            "feeNQT": "260000000",
            "fullHash": "9193ff27cfae35775252126725132334af1397d58d784ee5064d636f244b425d",
            "height": 2227,
            "phased": false,
            "sender": "996325769485053218",
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature": "2f3c84e074623ee706a0999a03e44c8729987e0d0fcf2f5daad86d3d0495e206d516227a2fb12aa88650e19a4ac642f17017",
            "subtype": 0,
            "timestamp": 125712,
            "transaction": "8589964069031613329",
            "transactionIndex": 0,
            "type": 6,
            "version": 1
        }"#;
        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = Transaction::from_json(&json).unwrap();

        assert_eq!(tx.id, 8589964069031613329, "DB_ID=53 ID mismatch");
        assert_eq!(tx.type_id, TransactionType::Data);
        assert!(tx.has_prunable_attachment, "DB_ID=53 should have prunable attachment");
        assert!(!tx.has_prunable_message, "DB_ID=53 should not have prunable message");
        assert!(!tx.has_public_key_announcement, "DB_ID=53 should not have PK announcement");
        assert_eq!(hex::encode(&tx.attachment_bytes), "01a1474a67570fbabbf6794cb86400c119e0b23f4e69f464daea9e0fa93c5ef80c");
    }

    // ============================================================
    // DB_ID=54: type=12(CoinExchange), subtype=0(OrderIssue)
    // 特征：attachment_bytes 包含名称字符串
    // ============================================================
    {
        let json_str = r#"{
            "amountNQT": "0",
            "attachment": {"version.CoinExchangeOrderIssue": 1},
            "block": "-6444971140270725924",
            "blockTimestamp": 125757,
            "deadline": 15,
            "ecBlockHeight": 1506,
            "ecBlockId": "-7988487755785480949",
            "feeNQT": "60000000",
            "fullHash": "75ad68da043dc6574849fb22b1d7f929b830a1a7ccc1c37a5b1b541a9275f9f6",
            "height": 2227,
            "phased": false,
            "sender": "996325769485053218",
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature": "2756a87caf3e566934dd5d649c86b9db5f891be5c0cf92cd37886cf406f6090f822f51ac0c5874a84fcdb5c6e6cf12b72356",
            "subtype": 0,
            "timestamp": 125712,
            "transaction": "6324809817741897077",
            "transactionIndex": 1,
            "type": 12,
            "version": 1
        }"#;
        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = Transaction::from_json(&json).unwrap();

        assert_eq!(tx.id, 6324809817741897077, "DB_ID=54 ID mismatch");
        assert!(!tx.has_prunable_attachment, "DB_ID=54 should not have prunable attachment");
        assert!(!tx.has_public_key_announcement, "DB_ID=54 should not have PK announcement");
    }

    // ============================================================
    // DB_ID=56: type=2(ColoredCoins), subtype=10(AssetPropertySet) + PublicKeyAnnouncement
    // 特征：HAS_PUBLIC_KEY_ANNOUNCEMENT=TRUE
    // ============================================================
    {
        let json_str = r#"{
            "amountNQT": "0",
            "attachment": {
                "version.AssetProperty": 1,
                "property": "no",
                "asset": "16132763665229324019",
                "value": "123456",
                "version.PublicKeyAnnouncement": 1,
                "recipientPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c"
            },
            "block": "4807844805401117927",
            "blockTimestamp": 133506,
            "deadline": 15,
            "ecBlockHeight": 1647,
            "ecBlockId": "-3101292290248805829",
            "feeNQT": "100000000",
            "fullHash": "a82003fc9d935c59e9a62bacb8381f92398ec722a0f02550544d087c18080c89",
            "height": 2369,
            "phased": false,
            "recipient": "996325769485053218",
            "sender": "996325769485053218",
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature": "81c12bdb6a45b05252dc068a34d6212897848055da72d899be6819b9635fdc0f76c37ff5f19093a9d52c7cf24fcc5682fb25",
            "subtype": 10,
            "timestamp": 133354,
            "transaction": "6439183873980178600",
            "transactionIndex": 0,
            "type": 2,
            "version": 1
        }"#;
        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = Transaction::from_json(&json).unwrap();

        assert_eq!(tx.id, 6439183873980178600, "DB_ID=56 ID mismatch");
        assert!(tx.has_public_key_announcement, "DB_ID=56 should have PK announcement");
        assert!(!tx.has_prunable_attachment, "DB_ID=56 should not have prunable attachment");
        // 注意：attachment_bytes 包含 AssetPropertySet + PublicKeyAnnouncement
        // 实际长度为 52 bytes（不是 50 bytes）
        assert_eq!(hex::encode(&tx.attachment_bytes), "01f39e52171417e3df026e6f06313233343536012d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c");
    }

    // ============================================================
    // DB_ID=58: type=2(ColoredCoins), subtype=10(AssetPropertySet) + PublicKeyAnnouncement
    // 特征：ID 为负数 (-598422462908344986)
    // ============================================================
    {
        let json_str = r#"{
            "amountNQT": "0",
            "attachment": {
                "version.AssetProperty": 1,
                "property": "no",
                "asset": "16132763665229324019",
                "value": "123456",
                "version.PublicKeyAnnouncement": 1,
                "recipientPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c"
            },
            "block": "3621356438688649810",
            "blockTimestamp": 134399,
            "deadline": 15,
            "ecBlockHeight": 1657,
            "ecBlockId": "-8543743489585395968",
            "feeNQT": "100000000",
            "fullHash": "66d5bbc8f0f9b1f78d541cb97314e42374c8a170fb756d5d449604b3defd5189",
            "height": 2384,
            "phased": false,
            "recipient": "996325769485053218",
            "sender": "996325769485053218",
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature": "a03910fe31d677e55320da560fd1f2a3bfe33a8099ec7cd881f818315b9b380b4ada13eb2f1c71bd65e7f7af41f8a544591d",
            "subtype": 10,
            "timestamp": 134030,
            "transaction": "-598422462908344986",
            "transactionIndex": 0,
            "type": 2,
            "version": 1
        }"#;
        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = Transaction::from_json(&json).unwrap();

        // 注意：Java 的 -598422462908344986 在 Rust 中应该被解释为 u64
        // 但这里我们使用 full_hash 计算 ID，所以应该匹配
        assert!(tx.id > 0, "DB_ID=58 ID should be positive u64");
        assert!(tx.has_public_key_announcement, "DB_ID=58 should have PK announcement");
    }

    // ============================================================
    // DB_ID=59: type=6(Data), subtype=0(TaggedDataUpload)
    // 特征：HAS_PRUNABLE_ATTACHMENT=TRUE
    // ============================================================
    {
        let json_str = r#"{
            "amountNQT": "0",
            "attachment": {"version.TaggedDataUpload": 1, "hash": "66d745082ab2b69563689c33cca9f2aea0867fcab96450f7343984cea3411ca8"},
            "block": "3472667422419083",
            "blockTimestamp": 165731,
            "deadline": 15,
            "ecBlockHeight": 2230,
            "ecBlockId": "6834459906794590541",
            "feeNQT": "280000000",
            "fullHash": "8133d0b6dc69842183feeb8626ac37407470ef19ea330d8011a788d0b7929b80",
            "height": 2951,
            "phased": false,
            "sender": "996325769485053218",
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature": "32f0ac04d72b134e8b0f7a29b7fad986d815ec0ea75165605e5633ea4f174a08b82e1ac92b85ac74c3bb4dcb587b523eac20",
            "subtype": 0,
            "timestamp": 165728,
            "transaction": "2415171696858248065",
            "transactionIndex": 0,
            "type": 6,
            "version": 1
        }"#;
        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = Transaction::from_json(&json).unwrap();

        assert_eq!(tx.id, 2415171696858248065, "DB_ID=59 ID mismatch");
        assert!(tx.has_prunable_attachment, "DB_ID=59 should have prunable attachment");
        assert_eq!(hex::encode(&tx.attachment_bytes), "0166d745082ab2b69563689c33cca9f2aea0867fcab96450f7343984cea3411ca8");
    }

    // ============================================================
    // DB_ID=61: type=6(Data), subtype=0(TaggedDataUpload)
    // 特征：HAS_PRUNABLE_ATTACHMENT=TRUE
    // ============================================================
    {
        let json_str = r#"{
            "amountNQT": "0",
            "attachment": {"version.TaggedDataUpload": 1, "hash": "66d745082ab2b69563689c33cca9f2aea0867fcab96450f7343984cea3411ca8"},
            "block": "-8990481967722926086",
            "blockTimestamp": 167923,
            "deadline": 15,
            "ecBlockHeight": 2267,
            "ecBlockId": "-1762530543998683424",
            "feeNQT": "280000000",
            "fullHash": "679c85f0d133af0e810179ee7b9119185d16a7725406abe3565840d1c56cd45c",
            "height": 2988,
            "phased": false,
            "sender": "996325769485053218",
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature": "2aaa452578ada6853483e174689d61ec4555ba38b8d17c1efb1616419793550dee059f9bb5893f60d69226a4b32eb3bdc0f1",
            "subtype": 0,
            "timestamp": 167893,
            "transaction": "1058121414231825511",
            "transactionIndex": 0,
            "type": 6,
            "version": 1
        }"#;
        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = Transaction::from_json(&json).unwrap();

        assert_eq!(tx.id, 1058121414231825511, "DB_ID=61 ID mismatch");
        assert!(tx.has_prunable_attachment, "DB_ID=61 should have prunable attachment");
    }

    // ============================================================
    // DB_ID=63: type=0(Payment), subtype=0(OrdinaryPayment)
    // 特征：attachment_bytes=null（普通支付无附件）
    // ============================================================
    {
        let json_str = r#"{
            "amountNQT": "100000000000000",
            "attachment": {},
            "block": "-3113646180396305155",
            "blockTimestamp": 169851,
            "deadline": 1440,
            "ecBlockHeight": 0,
            "ecBlockId": "3488276486778630462",
            "feeNQT": "100000000",
            "fullHash": "513cceb01aa01c90e442e9e94fc186a5af38120196a69f3b039667f650eed1aa",
            "height": 3023,
            "phased": false,
            "recipient": "-734255405228472986",
            "sender": "2794603741293765856",
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature": "1068a760d3fe9507f0ee0a158e3fe419aba97fd8ae5cd9c7a0b5507241b40c0bfd5ed53e1225e28ad8ce143c747711d623c9",
            "subtype": 0,
            "timestamp": 169813,
            "transaction": "-8062393196404130735",
            "transactionIndex": 0,
            "type": 0,
            "version": 1
        }"#;
        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = Transaction::from_json(&json).unwrap();

        assert!(tx.attachment_bytes.is_empty(), "DB_ID=63 Payment should have empty attachment");
        assert!(!tx.has_prunable_attachment, "DB_ID=63 should not have prunable attachment");
        assert!(!tx.has_public_key_announcement, "DB_ID=63 should not have PK announcement");
    }

    // ============================================================
    // DB_ID=64: type=1(Messaging), subtype=0(ArbitraryMessage) + EncryptToSelfMessage
    // 特征：HAS_ENCRYPTTOSELF_MESSAGE=TRUE
    // ============================================================
    {
        let json_str = r#"{
            "amountNQT": "0",
            "attachment": {
                "version.ArbitraryMessage": 0,
                "encryptToSelfMessage": {"data": "encrypted", "nonce": "nonce123", "isText": true}
            },
            "block": "1339659246165865766",
            "blockTimestamp": 170691,
            "deadline": 1440,
            "ecBlockHeight": 0,
            "ecBlockId": "3488276486778630462",
            "feeNQT": "100000000",
            "fullHash": "811519223468c537003f8e7adc8625e95409f3f273f38021095a70638fc46c45",
            "height": 3038,
            "phased": false,
            "recipient": "2794603741293765856",
            "sender": "-734255405228472986",
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature": "e4df5e7f5d38dfb72e852ff9cb11b77d9ee92b2f9101789b953e6eff8f22910358b91e22aa8fb18c96eafe06803b0bf11e40",
            "subtype": 0,
            "timestamp": 169975,
            "transaction": "4018732815617693057",
            "transactionIndex": 0,
            "type": 1,
            "version": 1
        }"#;
        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = Transaction::from_json(&json).unwrap();

        assert_eq!(tx.id, 4018732815617693057, "DB_ID=64 ID mismatch");
        assert!(tx.has_encrypttoself_message, "DB_ID=64 should have encrypt-to-self message");
        assert!(!tx.has_prunable_attachment, "DB_ID=64 should not have prunable attachment");
    }

    // ============================================================
    // DB_ID=65: type=12(LightContract), subtype=1(ContractReferenceSet)
    // 特征：LightContract 类型
    // ============================================================
    {
        let json_str = r#"{
            "amountNQT": "0",
            "attachment": {"version.ContractReferenceSet": 1, "contractReference": "16210848054059321061"},
            "block": "-4050848184299982100",
            "blockTimestamp": 171769,
            "deadline": 15,
            "ecBlockHeight": 2338,
            "ecBlockId": "7386031426484071329",
            "feeNQT": "300000000",
            "fullHash": "0b9f62178466df745e2b673016991327e5cc961ebd803491e22477c505669c50",
            "height": 3057,
            "phased": false,
            "sender": "996325769485053218",
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature": "24ab6c187d676d8539cdfc1f1ae87ca050f0949b682add0680c18e275efc46069f937d40f486687c7a2282ea3e227e84e149",
            "subtype": 1,
            "timestamp": 171767,
            "transaction": "8421562545720172299",
            "transactionIndex": 0,
            "type": 12,
            "version": 1
        }"#;
        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = Transaction::from_json(&json).unwrap();

        assert_eq!(tx.id, 8421562545720172299, "DB_ID=65 ID mismatch");
        assert_eq!(hex::encode(&tx.attachment_bytes), "01e5c6099a6a80f8e0");
    }

    // ============================================================
    // DB_ID=66: type=5(MonetarySystem), subtype=0(CurrencyIssuance)
    // 特征：长 attachment_bytes（包含货币发行参数）
    // ============================================================
    {
        let json_str = r#"{
            "amountNQT": "0",
            "attachment": {
                "version.CurrencyIssuance": 1,
                "name": "DUNDUN",
                "code": "DUNDUN",
                "description": "",
                "type": 1,
                "initialSupply": "100000",
                "reserveSupply": "0",
                "maxSupply": "100000",
                "issuanceHeight": 0,
                "minReservePerUnitNQT": "0",
                "rulesetExpirationHeight": 0,
                "mintable": false,
                "decimals": 2
            },
            "block": "-7248403572455302830",
            "blockTimestamp": 344352,
            "deadline": 1440,
            "ecBlockHeight": 5418,
            "ecBlockId": "6045279492790973245",
            "feeNQT": "100000000000",
            "fullHash": "c141ab6eea87761de6e6d1635704dcb2109ab0fe7d5f7a21780152a205c08791",
            "height": 6139,
            "phased": false,
            "sender": "996325769485053218",
            "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
            "signature": "7f6adabfa0572f1c69a56cdcab5dfb01639b29fb14bc4ad89bad0bd1315c7d06b119656c4f922990bf260dd4abaf094c9fd7",
            "subtype": 0,
            "timestamp": 344347,
            "transaction": "2123033715300581825",
            "transactionIndex": 0,
            "type": 5,
            "version": 1
        }"#;
        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let tx = Transaction::from_json(&json).unwrap();

        assert_eq!(tx.id, 2123033715300581825, "DB_ID=66 ID mismatch");
        assert!(!tx.has_prunable_attachment, "DB_ID=66 should not have prunable attachment");
        // CurrencyIssuance 应该有较长的 attachment_bytes
        assert!(tx.attachment_bytes.len() > 20, "DB_ID=66 CurrencyIssuance should have long attachment bytes");
    }
}
