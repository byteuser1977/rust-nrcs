CREATE TABLE IF NOT EXISTS account
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 balance BIGINT NOT NULL,
 unconfirmed_balance BIGINT NOT NULL,
 forged_balance BIGINT NOT NULL,
 active_lessee_id BIGINT,
 has_control_phasing SMALLINT DEFAULT FALSE NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX account_id_height_idx ON account(id, height desc);
CREATE INDEX account_active_lessee_id_idx ON account(active_lessee_id);
CREATE INDEX account_height_id_idx ON account(height, id);
CREATE TABLE IF NOT EXISTS account_asset
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 account_id BIGINT NOT NULL,
 asset_id BIGINT NOT NULL,
 quantity BIGINT NOT NULL,
 unconfirmed_quantity BIGINT NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX account_asset_id_height_idx ON account_asset(account_id, asset_id, height desc);
CREATE INDEX account_asset_asset_id_idx ON account_asset(asset_id);
CREATE INDEX account_asset_quantity_idx ON account_asset(quantity desc);
CREATE INDEX account_asset_height_id_idx ON account_asset(height, account_id, asset_id);
CREATE TABLE IF NOT EXISTS account_control_phasing
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 account_id BIGINT NOT NULL,
 whitelist varchar(5000),
 voting_model SMALLINT NOT NULL,
 quorum BIGINT,
 min_balance BIGINT,
 holding_id BIGINT,
 min_balance_model SMALLINT,
 max_fees BIGINT,
 min_duration SMALLINT,
 max_duration SMALLINT,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX account_control_phasing_id_height_idx ON account_control_phasing(account_id, height desc);
CREATE INDEX account_control_phasing_height_id_idx ON account_control_phasing(height, account_id);
CREATE TABLE IF NOT EXISTS account_currency
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 account_id BIGINT NOT NULL,
 currency_id BIGINT NOT NULL,
 units BIGINT NOT NULL,
 unconfirmed_units BIGINT NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX account_currency_id_height_idx ON account_currency(account_id, currency_id, height desc);
CREATE INDEX account_currency_currency_id_idx ON account_currency(currency_id);
CREATE INDEX account_currency_units_idx ON account_currency(units desc);
CREATE INDEX account_currency_height_id_idx ON account_currency(height, account_id, currency_id);
CREATE TABLE IF NOT EXISTS account_fxt
(
 id BIGINT NOT NULL,
 balance BLOB(1M) NOT NULL,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX account_fxt_id_idx ON account_fxt(id, height desc);
CREATE TABLE IF NOT EXISTS account_guaranteed_balance
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 account_id BIGINT NOT NULL,
 additions BIGINT NOT NULL,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX account_guaranteed_balance_id_height_idx ON account_guaranteed_balance(account_id, height desc);
CREATE INDEX account_guaranteed_balance_height_idx ON account_guaranteed_balance(height);
CREATE TABLE IF NOT EXISTS account_info
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 account_id BIGINT NOT NULL,
 name varchar(5000),
 description varchar(5000),
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX account_info_id_height_idx ON account_info(account_id, height desc);
CREATE INDEX account_info_height_id_idx ON account_info(height, account_id);
CREATE TABLE IF NOT EXISTS account_lease
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 lessor_id BIGINT NOT NULL,
 current_leasing_height_from INTEGER,
 current_leasing_height_to INTEGER,
 current_lessee_id BIGINT,
 next_leasing_height_from INTEGER,
 next_leasing_height_to INTEGER,
 next_lessee_id BIGINT,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX account_lease_lessor_id_height_idx ON account_lease(lessor_id, height desc);
CREATE INDEX account_lease_current_leasing_height_from_idx ON account_lease(current_leasing_height_from);
CREATE INDEX account_lease_current_leasing_height_to_idx ON account_lease(current_leasing_height_to);
CREATE INDEX account_lease_height_id_idx ON account_lease(height, lessor_id);
CREATE TABLE IF NOT EXISTS account_ledger
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 account_id BIGINT NOT NULL,
 event_type SMALLINT NOT NULL,
 event_id BIGINT NOT NULL,
 holding_type SMALLINT NOT NULL,
 holding_id BIGINT,
 "change" BIGINT NOT NULL,
 balance BIGINT NOT NULL,
 block_id BIGINT NOT NULL,
 height INTEGER NOT NULL,
 timestamp INTEGER NOT NULL
);
CREATE INDEX account_ledger_id_idx ON account_ledger(account_id, db_id);
CREATE INDEX account_ledger_height_idx ON account_ledger(height);
CREATE TABLE IF NOT EXISTS account_property
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 recipient_id BIGINT NOT NULL,
 setter_id BIGINT,
 property varchar(5000) NOT NULL,
 value varchar(5000),
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX account_property_id_height_idx ON account_property(id, height desc);
CREATE INDEX account_property_recipient_height_idx ON account_property(recipient_id, height desc);
CREATE INDEX account_property_setter_recipient_idx ON account_property(setter_id, recipient_id);
CREATE INDEX account_property_height_id_idx ON account_property(height, id);
CREATE TABLE IF NOT EXISTS alias
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 alias_name varchar(320) NOT NULL,
 alias_name_LOWER varchar(320) NOT NULL,
 alias_uri varchar(5000) NOT NULL,
 timestamp INTEGER NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX alias_id_height_idx ON alias(id, height desc);
