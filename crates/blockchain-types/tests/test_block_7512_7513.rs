use blockchain_types::block::Block;
use blockchain_types::transaction::Transaction;

fn build_block_7512_p2p() -> Block {
    let p2p_json_str = r#"{
        "version": 3,
        "timestamp": 422072,
        "previousBlock": "18089229018648297210",
        "totalAmountNQT": 0,
        "totalFeeNQT": 0,
        "payloadLength": 0,
        "payloadHash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "generatorPublicKey": "21a908b060ca909e4f4f665aa31243f51817e9cbd32ddbb751fcef2373aeba1f",
        "generationSignature": "5fbe82f78e031736ee92478d63d5d9677ec0cb6a2039904c206f1c2a6fe4ec73",
        "previousBlockHash": "fa3e389df3d909fb817475df35871e4440060e51e86778c755ae0dc9e9eea2fa",
        "blockSignature": "bb05ae881a4a4c80d352408c0a3e99bcb2771c36947592ee64c727ad0b06d00acb203a6c6ebb72c22536599c7ab9e940c0b41f9d4ba708bb8dcf5e8eba416f78",
        "transactions": []
    }"#;

    let block_json: serde_json::Value = serde_json::from_str(p2p_json_str).unwrap();
    let mut block_json_no_tx = block_json.clone();
    if let Some(obj) = block_json_no_tx.as_object_mut() {
        obj.remove("transactions");
    }
    let mut block: Block = serde_json::from_value(block_json_no_tx).unwrap();
    block.height = 7512;
    block.generator_id = Some(blockchain_types::block::account_id_from_public_key(
        block.generator_public_key.as_ref().unwrap()
    ));
    block.transactions = vec![];
    block.id = Some(block.calculate_id().unwrap_or(0));
    block
}

fn build_block_7513_p2p() -> (Block, Transaction) {
    let p2p_json_str = r#"{
        "version": 3,
        "timestamp": 422108,
        "previousBlock": "13152218554931764778",
        "totalAmountNQT": 0,
        "totalFeeNQT": 100000000,
        "payloadLength": 193,
        "payloadHash": "b475d5e88607cc1448cef4ea1c2168f8436f882d07a6487293c24e82038ca906",
        "generatorPublicKey": "21a908b060ca909e4f4f665aa31243f51817e9cbd32ddbb751fcef2373aeba1f",
        "generationSignature": "355e0a21bafa3c5477d6adba13188221d782d473f222dbffa0624f6b9e58573f",
        "previousBlockHash": "2a5e029f171186b6ed9ea879efb0a364b50a7ce3f209b3df880082978df77684",
        "blockSignature": "f7b00f99fc507667e06c061c6403f00664e7b1cdae2231ecc15ad7fc81996a021b0ad2acb4ea654839a31446ee4f143212c9323c5a4adbf118720abc4f2b43b2",
        "transactions": [
            {
                "type": 5,
                "subtype": 3,
                "timestamp": 422102,
                "deadline": 1440,
                "senderPublicKey": "2d37b522ee336ee1f97b6f2365dcecd10a128ea916b91e30ff4313553a931b2c",
                "recipient": "899669674531378169",
                "amountNQT": 0,
                "feeNQT": 100000000,
                "ecBlockHeight": 6792,
                "ecBlockId": "3324756215706311257",
                "signature": "6ebd8910ce0d92219a12ccfa413e44dce515b985d3427e64174a8d3c4aed910d651f079a81c505f14bb71262d069c23120e49e5c5b8228c29960fc0b856be5d2",
                "attachment": {
                    "version.CurrencyTransfer": 1,
                    "currency": "2123033715300581825",
                    "units": 500
                },
                "version": 1
            }
        ]
    }"#;

    let block_json: serde_json::Value = serde_json::from_str(p2p_json_str).unwrap();
    let transactions_json = block_json.get("transactions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut block_json_no_tx = block_json.clone();
    if let Some(obj) = block_json_no_tx.as_object_mut() {
        obj.remove("transactions");
    }
    let mut block: Block = serde_json::from_value(block_json_no_tx).unwrap();
    block.height = 7513;
    block.generator_id = Some(blockchain_types::block::account_id_from_public_key(
        block.generator_public_key.as_ref().unwrap()
    ));

    let mut tx = Transaction::from_json(&transactions_json[0]).unwrap();
    tx.block_timestamp = block.timestamp;
    block.transactions = vec![tx.clone()];
    block.id = Some(block.calculate_id().unwrap_or(0));

    (block, tx)
}

