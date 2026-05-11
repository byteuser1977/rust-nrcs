use blockchain_types::{
    Hash256, Signature, TransactionType,
    transaction::Transaction,
};
use crypto::{passphrase_to_keypair, derive_public_key, sha256};
use serde_json::json;

use crate::error::ApiError;
use crate::request_handler::{ApiRequest, RsRespBuilder, RsRespWithData};
use crate::state::ApiState;
use crate::parameter_parser::ParameterParser;

pub struct CommonTransactionParams {
    pub secret_phrase: Option<String>,
    pub public_key: Option<String>,
    pub fee_nqt: u64,
    pub deadline: u16,
    pub broadcast: bool,
    pub referenced_transaction_full_hash: Option<String>,

    // 附件参数（对照 Java: CreateTransactionCallBuilder）
    pub message: Option<String>,
    pub message_is_text: Option<bool>,
    pub encrypted_message_data: Option<String>,
    pub encrypted_message_nonce: Option<String>,
    pub encrypt_to_self_message_data: Option<String>,
    pub encrypt_to_self_message_nonce: Option<String>,
    pub recipient_public_key_announcement: Option<String>,

    // Phasing 参数（对照 Java: AppendixPhasing）
    pub phased: bool,
    pub phasing_params: Option<serde_json::Value>,

    // EC 区块参数（对照 Java: ecBlockId/ecBlockHeight）
    pub ec_block_id: Option<u64>,
    pub ec_block_height: Option<u32>,
    pub timestamp: Option<u32>,
}

pub struct CreateTransactionHelper;

impl CreateTransactionHelper {
    pub fn parse_common_params(req: &ApiRequest) -> std::result::Result<CommonTransactionParams, ApiError> {
        let secret_phrase = req.get_string("secretPhrase");
        let public_key = req.get_string("publicKey");

        if secret_phrase.is_none() && public_key.is_none() {
            return Err(ApiError::MissingParameter("secretPhrase or publicKey".to_string()));
        }

        let fee_nqt = req.get_string("feeNQT")
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| ApiError::MissingParameter("feeNQT".to_string()))?;

        let deadline = req.get_string("deadline")
            .and_then(|s| s.parse::<u16>().ok())
            .ok_or_else(|| ApiError::MissingParameter("deadline".to_string()))?;

        if deadline == 0 {
            return Err(ApiError::IncorrectValue("deadline must be >= 1".to_string()));
        }

        let broadcast = if secret_phrase.is_some() {
            !req.get_string("broadcast").map(|v| v.eq_ignore_ascii_case("false")).unwrap_or(false)
        } else {
            false
        };

        let referenced_transaction_full_hash = req.get_string("referencedTransactionFullHash");

        let message = req.get_string("message");
        let message_is_text = req.get_string("messageIsText")
            .map(|v| v.eq_ignore_ascii_case("true"));
        let encrypted_message_data = req.get_string("encryptedMessageData");
        let encrypted_message_nonce = req.get_string("encryptedMessageNonce");
        let encrypt_to_self_message_data = req.get_string("encryptToSelfMessageData");
        let encrypt_to_self_message_nonce = req.get_string("encryptToSelfMessageNonce");
        let recipient_public_key_announcement = req.get_string("recipientPublicKey");

        let phased = req.get_string("phased")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let phasing_params = ParameterParser::parse_phasing_params(req, None)?;

        let ec_block_height = req.get_string("ecBlockHeight")
            .and_then(|s| s.parse::<u32>().ok());
        let ec_block_id = req.get_string("ecBlockId")
            .and_then(|s| s.parse::<u64>().ok());
        let timestamp = req.get_string("timestamp")
            .and_then(|s| s.parse::<u32>().ok());