CREATE INDEX alias_account_id_idx ON alias(account_id, height desc);
CREATE INDEX alias_name_lower_idx ON alias(alias_name_lower);
CREATE INDEX alias_height_id_idx ON alias(height, id);
CREATE TABLE IF NOT EXISTS alias_offer
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 price BIGINT NOT NULL,
 buyer_id BIGINT,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX alias_offer_id_height_idx ON alias_offer(id, height desc);
CREATE INDEX alias_offer_height_id_idx ON alias_offer(height, id);
CREATE TABLE IF NOT EXISTS ask_order
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 asset_id BIGINT NOT NULL,
 price BIGINT NOT NULL,
 transaction_index SMALLINT NOT NULL,
 transaction_height INTEGER NOT NULL,
 quantity BIGINT NOT NULL,
 creation_height INTEGER NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX ask_order_id_height_idx ON ask_order(id, height desc);
CREATE INDEX ask_order_account_id_idx ON ask_order(account_id, height desc);
CREATE INDEX ask_order_asset_id_price_idx ON ask_order(asset_id, price);
CREATE INDEX ask_order_creation_idx ON ask_order(creation_height desc);
CREATE INDEX ask_order_height_id_idx ON ask_order(height, id);
CREATE TABLE IF NOT EXISTS asset
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 name varchar(5000) NOT NULL,
 description varchar(5000),
 quantity BIGINT NOT NULL,
 decimals SMALLINT NOT NULL,
 has_control_phasing SMALLINT DEFAULT FALSE NOT NULL,
 initial_quantity BIGINT NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX asset_id_height_idx ON asset(id, height desc);
CREATE INDEX asset_account_id_idx ON asset(account_id);
CREATE INDEX asset_height_id_idx ON asset(height, id);
CREATE TABLE IF NOT EXISTS asset_delete
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 asset_id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 quantity BIGINT NOT NULL,
 timestamp INTEGER NOT NULL,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX asset_delete_id_idx ON asset_delete(id);
CREATE INDEX asset_delete_asset_id_idx ON asset_delete(asset_id, height desc);
CREATE INDEX asset_delete_account_id_idx ON asset_delete(account_id, height desc);
CREATE INDEX asset_delete_height_idx ON asset_delete(height);
CREATE TABLE IF NOT EXISTS asset_dividend
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 asset_id BIGINT NOT NULL,
 amount BIGINT NOT NULL,
 dividend_height INTEGER NOT NULL,
 total_dividend BIGINT NOT NULL,
 num_accounts BIGINT NOT NULL,
 timestamp INTEGER NOT NULL,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX asset_dividend_id_idx ON asset_dividend(id);
CREATE INDEX asset_dividend_asset_id_idx ON asset_dividend(asset_id, height desc);
CREATE INDEX asset_dividend_height_idx ON asset_dividend(height);
CREATE TABLE IF NOT EXISTS asset_transfer
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 asset_id BIGINT NOT NULL,
 sender_id BIGINT NOT NULL,
 recipient_id BIGINT NOT NULL,
 quantity BIGINT NOT NULL,
 timestamp INTEGER NOT NULL,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX asset_transfer_id_idx ON asset_transfer(id);
CREATE INDEX asset_transfer_asset_id_idx ON asset_transfer(asset_id, height desc);
CREATE INDEX asset_transfer_sender_id_idx ON asset_transfer(sender_id, height desc);
CREATE INDEX asset_transfer_recipient_id_idx ON asset_transfer(recipient_id, height desc);
CREATE INDEX asset_transfer_height_idx ON asset_transfer(height);
CREATE TABLE IF NOT EXISTS bid_order
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 asset_id BIGINT NOT NULL,
 price BIGINT NOT NULL,
 transaction_index SMALLINT NOT NULL,
 transaction_height INTEGER NOT NULL,
 quantity BIGINT NOT NULL,
 creation_height INTEGER NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX bid_order_id_height_idx ON bid_order(id, height desc);
CREATE INDEX bid_order_account_id_idx ON bid_order(account_id, height desc);
CREATE INDEX bid_order_asset_id_price_idx ON bid_order(asset_id, price desc);
CREATE INDEX bid_order_creation_idx ON bid_order(creation_height desc);
CREATE INDEX bid_order_height_id_idx ON bid_order(height, id);
CREATE TABLE IF NOT EXISTS block
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 version INTEGER NOT NULL,
 timestamp INTEGER NOT NULL,
 previous_block_id BIGINT,
 total_amount BIGINT NOT NULL,
 total_fee BIGINT NOT NULL,
 payload_length INTEGER NOT NULL,
 previous_block_hash BLOB(1M),
 cumulative_difficulty BLOB(1M) NOT NULL,
 base_target BIGINT NOT NULL,
 next_block_id BIGINT,
 height INTEGER NOT NULL,
 generation_signature BLOB(1M) NOT NULL,
 block_signature BLOB(1M) NOT NULL,
 payload_hash BLOB(1M) NOT NULL,
 generator_id BIGINT NOT NULL
);
CREATE UNIQUE INDEX block_id_idx ON block(id);
CREATE UNIQUE INDEX block_timestamp_idx ON block(timestamp desc);
CREATE UNIQUE INDEX block_height_idx ON block(height);
CREATE INDEX block_generator_id_idx ON block(generator_id);
CREATE TABLE IF NOT EXISTS buy_offer
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 currency_id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 rate BIGINT NOT NULL,
 unit_limit BIGINT NOT NULL,
 supply BIGINT NOT NULL,
 expiration_height INTEGER NOT NULL,
 transaction_height INTEGER NOT NULL,
 creation_height INTEGER NOT NULL,
 transaction_index SMALLINT NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX buy_offer_id_idx ON buy_offer(id, height desc);