#[test]
fn test_block_7512_p2p_full_analysis() {
    let block = build_block_7512_p2p();

    println!("=== Block 7512 Full Analysis ===");
    println!("  id = {}", block.get_id());
    println!("  expected id = 13152218554931764778");
    println!("  height = {}", block.height);
    println!("  version = {}", block.version);
    println!("  timestamp = {}", block.timestamp);
    println!("  generator_id = {}", block.get_generator_id());
    println!("  expected generator_id = 2794603741293765856");
    println!("  base_target = {}", block.base_target);
    println!("  previous_block_id = {:?}", block.previous_block_id);
    println!("  total_amount = {}", block.total_amount);
    println!("  total_fee = {}", block.total_fee);
    println!("  payload_length = {}", block.payload_length);
    println!("  payload_hash = {}", hex::encode(block.payload_hash.0));
    println!("  generation_signature = {}", hex::encode(&block.generation_signature));
    println!("  previous_block_hash = {}", hex::encode(block.previous_block_hash.0));
    println!("  block_signature = {}", hex::encode(block.block_signature.0));

    let sig_result = block.verify_block_signature();
    println!("\n  verify_block_signature = {:?}", sig_result);
    if let Ok(false) = sig_result {
        println!("  ❌ BLOCK SIGNATURE VERIFICATION FAILED!");
        let data = block.serialize_for_signing();
        println!("  serialize_for_signing length = {}", data.len());
        println!("  serialize_for_signing hex = {}", hex::encode(&data));
    }

    let gen_sig_result = block.verify_generation_signature(&[0u8; 32]);
    println!("  verify_generation_signature (with dummy prev) = {:?}", gen_sig_result);

    let expected_id: u64 = 13152218554931764778;
    let calculated_id = block.get_id();
    if calculated_id != expected_id {
        println!("  ❌ BLOCK ID MISMATCH: calculated={}, expected={}", calculated_id, expected_id);
    } else {
        println!("  ✅ Block ID matches: {}", calculated_id);
    }

    let expected_generator_id: u64 = 2794603741293765856;
    let calculated_generator_id = block.get_generator_id();
    if calculated_generator_id != expected_generator_id {
        println!("  ❌ GENERATOR ID MISMATCH: calculated={}, expected={}", calculated_generator_id, expected_generator_id);
    } else {
        println!("  ✅ Generator ID matches: {}", calculated_generator_id);
    }
}

#[test]
fn test_block_7513_p2p_full_analysis() {
    let (block, tx) = build_block_7513_p2p();

    println!("=== Block 7513 Full Analysis ===");
    println!("  id = {}", block.get_id());
    println!("  expected id = 1032650923450679770");
    println!("  height = {}", block.height);
    println!("  version = {}", block.version);
    println!("  timestamp = {}", block.timestamp);
    println!("  generator_id = {}", block.get_generator_id());
    println!("  expected generator_id = 2794603741293765856");
    println!("  base_target = {}", block.base_target);
    println!("  previous_block_id = {:?}", block.previous_block_id);
    println!("  total_amount = {}", block.total_amount);
    println!("  total_fee = {}", block.total_fee);
    println!("  payload_length = {}", block.payload_length);
    println!("  payload_hash = {}", hex::encode(block.payload_hash.0));
    println!("  generation_signature = {}", hex::encode(&block.generation_signature));
    println!("  previous_block_hash = {}", hex::encode(block.previous_block_hash.0));
    println!("  block_signature = {}", hex::encode(block.block_signature.0));

    println!("\n  Transaction Analysis:");
    println!("    id = {}", tx.id);
    println!("    expected id = 4226461586804269089");
    println!("    type={:?}, subtype={}", tx.type_id, tx.subtype);
    println!("    sender_id = {}", tx.sender_id);
    println!("    recipient_id = {:?}", tx.recipient_id);
    println!("    amount = {}", tx.amount);
    println!("    fee = {}", tx.fee);
    println!("    attachment_bytes = {}", hex::encode(&tx.attachment_bytes));
    println!("    full_hash = {}", hex::encode(tx.full_hash.0));
    println!("    signature verified = {}", tx.verify_signature());

    let sig_result = block.verify_block_signature();
    println!("\n  verify_block_signature = {:?}", sig_result);
    if let Ok(false) = sig_result {
        println!("  ❌ BLOCK SIGNATURE VERIFICATION FAILED!");
        let data = block.serialize_for_signing();
        println!("  serialize_for_signing length = {}", data.len());
        println!("  serialize_for_signing hex = {}", hex::encode(&data));
    } else {
        println!("  ✅ Block signature verified");
    }

    let gen_sig_result = block.verify_generation_signature(&[0u8; 32]);
    println!("  verify_generation_signature (with dummy prev) = {:?}", gen_sig_result);

    let expected_id: u64 = 1032650923450679770;
    let calculated_id = block.get_id();
    if calculated_id != expected_id {
        println!("  ❌ BLOCK ID MISMATCH: calculated={}, expected={}", calculated_id, expected_id);
    } else {
        println!("  ✅ Block ID matches: {}", calculated_id);
    }

    let expected_generator_id: u64 = 2794603741293765856;
    let calculated_generator_id = block.get_generator_id();
    if calculated_generator_id != expected_generator_id {
        println!("  ❌ GENERATOR ID MISMATCH: calculated={}, expected={}", calculated_generator_id, expected_generator_id);
    } else {
        println!("  ✅ Generator ID matches: {}", calculated_generator_id);
    }

    let computed_payload_hash = Block::compute_payload_hash_from_bytes(&block.transactions).unwrap();
    if computed_payload_hash != block.payload_hash {
        println!("  ❌ PAYLOAD HASH MISMATCH: computed={}, block={}",
                 hex::encode(computed_payload_hash.0), hex::encode(block.payload_hash.0));
    } else {
        println!("  ✅ Payload hash matches");
    }

    let expected_tx_id: u64 = 4226461586804269089;
    if tx.id != expected_tx_id {
        println!("  ❌ TX ID MISMATCH: calculated={}, expected={}", tx.id, expected_tx_id);
    } else {
        println!("  ✅ Transaction ID matches");
    }
}