        Ok(CommonTransactionParams {
            secret_phrase,
            public_key,
            fee_nqt,
            deadline,
            broadcast,
            referenced_transaction_full_hash,
            message,
            message_is_text,
            encrypted_message_data,
            encrypted_message_nonce,
            encrypt_to_self_message_data,
            encrypt_to_self_message_nonce,
            recipient_public_key_announcement,
            phased,
            phasing_params,
            ec_block_id,
            ec_block_height,
            timestamp,
        })
    }

    pub async fn create_and_broadcast_transaction(
        params: &CommonTransactionParams,
        transaction_type: u8,
        subtype: u8,
        recipient_id: Option<u64>,
        amount_nqt: u64,
        attachment_json: Option<serde_json::Value>,
        state: &ApiState,
    ) -> std::result::Result<RsRespWithData, ApiError> {
        let public_key_bytes = if let Some(ref sp) = params.secret_phrase {
            derive_public_key(sp)
                .map_err(|e| ApiError::Internal(format!("failed to derive public key: {}", e)))?
        } else if let Some(ref pk_hex) = params.public_key {
            hex::decode(pk_hex)
                .map_err(|e| ApiError::IncorrectValue(format!("invalid publicKey hex: {}", e)))?
        } else {
            return Err(ApiError::MissingParameter("secretPhrase or publicKey".to_string()));
        };

        let mut pk_arr = [0u8; 32];
        if public_key_bytes.len() == 32 {
            pk_arr.copy_from_slice(&public_key_bytes);
        }

        let sender_id = Transaction::public_key_to_account_id(&pk_arr);

        let sender_account = state.account_manager
            .get_account_info(sender_id)
            .await
            .map_err(ApiError::Account)?;

        let type_id = TransactionType::from_type(transaction_type);

        let timestamp = params.timestamp.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as u32
        });

        let mut tx = Transaction::new(
            type_id,
            sender_id,
            recipient_id,
            amount_nqt,
            params.fee_nqt,
            timestamp,
            params.deadline,
        );
        tx.subtype = subtype;
        tx.sender_public_key = Hash256(pk_arr);

        if let Some(ref hash_hex) = params.referenced_transaction_full_hash {
            let hash_bytes = hex::decode(hash_hex).unwrap_or_default();
            if hash_bytes.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&hash_bytes);
                tx.referenced_transaction_full_hash = Some(Hash256(arr));
            }
        }

        if let Some(eb_height) = params.ec_block_height {
            tx.ec_block_height = Some(eb_height);
        }
        if let Some(eb_id) = params.ec_block_id {
            tx.ec_block_id = Some(eb_id);
        }

        if params.phased {
            tx.phased = true;
        }

        if params.message.is_some() {
            tx.has_message = true;
        }
        if params.encrypted_message_data.is_some() {
            tx.has_encrypted_message = true;
        }
        if params.encrypt_to_self_message_data.is_some() {
            tx.has_encrypttoself_message = true;
        }
        if params.recipient_public_key_announcement.is_some() {
            tx.has_public_key_announcement = true;
        }

        if let Some(ref att) = attachment_json {
            tx.attachment_json = att.as_object().cloned();
        }

        if let Some(ref phasing) = params.phasing_params {
            if tx.attachment_json.is_none() {
                tx.attachment_json = Some(serde_json::Map::new());
            }
            if let Some(ref mut att) = tx.attachment_json {
                att.insert("phasing".to_string(), phasing.clone());
            }
        }

        if params.message.is_some() || params.encrypted_message_data.is_some() {
            if tx.attachment_json.is_none() {
                tx.attachment_json = Some(serde_json::Map::new());
            }
            if let Some(ref mut att) = tx.attachment_json {
                if let Some(ref msg) = params.message {
                    let mut message_obj = serde_json::Map::new();
                    message_obj.insert("message".to_string(), json!(msg));
                    if let Some(is_text) = params.message_is_text {
                        message_obj.insert("messageIsText".to_string(), json!(is_text));
                    }
                    att.insert("message".to_string(), json!(message_obj));
                }
                if let Some(ref data) = params.encrypted_message_data {
                    let mut enc_msg_obj = serde_json::Map::new();
                    enc_msg_obj.insert("data".to_string(), json!(data));
                    if let Some(ref nonce) = params.encrypted_message_nonce {
                        enc_msg_obj.insert("nonce".to_string(), json!(nonce));
                    }
                    att.insert("encryptedMessage".to_string(), json!(enc_msg_obj));
                }
                if let Some(ref data) = params.encrypt_to_self_message_data {
                    let mut enc_self_obj = serde_json::Map::new();
                    enc_self_obj.insert("data".to_string(), json!(data));
                    if let Some(ref nonce) = params.encrypt_to_self_message_nonce {
                        enc_self_obj.insert("nonce".to_string(), json!(nonce));
                    }
                    att.insert("encryptToSelfMessage".to_string(), json!(enc_self_obj));
                }
                if let Some(ref pk_hex) = params.recipient_public_key_announcement {
                    att.insert("recipientPublicKey".to_string(), json!(pk_hex));
                }
            }
        }

        let mut builder = RsRespBuilder::new();

        if let Some(ref sp) = params.secret_phrase {
            let kp = passphrase_to_keypair(sp)
                .map_err(|e| ApiError::Internal(format!("failed to create keypair: {}", e)))?;

            let message = tx.serialize_for_signing();
            let sig_bytes = kp.sign(&message);
            tx.signature = Signature(sig_bytes);

            tx.full_hash = tx.calculate_full_hash()
                .unwrap_or(Hash256([0u8; 32]));
            tx.id = tx.calculate_id();

            let total_cost = amount_nqt.saturating_add(params.fee_nqt);
            if total_cost > sender_account.unconfirmed_balance {
                return Err(ApiError::Validation("Not enough funds".to_string()));
            }

            let tx_json = transaction_to_json(&tx);
            builder.insert("transactionJSON", tx_json);

            let unsigned_bytes = tx.serialize_for_signing();
            builder.insert("unsignedTransactionBytes", hex::encode(&unsigned_bytes));

            builder
                .insert("transaction", tx.id.to_string())
                .insert("fullHash", hex::encode(tx.full_hash.0))
                .insert("transactionBytes", hex::encode(tx.get_bytes()));

            let sig_hash = sha256(&tx.signature.0);
            builder.insert("signatureHash", hex::encode(sig_hash));

            if params.broadcast {
                state.tx_processor
                    .broadcast(&tx)
                    .await
                    .map_err(ApiError::TxEngine)?;
                builder.insert("broadcasted", true);
            } else {
                state.tx_processor
                    .validate(&tx)
                    .await
                    .map_err(ApiError::TxEngine)?;
                builder.insert("broadcasted", false);
            }
        } else {
            tx.full_hash = tx.calculate_full_hash()
                .unwrap_or(Hash256([0u8; 32]));
            tx.id = tx.calculate_id();

            let tx_json = transaction_to_json(&tx);
            builder.insert("transactionJSON", tx_json);

            let unsigned_bytes = tx.serialize_for_signing();
            builder.insert("unsignedTransactionBytes", hex::encode(&unsigned_bytes));
        }

        Ok(builder.build())
    }

    pub fn parse_transaction_from_bytes_or_json(
        req: &ApiRequest,
    ) -> std::result::Result<Transaction, ApiError> {
        let tx_bytes_hex = req.get_string("transactionBytes");
        let tx_json_str = req.get_string("transactionJSON");

        if let Some(ref hex_str) = tx_bytes_hex {
            let bytes = hex::decode(hex_str)
                .map_err(|e| ApiError::IncorrectValue(format!("invalid transactionBytes hex: {}", e)))?;
            return Self::parse_transaction_from_bytes(&bytes);
        }

        if let Some(ref json_str) = tx_json_str {
            let json_value: serde_json::Value = serde_json::from_str(json_str)
                .map_err(|e| ApiError::IncorrectValue(format!("invalid transactionJSON: {}", e)))?;
            return Transaction::from_json(&json_value)
                .map_err(ApiError::Blockchain);
        }

        Err(ApiError::MissingParameter("transactionBytes or transactionJSON".to_string()))
    }

    pub fn parse_transaction_from_bytes(bytes: &[u8]) -> std::result::Result<Transaction, ApiError> {
        if bytes.len() < 96 {
            return Err(ApiError::IncorrectValue("transactionBytes too short".to_string()));
        }

        let type_byte = bytes[0];
        let subtype_and_version = bytes[1];
        let subtype = subtype_and_version & 0x0F;
        let version = (subtype_and_version >> 4) & 0x0F;

        let timestamp = i32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
        let deadline = u16::from_le_bytes([bytes[6], bytes[7]]);

        let mut sender_pk = [0u8; 32];
        sender_pk.copy_from_slice(&bytes[8..40]);

        let mut recipient_bytes = [0u8; 8];
        recipient_bytes.copy_from_slice(&bytes[40..48]);
        let recipient_id = u64::from_le_bytes(recipient_bytes);

        let mut amount_bytes = [0u8; 8];
        amount_bytes.copy_from_slice(&bytes[48..56]);
        let amount = u64::from_le_bytes(amount_bytes);

        let mut fee_bytes = [0u8; 8];
        fee_bytes.copy_from_slice(&bytes[56..64]);
        let fee = u64::from_le_bytes(fee_bytes);

        let mut ref_hash = [0u8; 32];
        ref_hash.copy_from_slice(&bytes[64..96]);

        let mut signature = [0u8; 64];
        if bytes.len() >= 160 {
            signature.copy_from_slice(&bytes[96..160]);
        }

        let sender_id = Transaction::public_key_to_account_id(&sender_pk);

        let referenced_transaction_full_hash = if ref_hash == [0u8; 32] {
            None
        } else {
            Some(Hash256(ref_hash))
        };

        let (ec_block_height, ec_block_id, attachment_bytes) = if version > 0 && bytes.len() > 164 {
            let _flags = u32::from_le_bytes([bytes[160], bytes[161], bytes[162], bytes[163]]);
            let eb_height = u32::from_le_bytes([bytes[164], bytes[165], bytes[166], bytes[167]]);
            let mut eb_id_bytes = [0u8; 8];
            if bytes.len() >= 176 {
                eb_id_bytes.copy_from_slice(&bytes[168..176]);
            }
            let eb_id = u64::from_le_bytes(eb_id_bytes);
            let att = bytes[176..].to_vec();
            (Some(eb_height), Some(eb_id), att)
        } else {
            (None, None, vec![])
        };

        let has_message = if version > 0 && bytes.len() > 164 {
            let flags = u32::from_le_bytes([bytes[160], bytes[161], bytes[162], bytes[163]]);
            (flags & 1) != 0
        } else {
            false
        };
        let has_encrypted_message = if version > 0 && bytes.len() > 164 {
            let flags = u32::from_le_bytes([bytes[160], bytes[161], bytes[162], bytes[163]]);
            (flags & 2) != 0
        } else {
            false
        };
        let phased = if version > 0 && bytes.len() > 164 {
            let flags = u32::from_le_bytes([bytes[160], bytes[161], bytes[162], bytes[163]]);
            (flags & 16) != 0
        } else {
            false
        };

        let mut tx = Transaction {
            id: 0,
            version,
            type_id: TransactionType::from_type(type_byte),
            subtype,
            timestamp: timestamp as u32,
            deadline,
            sender_public_key: Hash256(sender_pk),
            sender_id,
            recipient_id: Some(recipient_id),
            amount,
            fee,
            height: 0,
            block_id: 0,
            block_timestamp: 0,
            transaction_index: 0,
            signature: Signature(signature),
            full_hash: Hash256([0u8; 32]),
            referenced_transaction_full_hash,
            attachment_bytes,
            pruned_attachment_bytes: 0,
            attachment_json: None,
            phased,
            has_message,
            has_encrypted_message,
            has_public_key_announcement: false,
            has_prunable_message: false,
            has_prunable_attachment: false,
            ec_block_height,
            ec_block_id,
            has_encrypttoself_message: false,
            has_prunable_encrypted_message: false,
        };

        tx.full_hash = tx.calculate_full_hash().unwrap_or(Hash256([0u8; 32]));
        tx.id = tx.calculate_id();

        Ok(tx)
    }

    pub fn sign_transaction_with_passphrase(
        tx: &mut Transaction,
        secret_phrase: &str,
    ) -> std::result::Result<(), ApiError> {
        let kp = passphrase_to_keypair(secret_phrase)
            .map_err(|e| ApiError::Internal(format!("failed to create keypair: {}", e)))?;

        let message = tx.serialize_for_signing();
        let sig_bytes = kp.sign(&message);
        tx.signature = Signature(sig_bytes);

        tx.full_hash = tx.calculate_full_hash()
            .unwrap_or(Hash256([0u8; 32]));
        tx.id = tx.calculate_id();

        Ok(())
    }

    pub fn build_transaction_response(tx: &Transaction, broadcasted: bool) -> RsRespWithData {
        let mut builder = RsRespBuilder::new();

        let tx_json = transaction_to_json(tx);
        builder.insert("transactionJSON", tx_json);

        let unsigned_bytes = tx.serialize_for_signing();
        builder.insert("unsignedTransactionBytes", hex::encode(&unsigned_bytes));

        builder
            .insert("transaction", tx.id.to_string())
            .insert("fullHash", hex::encode(tx.full_hash.0))
            .insert("transactionBytes", hex::encode(tx.get_bytes()));

        let sig_hash = sha256(&tx.signature.0);
        builder.insert("signatureHash", hex::encode(sig_hash));

        builder.insert("broadcasted", broadcasted);

        builder.build()
    }
}