CREATE INDEX buy_offer_currency_id_account_id_idx ON buy_offer(currency_id, account_id, height desc);
CREATE INDEX buy_offer_rate_height_idx ON buy_offer(rate desc, creation_height);
CREATE INDEX buy_offer_height_id_idx ON buy_offer(height, id);
CREATE TABLE IF NOT EXISTS currency
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 name varchar(320) NOT NULL,
 name_LOWER varchar(320) NOT NULL,
 code varchar(320) NOT NULL,
 description varchar(5000),
 type INTEGER NOT NULL,
 initial_supply BIGINT DEFAULT 0 NOT NULL,
 reserve_supply BIGINT NOT NULL,
 max_supply BIGINT NOT NULL,
 creation_height INTEGER NOT NULL,
 issuance_height INTEGER NOT NULL,
 min_reserve_per_unit_nqt BIGINT NOT NULL,
 min_difficulty SMALLINT NOT NULL,
 max_difficulty SMALLINT NOT NULL,
 ruleset SMALLINT NOT NULL,
 algorithm SMALLINT NOT NULL,
 decimals SMALLINT DEFAULT 0 NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX currency_id_height_idx ON currency(id, height desc);
CREATE INDEX currency_account_id_idx ON currency(account_id);
CREATE INDEX currency_name_idx ON currency(name_lower, height desc);
CREATE INDEX currency_code_idx ON currency(code, height desc);
CREATE INDEX currency_creation_height_idx ON currency(creation_height desc);
CREATE INDEX currency_issuance_height_idx ON currency(issuance_height);
CREATE INDEX currency_height_id_idx ON currency(height, id);
CREATE TABLE IF NOT EXISTS currency_founder
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 currency_id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 amount BIGINT NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX currency_founder_currency_id_idx ON currency_founder(currency_id, account_id, height desc);
CREATE INDEX currency_founder_account_id_idx ON currency_founder(account_id, height desc);
CREATE INDEX currency_founder_height_id_idx ON currency_founder(height, currency_id, account_id);
CREATE TABLE IF NOT EXISTS currency_mint
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 currency_id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 counter BIGINT NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX currency_mint_currency_id_account_id_idx ON currency_mint(currency_id, account_id, height desc);
CREATE INDEX currency_mint_height_id_idx ON currency_mint(height, currency_id, account_id);
CREATE TABLE IF NOT EXISTS currency_supply
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 current_supply BIGINT NOT NULL,
 current_reserve_per_unit_nqt BIGINT NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX currency_supply_id_height_idx ON currency_supply(id, height desc);
CREATE INDEX currency_supply_height_id_idx ON currency_supply(height, id);
CREATE TABLE IF NOT EXISTS currency_transfer
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 currency_id BIGINT NOT NULL,
 sender_id BIGINT NOT NULL,
 recipient_id BIGINT NOT NULL,
 units BIGINT NOT NULL,
 timestamp INTEGER NOT NULL,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX currency_transfer_id_idx ON currency_transfer(id);
CREATE INDEX currency_transfer_currency_id_idx ON currency_transfer(currency_id, height desc);
CREATE INDEX currency_transfer_sender_id_idx ON currency_transfer(sender_id, height desc);
CREATE INDEX currency_transfer_recipient_id_idx ON currency_transfer(recipient_id, height desc);
CREATE INDEX currency_transfer_height_idx ON currency_transfer(height);
CREATE TABLE IF NOT EXISTS data_tag
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 tag varchar(300) NOT NULL,
 tag_count INTEGER NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL,
 constraint constraint_995 FOREIGN KEY (height) references block(height) ON DELETE CASCADE
);
CREATE UNIQUE INDEX data_tag_tag_height_idx ON data_tag(tag, height desc);
CREATE INDEX data_tag_count_height_idx ON data_tag(tag_count desc, height desc);

CREATE TABLE IF NOT EXISTS exchange
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 transaction_id BIGINT NOT NULL,
 currency_id BIGINT NOT NULL,
 block_id BIGINT NOT NULL,
 offer_id BIGINT NOT NULL,
 seller_id BIGINT NOT NULL,
 buyer_id BIGINT NOT NULL,
 units BIGINT NOT NULL,
 rate BIGINT NOT NULL,
 timestamp INTEGER NOT NULL,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX exchange_offer_idx ON exchange(transaction_id, offer_id);
CREATE INDEX exchange_currency_id_idx ON exchange(currency_id, height desc);
CREATE INDEX exchange_seller_id_idx ON exchange(seller_id, height desc);
CREATE INDEX exchange_buyer_id_idx ON exchange(buyer_id, height desc);
CREATE INDEX exchange_height_db_id_idx ON exchange(height desc, db_id desc);
CREATE INDEX exchange_height_idx ON exchange(height);
CREATE TABLE IF NOT EXISTS exchange_request
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 currency_id BIGINT NOT NULL,
 units BIGINT NOT NULL,
 rate BIGINT NOT NULL,
 is_buy SMALLINT NOT NULL,
 timestamp INTEGER NOT NULL,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX exchange_request_id_idx ON exchange_request(id);
