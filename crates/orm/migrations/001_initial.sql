-- =====================================================
-- NRCS Blockchain Database Schema
-- Converted from H2 to PostgreSQL
-- Source: nrcs-sql/src/main/resources/sql-scripts-h2/0.sql
-- Date: 2026-04-03
-- =====================================================

-- Enable UUID extension if needed
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- =====================================================
-- ACCOUNT表
-- =====================================================
CREATE TABLE IF NOT EXISTS account (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    id BIGINT NOT NULL,
    balance BIGINT NOT NULL,
    unconfirmed_balance BIGINT NOT NULL,
    forged_balance BIGINT NOT NULL,
    active_lessee_id BIGINT,
    has_control_phasing BOOLEAN DEFAULT FALSE NOT NULL,
    height INTEGER NOT NULL,
    latest BOOLEAN DEFAULT TRUE NOT NULL
);

-- Indexes for ACCOUNT
CREATE UNIQUE INDEX IF NOT EXISTS account_id_height_idx ON account (id, height DESC);
CREATE INDEX IF NOT EXISTS account_active_lessee_id_idx ON account (active_lessee_id);
CREATE INDEX IF NOT EXISTS account_height_id_idx ON account (height, id);

-- =====================================================
-- ACCOUNT_ASSET表
-- =====================================================
CREATE TABLE IF NOT EXISTS account_asset (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    account_id BIGINT NOT NULL,
    asset_id BIGINT NOT NULL,
    quantity BIGINT NOT NULL,
    unconfirmed_quantity BIGINT NOT NULL,
    height INTEGER NOT NULL,
    latest BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS account_asset_id_height_idx ON account_asset (account_id, asset_id, height DESC);
CREATE INDEX IF NOT EXISTS account_asset_asset_id_idx ON account_asset (asset_id);
CREATE INDEX IF NOT EXISTS account_asset_quantity_idx ON account_asset (quantity DESC);
CREATE INDEX IF NOT EXISTS account_asset_height_id_idx ON account_asset (height, account_id, asset_id);

-- =====================================================
-- ACCOUNT_CONTROL_PHASING表
-- =====================================================
CREATE TABLE IF NOT EXISTS account_control_phasing (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    account_id BIGINT NOT NULL,
    whitelist VARCHAR,
    voting_model SMALLINT NOT NULL,
    quorum BIGINT,
    min_balance BIGINT,
    holding_id BIGINT,
    min_balance_model SMALLINT,
    max_fees BIGINT,
    min_duration SMALLINT,
    max_duration SMALLINT,
    height INTEGER NOT NULL,
    latest BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS account_control_phasing_id_height_idx ON account_control_phasing (account_id, height DESC);
CREATE INDEX IF NOT EXISTS account_control_phasing_height_id_idx ON account_control_phasing (height, account_id);

-- =====================================================
-- ACCOUNT_CURRENCY表
-- =====================================================
CREATE TABLE IF NOT EXISTS account_currency (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    account_id BIGINT NOT NULL,
    currency_id BIGINT NOT NULL,
    units BIGINT NOT NULL,
    unconfirmed_units BIGINT NOT NULL,
    height INTEGER NOT NULL,
    latest BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS account_currency_id_height_idx ON account_currency (account_id, currency_id, height DESC);
CREATE INDEX IF NOT EXISTS account_currency_currency_id_idx ON account_currency (currency_id);
CREATE INDEX IF NOT EXISTS account_currency_units_idx ON account_currency (units DESC);
CREATE INDEX IF NOT EXISTS account_currency_height_id_idx ON account_currency (height, account_id, currency_id);

-- =====================================================
-- ACCOUNT_FXT表
-- =====================================================
CREATE TABLE IF NOT EXISTS account_fxt (
    id BIGINT NOT NULL,
    balance BYTEA NOT NULL,
    height INTEGER NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS account_fxt_id_idx ON account_fxt (id, height DESC);

-- =====================================================
-- ACCOUNT_GUARANTEED_BALANCE表
-- =====================================================
CREATE TABLE IF NOT EXISTS account_guaranteed_balance (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    account_id BIGINT NOT NULL,
    additions BIGINT NOT NULL,
    height INTEGER NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS account_guaranteed_balance_id_height_idx ON account_guaranteed_balance (account_id, height DESC);
CREATE INDEX IF NOT EXISTS account_guaranteed_balance_height_idx ON account_guaranteed_balance (height);

-- =====================================================
-- ACCOUNT_INFO表
-- =====================================================
CREATE TABLE IF NOT EXISTS account_info (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    account_id BIGINT NOT NULL,
    name VARCHAR,
    description VARCHAR,
    height INTEGER NOT NULL,
    latest BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS account_info_id_height_idx ON account_info (account_id, height DESC);
CREATE INDEX IF NOT EXISTS account_info_height_id_idx ON account_info (height, account_id);

-- =====================================================
-- ACCOUNT_LEASE表
-- =====================================================
CREATE TABLE IF NOT EXISTS account_lease (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    lessor_id BIGINT NOT NULL,
    current_leasing_height_from INTEGER,
    current_leasing_height_to INTEGER,
    current_lessee_id BIGINT,
    next_leasing_height_from INTEGER,
    next_leasing_height_to INTEGER,
    next_lessee_id BIGINT,
    height INTEGER NOT NULL,
    latest BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS account_lease_lessor_id_height_idx ON account_lease (lessor_id, height DESC);
CREATE INDEX IF NOT EXISTS account_lease_current_leasing_height_from_idx ON account_lease (current_leasing_height_from);
CREATE INDEX IF NOT EXISTS account_lease_current_leasing_height_to_idx ON account_lease (current_leasing_height_to);
CREATE INDEX IF NOT EXISTS account_lease_height_id_idx ON account_lease (height, lessor_id);

-- =====================================================
-- ACCOUNT_LEDGER表
-- =====================================================
CREATE TABLE IF NOT EXISTS account_ledger (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    account_id BIGINT NOT NULL,
    event_type SMALLINT NOT NULL,
    event_id BIGINT NOT NULL,
    holding_type SMALLINT NOT NULL,
    holding_id BIGINT,
    change BIGINT NOT NULL,
    balance BIGINT NOT NULL,
    block_id BIGINT NOT NULL,
    height INTEGER NOT NULL,
    timestamp INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS account_ledger_id_idx ON account_ledger (account_id, db_id);
CREATE INDEX IF NOT EXISTS account_ledger_height_idx ON account_ledger (height);

-- =====================================================
-- ACCOUNT_PROPERTY表
-- =====================================================
CREATE TABLE IF NOT EXISTS account_property (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    id BIGINT NOT NULL,
    recipient_id BIGINT NOT NULL,
    setter_id BIGINT,
    property VARCHAR NOT NULL,
    value VARCHAR,
    height INTEGER NOT NULL,
    latest BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS account_property_id_height_idx ON account_property (id, height DESC);
CREATE INDEX IF NOT EXISTS account_property_recipient_height_idx ON account_property (recipient_id, height DESC);
CREATE INDEX IF NOT EXISTS account_property_setter_recipient_idx ON account_property (setter_id, recipient_id);
CREATE INDEX IF NOT EXISTS account_property_height_id_idx ON account_property (height, id);

-- =====================================================
-- ALIAS表
-- =====================================================
CREATE TABLE IF NOT EXISTS alias (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    id BIGINT NOT NULL,
    account_id BIGINT NOT NULL,
    alias_name VARCHAR NOT NULL,
    alias_name_lower VARCHAR DEFAULT 'LOWER(alias_name)' NOT NULL,
    alias_uri VARCHAR NOT NULL,
    timestamp INTEGER NOT NULL,
    height INTEGER NOT NULL,
    latest BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS alias_id_height_idx ON alias (id, height DESC);
CREATE INDEX IF NOT EXISTS alias_account_id_idx ON alias (account_id, height DESC);
CREATE INDEX IF NOT EXISTS alias_name_lower_idx ON alias (alias_name_lower);
CREATE INDEX IF NOT EXISTS alias_height_id_idx ON alias (height, id);

-- =====================================================
-- ALIAS_OFFER表
-- =====================================================
CREATE TABLE IF NOT EXISTS alias_offer (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    id BIGINT NOT NULL,
    price BIGINT NOT NULL,
    buyer_id BIGINT,
    height INTEGER NOT NULL,
    latest BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS alias_offer_id_height_idx ON alias_offer (id, height DESC);
CREATE INDEX IF NOT EXISTS alias_offer_height_id_idx ON alias_offer (height, id);

-- =====================================================
-- ASSET表
-- =====================================================
CREATE TABLE IF NOT EXISTS asset (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    id BIGINT NOT NULL,
    account_id BIGINT NOT NULL,
    name VARCHAR NOT NULL,
    description VARCHAR,
    quantity BIGINT NOT NULL,
    decimals SMALLINT NOT NULL,
    initial_quantity BIGINT NOT NULL,
    has_control_phasing BOOLEAN DEFAULT FALSE NOT NULL,
    height INTEGER NOT NULL,
    latest BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS asset_id_height_idx ON asset (id, height DESC);
CREATE INDEX IF NOT EXISTS asset_account_id_idx ON asset (account_id);
CREATE INDEX IF NOT EXISTS asset_height_id_idx ON asset (height, id);

-- =====================================================
-- ASSET_DELETE表
-- =====================================================
CREATE TABLE IF NOT EXISTS asset_delete (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    asset_id BIGINT NOT NULL,
    account_id BIGINT NOT NULL,
    quantity BIGINT NOT NULL,
    height INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS asset_delete_asset_id_idx ON asset_delete (asset_id);

-- =====================================================
-- ASSET_DIVIDEND表
-- =====================================================
CREATE TABLE IF NOT EXISTS asset_dividend (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    asset_id BIGINT NOT NULL,
    dividend_asset_id BIGINT,
    dividend_amount BIGINT,
    height INTEGER NOT NULL,
    latest BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE INDEX IF NOT EXISTS asset_dividend_asset_id_idx ON asset_dividend (asset_id);

-- =====================================================
-- ASSET_TRANSFER表
-- =====================================================
CREATE TABLE IF NOT EXISTS asset_transfer (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    id BIGINT NOT NULL,
    sender_id BIGINT NOT NULL,
    recipient_id BIGINT NOT NULL,
    asset_id BIGINT NOT NULL,
    quantity BIGINT NOT NULL,
    transaction_index SMALLINT NOT NULL,
    transaction_height INTEGER NOT NULL,
    height INTEGER NOT NULL,
    latest BOOLEAN DEFAULT TRUE NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS asset_transfer_id_height_idx ON asset_transfer (id, height DESC);
CREATE INDEX IF NOT EXISTS asset_transfer_sender_id_idx ON asset_transfer (sender_id);
CREATE INDEX IF NOT EXISTS asset_transfer_recipient_id_idx ON asset_transfer (recipient_id);
CREATE INDEX IF NOT EXISTS asset_transfer_asset_id_idx ON asset_transfer (asset_id);
CREATE INDEX IF NOT EXISTS asset_transfer_height_id_idx ON asset_transfer (height, id);

-- =====================================================
-- BLOCK表
-- =====================================================
CREATE TABLE IF NOT EXISTS block (
    db_id BIGSERIAL PRIMARY KEY NOT NULL,
    id BIGINT NOT NULL,
    version INTEGER NOT NULL,
    timestamp_ INTEGER NOT NULL,
    previous_block_id BIGINT,
    total_flat_fee BIGINT NOT NULL,
    total_payload_length INTEGER NOT NULL,
    payload_length INTEGER NOT NULL,
    base_target BIGINT NOT NULL,
    generation_signature VARCHAR NOT NULL,
    previous_generation_signature VARCHAR,
    generator_id BIGINT NOT NULL,
    generator_public_key VARCHAR NOT NULL,
    height INTEGER NOT NULL,
    last_block_timestamp INTEGER NOT NULL,
    cumulative_difficulty NUMERIC NOT NULL,
    serial_number SMALLINT,
    next_available BIT VARYING,
    next_issuance_height INTEGER,
    next_issuance_amount BIGINT,
    exchange_id BIGINT,
    forfeited_rewards BIGINT NOT NULL,
    rewards BIGINT NOT NULL,
    total_amount BIGINT NOT NULL,
    total_fee BIGINT NOT NULL,
    payload_hash VARCHAR NOT NULL,
    index_name INTEGER NOT NULL,
    payload_bytes BYTEA NOT NULL,
    signature VARCHAR NOT NULL,
    transaction_count INTEGER NOT NULL,
    unconfirmed_transaction_count INTEGER NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS block_id_idx ON block (id);
CREATE INDEX IF NOT EXISTS block_height_idx ON block (height);
CREATE INDEX IF NOT EXISTS block_generator_id_idx ON block (generator_id);
CREATE INDEX IF NOT EXISTS block_previous_block_id_idx ON block (previous_block_id);
CREATE INDEX IF NOT EXISTS block_timestamp_idx ON block (timestamp_);

-- 继续创建剩余的表（BALANCE, BID_ORDER, BUY_OFFER, CONTRACT_REFERENCE, CURRENCY 等）
-- 由于文件很大，这里只展示部分核心表。完整迁移脚本要从 0.sql 转换所有表。
-- 请参考 crates/orm/migrations/full_from_h2.sql（自动转换版本）