fn transaction_to_json(tx: &Transaction) -> serde_json::Value {
    let mut obj = serde_json::Map::new();

    obj.insert("type".to_string(), json!(u8::from(tx.type_id)));
    obj.insert("subtype".to_string(), json!(tx.subtype));
    obj.insert("version".to_string(), json!(tx.version));
    obj.insert("timestamp".to_string(), json!(tx.timestamp));
    obj.insert("deadline".to_string(), json!(tx.deadline));
    obj.insert("senderPublicKey".to_string(), json!(hex::encode(tx.sender_public_key.0)));
    obj.insert("sender".to_string(), json!(tx.sender_id.to_string()));
    obj.insert("senderRS".to_string(), json!(format_account_rs(tx.sender_id)));

    if let Some(recipient) = tx.recipient_id {
        obj.insert("recipient".to_string(), json!(recipient.to_string()));
        obj.insert("recipientRS".to_string(), json!(format_account_rs(recipient)));
    }

    obj.insert("amountNQT".to_string(), json!(tx.amount.to_string()));
    obj.insert("feeNQT".to_string(), json!(tx.fee.to_string()));
    obj.insert("fullHash".to_string(), json!(hex::encode(tx.full_hash.0)));
    obj.insert("signature".to_string(), json!(hex::encode(tx.signature.0)));
    obj.insert("phased".to_string(), json!(tx.phased));

    if let Some(ref hash) = tx.referenced_transaction_full_hash {
        obj.insert("referencedTransactionFullHash".to_string(), json!(hex::encode(hash.0)));
    }

    if let Some(eb_height) = tx.ec_block_height {
        obj.insert("ecBlockHeight".to_string(), json!(eb_height));
    }
    if let Some(eb_id) = tx.ec_block_id {
        obj.insert("ecBlockId".to_string(), json!(eb_id.to_string()));
    }

    if let Some(ref att) = tx.attachment_json {
        obj.insert("attachment".to_string(), json!(att.clone()));
    }

    serde_json::Value::Object(obj)
}

fn format_account_rs(account_id: u64) -> String {
    common::convert::rs_account(account_id)
}