CREATE INDEX exchange_request_account_currency_idx ON exchange_request(account_id, currency_id, height desc);
CREATE INDEX exchange_request_currency_idx ON exchange_request(currency_id, height desc);
CREATE INDEX exchange_request_height_db_id_idx ON exchange_request(height desc, db_id desc);
CREATE INDEX exchange_request_height_idx ON exchange_request(height);
CREATE TABLE IF NOT EXISTS goods
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 seller_id BIGINT NOT NULL,
 name varchar(300) NOT NULL,
 description varchar(5000),
 parsed_tags varchar(5000),
 tags varchar(2000),
 timestamp INTEGER NOT NULL,
 quantity INTEGER NOT NULL,
 price BIGINT NOT NULL,
 delisted SMALLINT NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL,
 has_image SMALLINT DEFAULT FALSE NOT NULL
);
CREATE UNIQUE INDEX goods_id_height_idx ON goods(id, height desc);
CREATE INDEX goods_seller_id_name_idx ON goods(seller_id, name);
CREATE INDEX goods_timestamp_idx ON goods(timestamp desc, height desc);
CREATE INDEX goods_height_id_idx ON goods(height, id);
CREATE TABLE IF NOT EXISTS hub
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 account_id BIGINT,
 min_fee_per_byte BIGINT,
 uris varchar(5000),
 height INTEGER,
 latest SMALLINT
);

CREATE TABLE IF NOT EXISTS peer
(
 address varchar(220) PRIMARY KEY NOT NULL,
 last_updated INTEGER,
 services BIGINT
);

CREATE TABLE IF NOT EXISTS phasing_poll
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 whitelist_size SMALLINT DEFAULT 0 NOT NULL,
 finish_height INTEGER NOT NULL,
 voting_model SMALLINT NOT NULL,
 quorum BIGINT,
 min_balance BIGINT,
 holding_id BIGINT,
 min_balance_model SMALLINT,
 hashed_secret BLOB(1M),
 algorithm SMALLINT,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX phasing_poll_id_idx ON phasing_poll(id);
CREATE INDEX phasing_poll_account_id_idx ON phasing_poll(account_id, height desc);
CREATE INDEX phasing_poll_holding_id_idx ON phasing_poll(holding_id, height desc);
CREATE INDEX phasing_poll_height_idx ON phasing_poll(height);
CREATE TABLE IF NOT EXISTS phasing_poll_linked_transaction
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 transaction_id BIGINT NOT NULL,
 linked_full_hash BLOB(1M) NOT NULL,
 linked_transaction_id BIGINT NOT NULL,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX phasing_poll_linked_transaction_id_link_idx ON phasing_poll_linked_transaction(transaction_id, linked_transaction_id);
CREATE UNIQUE INDEX phasing_poll_linked_transaction_link_id_idx ON phasing_poll_linked_transaction(linked_transaction_id, transaction_id);
CREATE INDEX phasing_poll_linked_transaction_height_idx ON phasing_poll_linked_transaction(height);
CREATE TABLE IF NOT EXISTS phasing_poll_result
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 result BIGINT NOT NULL,
 approved SMALLINT NOT NULL,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX phasing_poll_result_id_idx ON phasing_poll_result(id);
CREATE INDEX phasing_poll_result_height_idx ON phasing_poll_result(height);
CREATE TABLE IF NOT EXISTS phasing_poll_voter
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 transaction_id BIGINT NOT NULL,
 voter_id BIGINT NOT NULL,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX phasing_poll_voter_transaction_voter_idx ON phasing_poll_voter(transaction_id, voter_id);
CREATE INDEX phasing_poll_voter_height_idx ON phasing_poll_voter(height);
CREATE TABLE IF NOT EXISTS phasing_vote
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 vote_id BIGINT NOT NULL,
 transaction_id BIGINT NOT NULL,
 voter_id BIGINT NOT NULL,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX phasing_vote_transaction_voter_idx ON phasing_vote(transaction_id, voter_id);
CREATE INDEX phasing_vote_height_idx ON phasing_vote(height);
CREATE TABLE IF NOT EXISTS poll
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 name varchar(5000) NOT NULL,
 description varchar(5000),
 options varchar(5000) NOT NULL,
 min_num_options SMALLINT,
 max_num_options SMALLINT,
 min_range_value SMALLINT,
 max_range_value SMALLINT,
 timestamp INTEGER NOT NULL,
 finish_height INTEGER NOT NULL,
 voting_model SMALLINT NOT NULL,
 min_balance BIGINT,
 min_balance_model SMALLINT,
 holding_id BIGINT,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX poll_id_idx ON poll(id);