#[test]
fn test_block_7512_api_format_deserialization() {
    let api_json_str = r#"{
        "baseTarget": "198221970",
        "block": "13152218554931764778",
        "blockSignature": "bb05ae881a4a4c80d352408c0a3e99bcb2771c36947592ee64c727ad0b06d00acb203a6c6ebb72c22536599c7ab9e940c0b41f9d4ba708bb8dcf5e8eba416f78",
        "cumulativeDifficulty": "767865397194630",
        "errorCode": 0,
        "generationSignature": "5fbe82f78e031736ee92478d63d5d9677ec0cb6a2039904c206f1c2a6fe4ec73",
        "generator": "2794603741293765856",
        "generatorPublicKey": "21a908b060ca909e4f4f665aa31243f51817e9cbd32ddbb751fcef2373aeba1f",
        "height": 7512,
        "nextBlock": "1032650923450679770",
        "numberOfTransactions": 0,
        "payloadHash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "payloadLength": 0,
        "previousBlock": "18089229018648297210",
        "previousBlockHash": "fa3e389df3d909fb817475df35871e4440060e51e86778c755ae0dc9e9eea2fa",
        "timestamp": 422072,
        "totalAmountNQT": "0",
        "totalFeeNQT": "0",
        "transactions": [],
        "version": 3
    }"#;

    let block_json: serde_json::Value = serde_json::from_str(api_json_str).unwrap();
    let mut block_json_no_tx = block_json.clone();
    if let Some(obj) = block_json_no_tx.as_object_mut() {
        obj.remove("transactions");
    }

    let block: std::result::Result<Block, serde_json::Error> = serde_json::from_value(block_json_no_tx);
    match &block {
        Ok(b) => {
            println!("✅ Block 7512 (API format) deserialized successfully");
            println!("  id={:?}", b.id);
            println!("  height={}", b.height);
            println!("  generator_id={:?}", b.generator_id);
            println!("  base_target={}", b.base_target);
        }
        Err(e) => {
            println!("❌ Block 7512 (API format) deserialization FAILED: {}", e);
        }
    }
}

#[test]
fn test_block_7513_api_format_deserialization() {
    let api_json_str = r#"{
        "baseTarget": "183421397",
        "block": "1032650923450679770",
        "blockSignature": "f7b00f99fc507667e06c061c6403f00664e7b1cdae2231ecc15ad7fc81996a021b0ad2acb4ea654839a31446ee4f143212c9323c5a4adbf118720abc4f2b43b2",
        "cumulativeDifficulty": "767965967490001",
        "errorCode": 0,
        "generationSignature": "355e0a21bafa3c5477d6adba13188221d782d473f222dbffa0624f6b9e58573f",
        "generator": "2794603741293765856",
        "generatorPublicKey": "21a908b060ca909e4f4f665aa31243f51817e9cbd32ddbb751fcef2373aeba1f",
        "height": 7513,
        "nextBlock": "17569309662638758236",
        "numberOfTransactions": 1,
        "payloadHash": "b475d5e88607cc1448cef4ea1c2168f8436f882d07a6487293c24e82038ca906",
        "payloadLength": 193,
        "previousBlock": "13152218554931764778",
        "previousBlockHash": "2a5e029f171186b6ed9ea879efb0a364b50a7ce3f209b3df880082978df77684",
        "timestamp": 422108,
        "totalAmountNQT": "0",
        "totalFeeNQT": "100000000",
        "transactions": [
            {
                "amountNQT": "0",
                "attachment": {
                    "version.CurrencyTransfer": 1,
                    "currency": "2123033715300581825",
                    "units": "500"
                },
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
            }
        ],
        "version": 3
    }"#;

    let block_json: serde_json::Value = serde_json::from_str(api_json_str).unwrap();
    let mut block_json_no_tx = block_json.clone();
    if let Some(obj) = block_json_no_tx.as_object_mut() {
        obj.remove("transactions");
    }

    let block: std::result::Result<Block, serde_json::Error> = serde_json::from_value(block_json_no_tx);
    match &block {
        Ok(b) => {
            println!("✅ Block 7513 (API format) deserialized successfully");
            println!("  id={:?}", b.id);
            println!("  height={}", b.height);
            println!("  generator_id={:?}", b.generator_id);
            println!("  base_target={}", b.base_target);
        }
        Err(e) => {
            println!("❌ Block 7513 (API format) deserialization FAILED: {}", e);
        }
    }
}
