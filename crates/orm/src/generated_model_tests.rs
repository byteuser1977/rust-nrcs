// Auto-generated tests for ORM models. DO NOT EDIT MANUALLY.
#[cfg(test)]
mod generated_model_tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    #[test]
    fn test__construction() {
        let _ = ACCOUNTModel {
            db_id: 0,
            id: 0,
            balance: 0,
            unconfirmed_balance: 0,
            forged_balance: 0,
            active_lessee_id: None,
            has_control_phasing: false,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ACCOUNTASSETModel {
            db_id: 0,
            account_id: 0,
            asset_id: 0,
            quantity: 0,
            unconfirmed_quantity: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ACCOUNTCONTROLPHASINGModel {
            db_id: 0,
            account_id: 0,
            whitelist: None,
            voting_model: 0,
            quorum: None,
            min_balance: None,
            holding_id: None,
            min_balance_model: None,
            max_fees: None,
            min_duration: None,
            max_duration: None,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ACCOUNTCURRENCYModel {
            db_id: 0,
            account_id: 0,
            currency_id: 0,
            units: 0,
            unconfirmed_units: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ACCOUNTFXTModel {
            id: 0,
            balance: vec![],
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ACCOUNTGUARANTEEDBALANCEModel {
            db_id: 0,
            account_id: 0,
            additions: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ACCOUNTINFOModel {
            db_id: 0,
            account_id: 0,
            name: None,
            description: None,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ACCOUNTLEASEModel {
            db_id: 0,
            lessor_id: 0,
            current_leasing_height_from: None,
            current_leasing_height_to: None,
            current_lessee_id: None,
            next_leasing_height_from: None,
            next_leasing_height_to: None,
            next_lessee_id: None,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ACCOUNTLEDGERModel {
            db_id: 0,
            account_id: 0,
            event_type: 0,
            event_id: 0,
            holding_type: 0,
            holding_id: None,
            change: 0,
            balance: 0,
            block_id: 0,
            height: 0,
            timestamp: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ACCOUNTPROPERTYModel {
            db_id: 0,
            id: 0,
            recipient_id: 0,
            setter_id: None,
            property: String::new(),
            value: None,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ALIASModel {
            db_id: 0,
            id: 0,
            account_id: 0,
            alias_name: String::new(),
            alias_name_lower: String::new(),
            alias_uri: String::new(),
            timestamp: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ALIASOFFERModel {
            db_id: 0,
            id: 0,
            price: 0,
            buyer_id: None,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ASKORDERModel {
            db_id: 0,
            id: 0,
            account_id: 0,
            asset_id: 0,
            price: 0,
            transaction_index: 0,
            transaction_height: 0,
            quantity: 0,
            creation_height: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ASSETModel {
            db_id: 0,
            id: 0,
            account_id: 0,
            name: String::new(),
            description: None,
            quantity: 0,
            decimals: 0,
            initial_quantity: 0,
            has_control_phasing: false,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ASSETCONTROLPHASINGModel {
            db_id: 0,
            asset_id: 0,
            voting_model: 0,
            quorum: None,
            min_balance: None,
            holding_id: None,
            min_balance_model: None,
            whitelist: None,
            expression: None,
            sender_property_setter_id: None,
            sender_property_name: None,
            sender_property_value: None,
            recipient_property_setter_id: None,
            recipient_property_name: None,
            recipient_property_value: None,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ASSETCONTROLPHASINGSUBPOLLModel {
            db_id: 0,
            asset_id: 0,
            name: None,
            voting_model: 0,
            quorum: None,
            min_balance: None,
            holding_id: None,
            min_balance_model: None,
            whitelist: None,
            sender_property_setter_id: None,
            sender_property_name: None,
            sender_property_value: None,
            recipient_property_setter_id: None,
            recipient_property_name: None,
            recipient_property_value: None,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ASSETDELETEModel {
            db_id: 0,
            id: 0,
            asset_id: 0,
            account_id: 0,
            quantity: 0,
            timestamp: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ASSETDIVIDENDModel {
            db_id: 0,
            id: 0,
            asset_id: 0,
            amount: 0,
            dividend_height: 0,
            total_dividend: 0,
            num_accounts: 0,
            timestamp: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ASSETHISTORYModel {
            db_id: 0,
            id: 0,
            full_hash: vec![],
            asset_id: 0,
            account_id: 0,
            quantity: 0,
            timestamp: 0,
            chain_id: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ASSETPROPERTYModel {
            db_id: 0,
            id: 0,
            asset_id: 0,
            setter_id: 0,
            property: String::new(),
            value: None,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = ASSETTRANSFERModel {
            db_id: 0,
            id: 0,
            asset_id: 0,
            sender_id: 0,
            recipient_id: 0,
            quantity: 0,
            timestamp: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = BALANCEModel {
            db_id: 0,
            account_id: 0,
            balance: 0,
            unconfirmed_balance: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = BIDORDERModel {
            db_id: 0,
            id: 0,
            account_id: 0,
            asset_id: 0,
            price: 0,
            transaction_index: 0,
            transaction_height: 0,
            quantity: 0,
            creation_height: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = BLOCKModel {
            db_id: 0,
            id: 0,
            version: 0,
            timestamp: 0,
            previous_block_id: None,
            total_amount: 0,
            total_fee: 0,
            payload_length: 0,
            previous_block_hash: None,
            cumulative_difficulty: vec![],
            base_target: 0,
            next_block_id: None,
            height: 0,
            generation_signature: String::new(),
            block_signature: vec![],
            payload_hash: vec![],
            generator_id: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = BUYOFFERModel {
            db_id: 0,
            id: 0,
            currency_id: 0,
            account_id: 0,
            rate: 0,
            unit_limit: 0,
            supply: 0,
            expiration_height: 0,
            transaction_height: 0,
            creation_height: 0,
            transaction_index: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = COINORDERFXTModel {
            db_id: 0,
            id: 0,
            account_id: 0,
            chain_id: 0,
            exchange_id: 0,
            full_hash: vec![],
            amount: 0,
            quantity: 0,
            bid_price: 0,
            ask_price: 0,
            creation_height: 0,
            height: 0,
            transaction_height: 0,
            transaction_index: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = COINTRADEFXTModel {
            db_id: 0,
            chain_id: 0,
            exchange_id: 0,
            account_id: 0,
            block_id: 0,
            height: 0,
            timestamp: 0,
            exchange_quantity: 0,
            exchange_price: 0,
            order_id: 0,
            order_full_hash: vec![],
            match_id: 0,
            match_full_hash: vec![],
        };
    }

    #[test]
    fn test__construction() {
        let _ = CONTRACTREFERENCEModel {
            db_id: 0,
            id: 0,
            account_id: 0,
            contract_name: String::new(),
            contract_params: None,
            contract_transaction_chain_id: 0,
            contract_transaction_full_hash: None,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = CURRENCYModel {
            db_id: 0,
            id: 0,
            account_id: 0,
            name: String::new(),
            name_lower: String::new(),
            code: String::new(),
            description: None,
            type: 0,
            initial_supply: 0,
            reserve_supply: 0,
            max_supply: 0,
            creation_height: 0,
            issuance_height: 0,
            min_reserve_per_unit_nqt: 0,
            min_difficulty: 0,
            max_difficulty: 0,
            ruleset: 0,
            algorithm: 0,
            decimals: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = CURRENCYFOUNDERModel {
            db_id: 0,
            currency_id: 0,
            account_id: 0,
            amount: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = CURRENCYMINTModel {
            db_id: 0,
            currency_id: 0,
            account_id: 0,
            counter: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = CURRENCYSUPPLYModel {
            db_id: 0,
            id: 0,
            current_supply: 0,
            current_reserve_per_unit_nqt: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = CURRENCYTRANSFERModel {
            db_id: 0,
            id: 0,
            currency_id: 0,
            sender_id: 0,
            recipient_id: 0,
            units: 0,
            timestamp: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = DATATAGModel {
            db_id: 0,
            tag: String::new(),
            tag_count: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = EXCHANGEModel {
            db_id: 0,
            transaction_id: 0,
            currency_id: 0,
            block_id: 0,
            offer_id: 0,
            seller_id: 0,
            buyer_id: 0,
            units: 0,
            rate: 0,
            timestamp: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = EXCHANGEREQUESTModel {
            db_id: 0,
            id: 0,
            account_id: 0,
            currency_id: 0,
            units: 0,
            rate: 0,
            is_buy: false,
            timestamp: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = GOODSModel {
            db_id: 0,
            id: 0,
            seller_id: 0,
            name: String::new(),
            description: None,
            parsed_tags: None,
            tags: None,
            timestamp: 0,
            quantity: 0,
            price: 0,
            delisted: false,
            height: 0,
            latest: false,
            has_image: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = HUBModel {
            db_id: 0,
            account_id: None,
            min_fee_per_byte: None,
            uris: None,
            height: None,
            latest: None,
        };
    }

    #[test]
    fn test__construction() {
        let _ = PEERModel {
            address: String::new(),
            last_updated: None,
            services: None,
        };
    }

    #[test]
    fn test__construction() {
        let _ = PHASINGPOLLModel {
            db_id: 0,
            id: 0,
            account_id: 0,
            whitelist_size: 0,
            finish_height: 0,
            voting_model: 0,
            quorum: None,
            min_balance: None,
            holding_id: None,
            min_balance_model: None,
            hashed_secret: None,
            algorithm: None,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = PHASINGPOLLHASHEDSECRETModel {
            db_id: 0,
            hashed_secret: vec![],
            hashed_secret_id: 0,
            algorithm: 0,
            transaction_full_hash: None,
            transaction_id: 0,
            chain_id: 0,
            finish_height: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = PHASINGPOLLLINKEDTRANSACTIONModel {
            db_id: 0,
            transaction_id: 0,
            linked_full_hash: vec![],
            linked_transaction_id: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = PHASINGPOLLRESULTModel {
            db_id: 0,
            id: 0,
            result: 0,
            approved: false,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = PHASINGPOLLVOTERModel {
            db_id: 0,
            transaction_id: 0,
            voter_id: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = PHASINGVOTEModel {
            db_id: 0,
            vote_id: 0,
            transaction_id: 0,
            voter_id: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = POLLModel {
            db_id: 0,
            id: 0,
            account_id: 0,
            name: String::new(),
            description: None,
            options: String::new(),
            min_num_options: None,
            max_num_options: None,
            min_range_value: None,
            max_range_value: None,
            timestamp: 0,
            finish_height: 0,
            voting_model: 0,
            min_balance: None,
            min_balance_model: None,
            holding_id: None,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = POLLRESULTModel {
            db_id: 0,
            poll_id: 0,
            result: None,
            weight: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = PRUNABLEMESSAGEModel {
            db_id: 0,
            id: 0,
            sender_id: 0,
            recipient_id: None,
            message: None,
            message_is_text: false,
            is_compressed: false,
            encrypted_message: None,
            encrypted_is_text: None,
            block_timestamp: 0,
            transaction_timestamp: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = PUBLICKEYModel {
            db_id: 0,
            account_id: 0,
            public_key: None,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = PURCHASEModel {
            db_id: 0,
            id: 0,
            buyer_id: 0,
            goods_id: 0,
            seller_id: 0,
            quantity: 0,
            price: 0,
            deadline: 0,
            note: None,
            nonce: None,
            timestamp: 0,
            pending: false,
            goods: None,
            goods_nonce: None,
            goods_is_text: false,
            refund_note: None,
            refund_nonce: None,
            has_feedback_notes: false,
            has_public_feedbacks: false,
            discount: 0,
            refund: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = PURCHASEFEEDBACKModel {
            db_id: 0,
            id: 0,
            feedback_data: vec![],
            feedback_nonce: vec![],
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = PURCHASEPUBLICFEEDBACKModel {
            db_id: 0,
            id: 0,
            public_feedback: String::new(),
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = REFERENCEDTRANSACTIONModel {
            db_id: 0,
            transaction_id: 0,
            referenced_transaction_id: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = SCANModel {
            rescan: false,
            height: 0,
            validate: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = SELLOFFERModel {
            db_id: 0,
            id: 0,
            currency_id: 0,
            account_id: 0,
            rate: 0,
            unit_limit: 0,
            supply: 0,
            expiration_height: 0,
            transaction_height: 0,
            creation_height: 0,
            transaction_index: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = SHUFFLINGModel {
            db_id: 0,
            id: 0,
            holding_id: None,
            holding_type: 0,
            issuer_id: 0,
            amount: 0,
            participant_count: 0,
            blocks_remaining: None,
            stage: 0,
            assignee_account_id: None,
            registrant_count: 0,
            recipient_public_keys: None,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = SHUFFLINGDATAModel {
            db_id: 0,
            shuffling_id: 0,
            account_id: 0,
            data: None,
            transaction_timestamp: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = SHUFFLINGPARTICIPANTModel {
            db_id: 0,
            shuffling_id: 0,
            account_id: 0,
            next_account_id: None,
            participant_index: 0,
            state: 0,
            blame_data: None,
            key_seeds: None,
            data_transaction_full_hash: None,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = TAGModel {
            db_id: 0,
            tag: String::new(),
            in_stock_count: 0,
            total_count: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = TAGGEDDATAModel {
            db_id: 0,
            id: 0,
            account_id: 0,
            name: String::new(),
            description: None,
            tags: None,
            parsed_tags: None,
            type: None,
            data: vec![],
            is_text: false,
            filename: None,
            channel: None,
            block_timestamp: 0,
            transaction_timestamp: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = TAGGEDDATAEXTENDModel {
            db_id: 0,
            id: 0,
            extend_id: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = TAGGEDDATATIMESTAMPModel {
            db_id: 0,
            id: 0,
            timestamp: 0,
            height: 0,
            latest: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = TRADEModel {
            db_id: 0,
            asset_id: 0,
            block_id: 0,
            ask_order_id: 0,
            bid_order_id: 0,
            ask_order_height: 0,
            bid_order_height: 0,
            seller_id: 0,
            buyer_id: 0,
            is_buy: false,
            quantity: 0,
            price: 0,
            timestamp: 0,
            height: 0,
        };
    }

    #[test]
    fn test__construction() {
        let _ = TRANSACTIONModel {
            db_id: 0,
            id: 0,
            deadline: 0,
            recipient_id: None,
            amount: 0,
            fee: 0,
            full_hash: vec![],
            height: 0,
            block_id: 0,
            signature: vec![],
            timestamp: 0,
            type: 0,
            subtype: 0,
            sender_id: 0,
            block_timestamp: 0,
            referenced_transaction_full_hash: None,
            transaction_index: 0,
            phased: false,
            attachment_bytes: None,
            version: 0,
            has_message: false,
            has_encrypted_message: false,
            has_public_key_announcement: false,
            has_prunable_message: false,
            has_prunable_attachment: false,
            ec_block_height: None,
            ec_block_id: None,
            has_encrypttoself_message: false,
            has_prunable_encrypted_message: false,
        };
    }

    #[test]
    fn test__construction() {
        let _ = UNCONFIRMEDTRANSACTIONModel {
            db_id: 0,
            id: 0,
            expiration: 0,
            transaction_height: 0,
            fee_per_byte: 0,
            arrival_timestamp: 0,
            transaction_bytes: vec![],
            height: 0,
            prunable_json: None,
        };
    }

    #[test]
    fn test__construction() {
        let _ = VOTEModel {
            db_id: 0,
            id: 0,
            poll_id: 0,
            voter_id: 0,
            vote_bytes: vec![],
            height: 0,
        };
    }

}