CREATE INDEX poll_account_idx ON poll(account_id);
CREATE INDEX poll_finish_height_idx ON poll(finish_height desc);
CREATE INDEX poll_height_idx ON poll(height);
CREATE TABLE IF NOT EXISTS poll_result
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 poll_id BIGINT NOT NULL,
 result BIGINT,
 weight BIGINT NOT NULL,
 height INTEGER NOT NULL
);
CREATE INDEX poll_result_poll_id_idx ON poll_result(poll_id);
CREATE INDEX poll_result_height_idx ON poll_result(height);
CREATE TABLE IF NOT EXISTS prunable_message
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 sender_id BIGINT NOT NULL,
 recipient_id BIGINT,
 message BLOB(1M),
 message_is_text SMALLINT NOT NULL,
 is_compressed SMALLINT NOT NULL,
 encrypted_message BLOB(1M),
 encrypted_is_text SMALLINT DEFAULT FALSE,
 block_timestamp INTEGER NOT NULL,
 transaction_timestamp INTEGER NOT NULL,
 height INTEGER NOT NULL,
 constraint constraint_b40 FOREIGN KEY (height) references block(height) ON DELETE CASCADE
);
CREATE UNIQUE INDEX prunable_message_id_idx ON prunable_message(id);
CREATE INDEX prunable_message_sender_idx ON prunable_message(sender_id);
CREATE INDEX prunable_message_recipient_idx ON prunable_message(recipient_id);
CREATE INDEX prunable_message_block_timestamp_dbid_idx ON prunable_message(block_timestamp desc, db_id desc);
CREATE INDEX prunable_message_transaction_timestamp_idx ON prunable_message(transaction_timestamp desc);
CREATE TABLE IF NOT EXISTS public_key
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 account_id BIGINT NOT NULL,
 public_key BLOB(1M),
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL,
 constraint constraint_8e8 FOREIGN KEY (height) references block(height) ON DELETE CASCADE
);
CREATE UNIQUE INDEX public_key_account_id_height_idx ON public_key(account_id, height desc);
CREATE TABLE IF NOT EXISTS purchase
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 buyer_id BIGINT NOT NULL,
 goods_id BIGINT NOT NULL,
 seller_id BIGINT NOT NULL,
 quantity INTEGER NOT NULL,
 price BIGINT NOT NULL,
 deadline INTEGER NOT NULL,
 note BLOB(1M),
 nonce BLOB(1M),
 timestamp INTEGER NOT NULL,
 pending SMALLINT NOT NULL,
 goods BLOB(1M),
 goods_nonce BLOB(1M),
 goods_is_text SMALLINT DEFAULT TRUE NOT NULL,
 refund_note BLOB(1M),
 refund_nonce BLOB(1M),
 has_feedback_notes SMALLINT DEFAULT FALSE NOT NULL,
 has_public_feedbacks SMALLINT DEFAULT FALSE NOT NULL,
 discount BIGINT NOT NULL,
 refund BIGINT NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX purchase_id_height_idx ON purchase(id, height desc);
CREATE INDEX purchase_buyer_id_height_idx ON purchase(buyer_id, height desc);
CREATE INDEX purchase_seller_id_height_idx ON purchase(seller_id, height desc);
CREATE INDEX purchase_deadline_idx ON purchase(deadline desc, height desc);
CREATE INDEX purchase_timestamp_idx ON purchase(timestamp desc, id);
CREATE INDEX purchase_height_id_idx ON purchase(height, id);
CREATE TABLE IF NOT EXISTS purchase_feedback
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 feedback_data BLOB(1M) NOT NULL,
 feedback_nonce BLOB(1M) NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE INDEX purchase_feedback_id_height_idx ON purchase_feedback(id, height desc);
CREATE INDEX purchase_feedback_height_id_idx ON purchase_feedback(height, id);
CREATE TABLE IF NOT EXISTS purchase_public_feedback
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 PUBLIC_feeDBACK varchar(5000) NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE INDEX purchase_public_feedback_id_height_idx ON purchase_public_feedback(id, height desc);
CREATE INDEX purchase_public_feedback_height_id_idx ON purchase_public_feedback(height, id);

CREATE TABLE IF NOT EXISTS scan
(
 rescan SMALLINT DEFAULT FALSE NOT NULL,
 height INTEGER DEFAULT 0 NOT NULL,
 validate SMALLINT DEFAULT FALSE NOT NULL
);
CREATE TABLE IF NOT EXISTS sell_offer
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 currency_id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 rate BIGINT NOT NULL,
 unit_limit BIGINT NOT NULL,
 supply BIGINT NOT NULL,
 expiration_height INTEGER NOT NULL,
 transaction_height INTEGER NOT NULL,
 creation_height INTEGER NOT NULL,
 transaction_index SMALLINT NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX sell_offer_id_idx ON sell_offer(id, height desc);
CREATE INDEX sell_offer_currency_id_account_id_idx ON sell_offer(currency_id, account_id, height desc);
CREATE INDEX sell_offer_rate_height_idx ON sell_offer(rate, creation_height);
CREATE INDEX sell_offer_height_id_idx ON sell_offer(height, id);
CREATE TABLE IF NOT EXISTS shuffling
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 holding_id BIGINT,
 holding_type SMALLINT NOT NULL,
 issuer_id BIGINT NOT NULL,
 amount BIGINT NOT NULL,
 participant_count SMALLINT NOT NULL,
 blocks_remaining SMALLINT,
 stage SMALLINT NOT NULL,
 assignee_account_id BIGINT,
 registrant_count SMALLINT NOT NULL,
 RECIPIENT_public_keyS varchar(5000),
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX shuffling_id_height_idx ON shuffling(id, height desc);
CREATE INDEX shuffling_holding_id_height_idx ON shuffling(holding_id, height desc);
CREATE INDEX shuffling_blocks_remaining_height_idx ON shuffling(blocks_remaining, height desc);
CREATE INDEX shuffling_assignee_account_id_height_idx ON shuffling(assignee_account_id, height desc);
CREATE INDEX shuffling_height_id_idx ON shuffling(height, id);
CREATE TABLE IF NOT EXISTS shuffling_data
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 shuffling_id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 data varchar(5000),
 transaction_timestamp INTEGER NOT NULL,
 height INTEGER NOT NULL,
 constraint constraint_a08 FOREIGN KEY (height) references block(height) ON DELETE CASCADE
);
CREATE UNIQUE INDEX shuffling_data_id_height_idx ON shuffling_data(shuffling_id, height desc);
CREATE INDEX shuffling_data_transaction_timestamp_idx ON shuffling_data(transaction_timestamp desc);
CREATE TABLE IF NOT EXISTS shuffling_participant
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 shuffling_id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 next_account_id BIGINT,
 participant_index SMALLINT NOT NULL,
 state SMALLINT NOT NULL,
 BLAME_data varchar(5000),
 key_seeds varchar(5000),
 data_transaction_full_hash BLOB(1M),
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX shuffling_participant_shuffling_id_account_id_idx ON shuffling_participant(shuffling_id, account_id, height desc);
CREATE INDEX shuffling_participant_height_idx ON shuffling_participant(height, shuffling_id, account_id);
CREATE TABLE IF NOT EXISTS tag
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 tag varchar(320) NOT NULL,
 in_stock_count INTEGER NOT NULL,
 total_count INTEGER NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX tag_tag_idx ON tag(tag, height desc);
CREATE INDEX tag_in_stock_count_idx ON tag(in_stock_count desc, height desc);
CREATE INDEX tag_height_tag_idx ON tag(height, tag);
CREATE TABLE IF NOT EXISTS tagged_data
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 name varchar(2000) NOT NULL,
 description varchar(5000),
 tags varchar(2000),
 parsed_tags varchar(2000),
 type_ varchar(2000),
 data BLOB(1M) NOT NULL,
 is_text SMALLINT NOT NULL,
 filename varchar(2000),
 channel varchar(300),
 block_timestamp INTEGER NOT NULL,
 transaction_timestamp INTEGER NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL,
 constraint constraint_8b9 FOREIGN KEY (height) references block(height) ON DELETE CASCADE
);
CREATE UNIQUE INDEX tagged_data_id_height_idx ON tagged_data(id, height desc);
CREATE INDEX tagged_data_account_id_height_idx ON tagged_data(account_id, height desc);
CREATE INDEX tagged_data_channel_idx ON tagged_data(channel, height desc);
CREATE INDEX tagged_data_block_timestamp_height_db_id_idx ON tagged_data(block_timestamp desc, height desc, db_id desc);
CREATE INDEX tagged_data_expiration_idx ON tagged_data(transaction_timestamp desc);
CREATE TABLE IF NOT EXISTS tagged_data_extend
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 extend_id BIGINT NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE INDEX tagged_data_extend_id_height_idx ON tagged_data_extend(id, height desc);
CREATE INDEX tagged_data_extend_height_id_idx ON tagged_data_extend(height, id);
CREATE TABLE IF NOT EXISTS tagged_data_timestamp
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 timestamp INTEGER NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);
CREATE UNIQUE INDEX tagged_data_timestamp_id_height_idx ON tagged_data_timestamp(id, height desc);
CREATE INDEX tagged_data_timestamp_height_id_idx ON tagged_data_timestamp(height, id);
CREATE TABLE IF NOT EXISTS trade
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 asset_id BIGINT NOT NULL,
 block_id BIGINT NOT NULL,
 ask_order_id BIGINT NOT NULL,
 bid_order_id BIGINT NOT NULL,
 ask_order_height INTEGER NOT NULL,
 bid_order_height INTEGER NOT NULL,
 seller_id BIGINT NOT NULL,
 buyer_id BIGINT NOT NULL,
 is_buy SMALLINT NOT NULL,
 quantity BIGINT NOT NULL,
 price BIGINT NOT NULL,
 timestamp INTEGER NOT NULL,
 height INTEGER NOT NULL
);
CREATE INDEX trade_asset_id_idx ON trade(asset_id, height desc);
CREATE INDEX trade_ask_idx ON trade(ask_order_id, height desc);
CREATE INDEX trade_bid_idx ON trade(bid_order_id, height desc);
CREATE INDEX trade_seller_id_idx ON trade(seller_id, height desc);
CREATE INDEX trade_buyer_id_idx ON trade(buyer_id, height desc);
CREATE INDEX trade_height_db_id_idx ON trade(height desc, db_id desc);
CREATE INDEX trade_height_idx ON trade(height);
CREATE TABLE IF NOT EXISTS transaction
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 deadline SMALLINT NOT NULL,
 recipient_id BIGINT,
 amount BIGINT NOT NULL,
 fee BIGINT NOT NULL,
 full_hash BLOB(1M) NOT NULL,
 height INTEGER NOT NULL,
 block_id BIGINT NOT NULL,
 signature BLOB(1M) NOT NULL,
 timestamp INTEGER NOT NULL,
 type SMALLINT NOT NULL,
 subtype SMALLINT NOT NULL,
 sender_id BIGINT NOT NULL,
 block_timestamp INTEGER NOT NULL,
 referenced_transaction_full_hash BLOB(1M),
 transaction_index SMALLINT NOT NULL,
 phased SMALLINT DEFAULT FALSE NOT NULL,
 attachment_bytes BLOB(1M),
 version SMALLINT NOT NULL,
 has_message SMALLINT DEFAULT FALSE NOT NULL,
 has_encrypted_message SMALLINT DEFAULT FALSE NOT NULL,
 has_public_key_announcement SMALLINT DEFAULT FALSE NOT NULL,
 has_prunable_message SMALLINT DEFAULT FALSE NOT NULL,
 has_prunable_attachment SMALLINT DEFAULT FALSE NOT NULL,
 ec_block_height INTEGER DEFAULT NULL,
 ec_block_id BIGINT DEFAULT NULL,
 has_encrypttoself_message SMALLINT DEFAULT FALSE NOT NULL,
 has_prunable_encrypted_message SMALLINT DEFAULT FALSE NOT NULL,
 constraint constraint_fff FOREIGN KEY (BLOCK_id) references block(id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX transaction_id_idx ON transaction(id);
CREATE INDEX transaction_recipient_id_idx ON transaction(recipient_id);
CREATE INDEX transaction_sender_id_idx ON transaction(sender_id);
CREATE INDEX transaction_block_timestamp_idx ON transaction(block_timestamp desc);
CREATE TABLE IF NOT EXISTS unconfirmed_transaction
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 expiration INTEGER NOT NULL,
 transaction_height INTEGER NOT NULL,
 fee_per_byte BIGINT NOT NULL,
 arrival_timestamp BIGINT NOT NULL,
 transaction_bytes BLOB(1M) NOT NULL,
 height INTEGER NOT NULL,
 prunable_json varchar(5000)
);
CREATE UNIQUE INDEX unconfirmed_transaction_id_idx ON unconfirmed_transaction(id);
CREATE INDEX unconfirmed_transaction_expiration_idx ON unconfirmed_transaction(expiration desc);
CREATE INDEX unconfirmed_transaction_height_fee_timestamp_idx ON unconfirmed_transaction(transaction_height, fee_per_byte desc, arrival_timestamp);

CREATE TABLE IF NOT EXISTS vote
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 poll_id BIGINT NOT NULL,
 voter_id BIGINT NOT NULL,
 vote_bytes BLOB(1M) NOT NULL,
 height INTEGER NOT NULL
);
CREATE UNIQUE INDEX vote_id_idx ON vote(id);
CREATE UNIQUE INDEX vote_poll_id_idx ON vote(poll_id, voter_id);
CREATE INDEX vote_height_idx ON vote(height);


CREATE TABLE IF NOT EXISTS referenced_transaction
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 transaction_id BIGINT NOT NULL,
 referenced_transaction_id BIGINT NOT NULL,
 constraint constraint_4b1 FOREIGN KEY (TRANSACTION_id) references transaction(id) ON DELETE CASCADE
);
CREATE INDEX referenced_transaction_referenced_transaction_id_idx ON referenced_transaction(referenced_transaction_id);


CREATE TABLE IF NOT EXISTS contract_reference
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 contract_name VARCHAR(5000) NOT NULL,
 contract_params VARCHAR(5000),
 contract_transaction_chain_id INTEGER NOT NULL,
 contract_transaction_full_hash BLOB(1M),
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);

CREATE INDEX contract_reference_height_id_idx
 ON contract_reference(height asc, id asc);

CREATE UNIQUE INDEX contract_reference_id_height_idx
 ON contract_reference(id asc, height desc);

CREATE INDEX contract_reference_account_height_idx
 ON contract_reference(account_id asc, height desc);

CREATE TABLE IF NOT EXISTS coin_order_fxt
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 chain_id INTEGER NOT NULL,
 exchange_id INTEGER NOT NULL,
 full_hash BLOB(1M) NOT NULL,
 amount BIGINT NOT NULL,
 quantity BIGINT NOT NULL,
 bid_price BIGINT NOT NULL,
 ask_price BIGINT NOT NULL,
 creation_height INTEGER NOT NULL,
 height INTEGER NOT NULL,
 transaction_height INTEGER NOT NULL,
 transaction_index SMALLINT NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);

CREATE INDEX coin_order_fxt_account_idx
 ON coin_order_fxt(account_id asc);

CREATE INDEX coin_order_fxt_chain_idx
 ON coin_order_fxt(chain_id asc, exchange_id asc);

CREATE INDEX coin_order_fxt_exchange_idx
 ON coin_order_fxt(exchange_id asc);

CREATE INDEX coin_order_fxt_id_idx
 ON coin_order_fxt(id asc, height desc);

CREATE TABLE IF NOT EXISTS coin_trade_fxt
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 chain_id INTEGER NOT NULL,
 exchange_id INTEGER NOT NULL,
 account_id BIGINT NOT NULL,
 block_id BIGINT NOT NULL,
 height INTEGER NOT NULL,
 timestamp INTEGER NOT NULL,
 exchange_quantity BIGINT NOT NULL,
 exchange_price BIGINT NOT NULL,
 order_id BIGINT NOT NULL,
 order_full_hash BLOB(1M) NOT NULL,
 match_id BIGINT NOT NULL,
 match_full_hash BLOB(1M) NOT NULL
);

CREATE INDEX coin_trade_fxt_exchange_idx
 ON coin_trade_fxt(exchange_id asc);

CREATE INDEX coin_trade_fxt_chain_idx
 ON coin_trade_fxt(chain_id asc);

CREATE INDEX coin_trade_fxt_account_idx
 ON coin_trade_fxt(account_id asc);

CREATE INDEX coin_trade_fxt_order_idx
 ON coin_trade_fxt(order_id asc);

CREATE TABLE IF NOT EXISTS balance
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 account_id BIGINT NOT NULL,
 balance BIGINT NOT NULL,
 unconfirmed_balance BIGINT NOT NULL,
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);

CREATE UNIQUE INDEX balance_fxt_id_height_idx
 ON balance(account_id asc, height desc);

CREATE INDEX balance_fxt_height_id_idx
 ON balance(height asc, account_id asc);



CREATE TABLE IF NOT EXISTS asset_property
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 asset_id BIGINT NOT NULL,
 setter_id BIGINT NOT NULL,
 property VARCHAR(320) NOT NULL,
 value VARCHAR(5000),
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);


CREATE INDEX asset_property_asset_height_idx
 ON asset_property(asset_id asc, height desc);

CREATE INDEX asset_property_height_id_idx
 ON asset_property(height asc, id asc);

CREATE UNIQUE INDEX asset_property_id_height_idx
 ON asset_property(id asc, height desc);

CREATE INDEX asset_property_setter_property_idx
 ON asset_property(setter_id asc, property asc);


CREATE TABLE IF NOT EXISTS asset_history
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 id BIGINT NOT NULL,
 full_hash BLOB(1M) NOT NULL,
 asset_id BIGINT NOT NULL,
 account_id BIGINT NOT NULL,
 quantity BIGINT NOT NULL,
 timestamp INTEGER NOT NULL,
 chain_id INTEGER NOT NULL,
 height INTEGER NOT NULL
);

CREATE INDEX asset_history_account_id_idx
 ON asset_history(account_id asc, height desc);

CREATE INDEX asset_history_asset_id_idx
 ON asset_history(asset_id asc, height desc);

CREATE INDEX asset_history_height_idx
 ON asset_history(height asc);

CREATE INDEX asset_history_id_idx
 ON asset_history(id asc);

CREATE TABLE IF NOT EXISTS asset_control_phasing
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 asset_id BIGINT NOT NULL,
 voting_model SMALLINT NOT NULL,
 quorum BIGINT,
 min_balance BIGINT,
 holding_id BIGINT,
 min_balance_model SMALLINT,
 whitelist VARCHAR(5000),
 expression TEXT,
 sender_property_setter_id BIGINT,
 sender_property_name VARCHAR(1000),
 sender_property_value VARCHAR(1000),
 recipient_property_setter_id BIGINT,
 recipient_property_name VARCHAR(1000),
 recipient_property_value VARCHAR(1000),
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);


CREATE INDEX asset_control_phasing_id_height_idx
 ON asset_control_phasing(asset_id asc, height desc);

CREATE INDEX asset_control_phasing_height_id_idx
 ON asset_control_phasing(height asc, asset_id asc);


CREATE TABLE IF NOT EXISTS asset_control_phasing_sub_poll
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 asset_id BIGINT NOT NULL,
 name VARCHAR(1000),
 voting_model SMALLINT NOT NULL,
 quorum BIGINT,
 min_balance BIGINT,
 holding_id BIGINT,
 min_balance_model SMALLINT,
 whitelist VARCHAR(5000),
 sender_property_setter_id BIGINT,
 sender_property_name VARCHAR(1000),
 sender_property_value VARCHAR(1000),
 recipient_property_setter_id BIGINT,
 recipient_property_name VARCHAR(1000),
 recipient_property_value VARCHAR(1000),
 height INTEGER NOT NULL,
 latest SMALLINT DEFAULT TRUE NOT NULL
);


CREATE INDEX asset_control_phasing_sub_poll_id_height_idx
 ON asset_control_phasing_sub_poll(asset_id asc, height desc);

CREATE INDEX asset_control_phasing_sub_poll_height_id_idx
 ON asset_control_phasing_sub_poll(height asc, asset_id asc);


CREATE TABLE IF NOT EXISTS phasing_poll_hashed_secret
(
 db_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
 hashed_secret BLOB(1M) NOT NULL,
 hashed_secret_id BIGINT NOT NULL,
 algorithm SMALLINT NOT NULL,
 transaction_full_hash BLOB(1M),
 transaction_id BIGINT NOT NULL,
 chain_id INTEGER NOT NULL,
 finish_height INTEGER NOT NULL,
 height INTEGER NOT NULL
);


CREATE INDEX phasing_poll_hashed_secret_id_ix
 ON phasing_poll_hashed_secret(hashed_secret_id asc);

CREATE INDEX phasing_poll_hashed_secret_height_idx
 ON phasing_poll_hashed_secret(height asc);

CREATE INDEX phasing_poll_hashed_secret_transaction_id_idx
 ON phasing_poll_hashed_secret(transaction_id asc);


/* 如果是历史库需要增加此字段
alter table asset add column has_control_phasing SMALLINT DEFAULT FALSE NOT NULL
*/
/*
grant all privileges on *.* to 'nrcs'@'localhost'
*/
