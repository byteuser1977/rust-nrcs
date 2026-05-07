//! CLI 命令实现

use clap::Args;
use std::io::{self, BufRead, Write};

#[allow(dead_code)]
pub struct KeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

#[derive(Args)]
pub struct GenerateKeypairArgs {
    #[arg(short, long, help = "Secret phrase (will prompt if not provided)")]
    pub passphrase: Option<String>,

    #[arg(short, long, help = "Output format (hex, json)")]
    pub format: Option<String>,
}

#[derive(Args)]
pub struct GeneratePassphraseArgs {
    #[arg(short, long, help = "Number of words (12 or 24)")]
    pub words: Option<usize>,
}

#[derive(Args)]
pub struct SignTransactionArgs {
    #[arg(short, long, help = "Input file containing unsigned transaction bytes")]
    pub input: String,

    #[arg(short, long, help = "Output file for signed transaction bytes")]
    pub output: String,

    #[arg(short, long, help = "Secret phrase (will prompt if not provided)")]
    pub passphrase: Option<String>,
}

#[derive(Args)]
pub struct VerifyAddressArgs {
    #[arg(short, long, help = "Account address to verify")]
    pub address: String,
}

#[derive(Args)]
pub struct ConvertHexArgs {
    #[arg(short, long, help = "Hex string to convert")]
    pub hex: String,

    #[arg(short, long, help = "Convert to (bytes, decimal)")]
    pub to: Option<String>,
}

// ============================================================================
// ExportConstants
// ============================================================================

/// Export blockchain constants to stdout
#[derive(Args)]
pub struct ExportConstantsArgs {
    #[arg(short, long, help = "Output format (text, json)")]
    pub format: Option<String>,
}

// ============================================================================
// CompactDatabase
// ============================================================================

/// Compact and reorganize the blockchain database
#[derive(Args)]
pub struct CompactDatabaseArgs {
    #[arg(short, long, help = "Database directory path")]
    pub db_dir: Option<String>,

    #[arg(short, long, help = "Database URL (jdbc-style or sqlite path)")]
    pub db_url: Option<String>,
}

// ============================================================================
// RecoverPassphrase
// ============================================================================

/// Recover a passphrase from partial information via brute-force
#[derive(Args)]
pub struct RecoverPassphraseArgs {
    /// Approximate passphrase with unknown chars (use wildcard/replacement chars)
    #[arg(short, long, help = "Approximate passphrase with unknown characters")]
    pub wildcard: String,

    /// Comma-separated positions of unknown characters (1-indexed)
    #[arg(short, long, help = "Positions of unknown chars (1-indexed, comma-separated)")]
    pub positions: Option<String>,

    /// Dictionary to use: ascii, asciiall, unicode, or a custom character set string
    #[arg(short, long, help = "Dictionary: ascii, asciiall, unicode, or custom chars")]
    pub dictionary: Option<String>,

    /// Optional: comma-separated hex-encoded known public keys to match against
    #[arg(short = 'k', long, help = "Known public keys (hex, comma-separated) to match against")]
    pub known_keys: Option<String>,

    /// Optional: comma-separated known account IDs to match against
    #[arg(short = 'a', long, help = "Known account IDs (comma-separated) to match against")]
    pub known_accounts: Option<String>,
}

// ============================================================================
// BaseTargetTest
// ============================================================================

/// Test base target calculation with given parameters
#[derive(Args)]
pub struct BaseTargetTestArgs {
    /// Previous base target value
    #[arg(short, long, help = "Previous base target")]
    pub previous_base_target: u64,

    /// Block time EMA (exponential moving average) in seconds
    #[arg(short, long, help = "Block time EMA in seconds")]
    pub blocktime_ema: u64,

    /// Use testnet max base target (default: mainnet)
    #[arg(short, long, help = "Use testnet parameters")]
    pub testnet: bool,
}

pub async fn generate_keypair(args: GenerateKeypairArgs) -> anyhow::Result<()> {
    let passphrase = match args.passphrase {
        Some(p) => p,
        None => {
            print!("Enter secret phrase: ");
            io::stdout().flush()?;
            let stdin = io::stdin();
            let mut input = String::new();
            stdin.lock().read_line(&mut input)?;
            input.trim().to_string()
        }
    };
    
    if passphrase.is_empty() {
        anyhow::bail!("Passphrase cannot be empty");
    }
    
    let keypair = crypto::generate_keypair_from_passphrase(&passphrase)?;
    
    let public_key = keypair.public_key();
    let secret_key = keypair.secret_key();
    
    let public_key_hex = match public_key {
        crypto::PublicKey::Ed25519(bytes) => hex::encode(bytes),
        crypto::PublicKey::Curve25519(bytes) => hex::encode(bytes),
        crypto::PublicKey::Sm2 { public_key, .. } => hex::encode(public_key),
    };
    
    let secret_key_hex = match secret_key {
        crypto::SecretKey::Ed25519(bytes) => hex::encode(bytes),
        crypto::SecretKey::Curve25519(bytes) => hex::encode(bytes),
        crypto::SecretKey::Sm2 { secret_key, .. } => hex::encode(secret_key),
    };
    
    match args.format.as_deref() {
        Some("json") => {
            let json = serde_json::json!({
                "publicKey": public_key_hex,
                "secretKey": secret_key_hex,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            println!("Public Key: {}", public_key_hex);
            println!("Secret Key: {}", secret_key_hex);
        }
    }
    
    Ok(())
}

pub async fn generate_passphrase(args: GeneratePassphraseArgs) -> anyhow::Result<()> {
    let word_count = args.words.unwrap_or(12);
    
    if word_count != 12 {
        anyhow::bail!("NRCS only supports 12-word passphrases");
    }
    
    let passphrase = crypto::generate_passphrase()?;
    
    let keypair = crypto::passphrase_to_keypair(&passphrase)?;
    let public_key = keypair.public_key();
    
    let public_key_hex = match public_key {
        crypto::PublicKey::Ed25519(bytes) => hex::encode(bytes),
        crypto::PublicKey::Curve25519(bytes) => hex::encode(bytes),
        crypto::PublicKey::Sm2 { public_key, .. } => hex::encode(public_key),
    };
    
    println!("Generated Passphrase:");
    println!("====================");
    println!("Passphrase: {}", passphrase);
    println!("Public Key: {}", public_key_hex);
    
    Ok(())
}

pub async fn sign_transaction(args: SignTransactionArgs) -> anyhow::Result<()> {
    let passphrase = match args.passphrase {
        Some(p) => p,
        None => {
            rpassword::prompt_password("Enter secret phrase: ")?
        }
    };
    
    if passphrase.is_empty() {
        anyhow::bail!("Passphrase cannot be empty");
    }
    
    let input_content = std::fs::read_to_string(&args.input)?;
    let output_path = std::path::Path::new(&args.output);
    
    if output_path.exists() {
        anyhow::bail!("Output file already exists: {}", args.output);
    }
    
    let mut signed_count = 0;
    let mut output_lines = Vec::new();
    
    for line in input_content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        let tx_bytes = hex::decode(line)?;
        let signed_bytes = crypto::sign_transaction_bytes(&tx_bytes, &passphrase)?;
        output_lines.push(hex::encode(&signed_bytes));
        signed_count += 1;
    }
    
    std::fs::write(&args.output, output_lines.join("\n"))?;
    
    println!("Signed {} transactions", signed_count);
    println!("Output written to: {}", args.output);
    
    Ok(())
}

pub async fn verify_address(args: VerifyAddressArgs) -> anyhow::Result<()> {
    let address = args.address.trim();
    
    if !address.starts_with("NRCS-") && !address.starts_with("NRCSTEST-") {
        anyhow::bail!("Invalid address format. Must start with NRCS- or NRCSTEST-");
    }
    
    match crypto::validate_account_address(address) {
        Ok(account_id) => {
            println!("Address: {}", address);
            println!("Valid: true");
            println!("Account ID: {}", account_id);
        }
        Err(e) => {
            println!("Address: {}", address);
            println!("Valid: false");
            println!("Error: {}", e);
        }
    }
    
    Ok(())
}

pub async fn convert_hex(args: ConvertHexArgs) -> anyhow::Result<()> {
    let bytes = hex::decode(&args.hex)?;

    match args.to.as_deref() {
        Some("decimal") => {
            let mut result = 0u64;
            for (i, byte) in bytes.iter().enumerate() {
                result |= (*byte as u64) << (i * 8);
            }
            println!("Decimal: {}", result);
        }
        Some("bytes") | None => {
            println!("Bytes: {:?}", bytes);
            println!("Length: {} bytes", bytes.len());
        }
        _ => {
            anyhow::bail!("Unknown conversion target. Use 'bytes' or 'decimal'");
        }
    }

    Ok(())
}

// ============================================================================
// ExportConstants
// ============================================================================

pub async fn export_constants(args: ExportConstantsArgs) -> anyhow::Result<()> {
    use blockchain_types::constants::*;

    let constants: Vec<(&str, String)> = vec![
        ("VERSION", VERSION.to_string()),
        ("APPLICATION", APPLICATION.to_string()),
        ("CHAIN", CHAIN.to_string()),
        ("BLOCK_VERSION", BLOCK_VERSION.to_string()),
        ("TRANSACTION_VERSION", TRANSACTION_VERSION.to_string()),
        ("BLOCK_TIME (seconds)", BLOCK_TIME.to_string()),
        ("MAX_NUMBER_OF_TRANSACTIONS", MAX_NUMBER_OF_TRANSACTIONS.to_string()),
        ("MIN_TRANSACTION_SIZE (bytes)", MIN_TRANSACTION_SIZE.to_string()),
        ("MAX_PAYLOAD_LENGTH (bytes)", MAX_PAYLOAD_LENGTH.to_string()),
        ("MAX_BALANCE_NRCS", MAX_BALANCE_NRCS.to_string()),
        ("ONE_NRCS (NQT)", ONE_NRCS.to_string()),
        ("ONE_FXT", ONE_FXT.to_string()),
        ("MAX_BALANCE_NQT", MAX_BALANCE_NQT.to_string()),
        ("MAX_BALANCE_FXT", MAX_BALANCE_FXT.to_string()),
        ("GENESIS_BLOCK_ID", GENESIS_BLOCK_ID.to_string()),
        ("GENESIS_CREATOR_PUBLIC_KEY", hex::encode(GENESIS_CREATOR_PUBLIC_KEY)),
        ("", String::new()),
        ("INITIAL_BASE_TARGET", INITIAL_BASE_TARGET.to_string()),
        ("MAX_BASE_TARGET", MAX_BASE_TARGET.to_string()),
        ("MIN_BASE_TARGET", MIN_BASE_TARGET.to_string()),
        ("MIN_BLOCKTIME_LIMIT", MIN_BLOCKTIME_LIMIT.to_string()),
        ("MAX_BLOCKTIME_LIMIT", MAX_BLOCKTIME_LIMIT.to_string()),
        ("BASE_TARGET_GAMMA", BASE_TARGET_GAMMA.to_string()),
        ("MIN_FORGING_BALANCE_NQT", MIN_FORGING_BALANCE_NQT.to_string()),
        ("", String::new()),
        ("MAX_TIMEDRIFT", MAX_TIMEDRIFT.to_string()),
        ("MAX_PHASING_VOTE_TRANSACTIONS", MAX_PHASING_VOTE_TRANSACTIONS.to_string()),
        ("MAX_PHASING_WHITELIST_SIZE", MAX_PHASING_WHITELIST_SIZE.to_string()),
        ("MAX_PHASING_LINKED_TRANSACTIONS", MAX_PHASING_LINKED_TRANSACTIONS.to_string()),
        ("MAX_PHASING_DURATION", MAX_PHASING_DURATION.to_string()),
        ("MAX_PHASING_REVEALED_SECRETS_COUNT", MAX_PHASING_REVEALED_SECRETS_COUNT.to_string()),
        ("MAX_PHASING_REVEALED_SECRET_LENGTH", MAX_PHASING_REVEALED_SECRET_LENGTH.to_string()),
        ("", String::new()),
        ("MAX_ALIAS_URI_LENGTH", MAX_ALIAS_URI_LENGTH.to_string()),
        ("MAX_ALIAS_LENGTH", MAX_ALIAS_LENGTH.to_string()),
        ("MAX_ARBITRARY_MESSAGE_LENGTH", MAX_ARBITRARY_MESSAGE_LENGTH.to_string()),
        ("MAX_ENCRYPTED_MESSAGE_LENGTH", MAX_ENCRYPTED_MESSAGE_LENGTH.to_string()),
        ("MAX_PRUNABLE_MESSAGE_LENGTH", MAX_PRUNABLE_MESSAGE_LENGTH.to_string()),
        ("MAX_PRUNABLE_ENCRYPTED_MESSAGE_LENGTH", MAX_PRUNABLE_ENCRYPTED_MESSAGE_LENGTH.to_string()),
        ("", String::new()),
        ("MAX_ACCOUNT_NAME_LENGTH", MAX_ACCOUNT_NAME_LENGTH.to_string()),
        ("MAX_ACCOUNT_DESCRIPTION_LENGTH", MAX_ACCOUNT_DESCRIPTION_LENGTH.to_string()),
        ("", String::new()),
        ("MAX_ASSET_QUANTITY_QNT", MAX_ASSET_QUANTITY_QNT.to_string()),
        ("MIN_ASSET_NAME_LENGTH", MIN_ASSET_NAME_LENGTH.to_string()),
        ("MAX_ASSET_NAME_LENGTH", MAX_ASSET_NAME_LENGTH.to_string()),
        ("MAX_ASSET_DESCRIPTION_LENGTH", MAX_ASSET_DESCRIPTION_LENGTH.to_string()),
        ("MAX_SINGLETON_ASSET_DESCRIPTION_LENGTH", MAX_SINGLETON_ASSET_DESCRIPTION_LENGTH.to_string()),
        ("MAX_DIVIDEND_PAYMENT_ROLLBACK", MAX_DIVIDEND_PAYMENT_ROLLBACK.to_string()),
        ("", String::new()),
        ("MAX_POLL_NAME_LENGTH", MAX_POLL_NAME_LENGTH.to_string()),
        ("MAX_POLL_DESCRIPTION_LENGTH", MAX_POLL_DESCRIPTION_LENGTH.to_string()),
        ("MAX_POLL_OPTION_LENGTH", MAX_POLL_OPTION_LENGTH.to_string()),
        ("MAX_POLL_OPTION_COUNT", MAX_POLL_OPTION_COUNT.to_string()),
        ("MAX_POLL_DURATION", MAX_POLL_DURATION.to_string()),
        ("MIN_VOTE_VALUE", MIN_VOTE_VALUE.to_string()),
        ("MAX_VOTE_VALUE", MAX_VOTE_VALUE.to_string()),
        ("", String::new()),
        ("MAX_DGS_LISTING_QUANTITY", MAX_DGS_LISTING_QUANTITY.to_string()),
        ("MAX_DGS_LISTING_NAME_LENGTH", MAX_DGS_LISTING_NAME_LENGTH.to_string()),
        ("MAX_DGS_LISTING_DESCRIPTION_LENGTH", MAX_DGS_LISTING_DESCRIPTION_LENGTH.to_string()),
        ("", String::new()),
        ("MAX_HUB_ANNOUNCEMENT_URIS", MAX_HUB_ANNOUNCEMENT_URIS.to_string()),
        ("MAX_HUB_ANNOUNCEMENT_URI_LENGTH", MAX_HUB_ANNOUNCEMENT_URI_LENGTH.to_string()),
        ("MIN_HUB_EFFECTIVE_BALANCE", MIN_HUB_EFFECTIVE_BALANCE.to_string()),
        ("", String::new()),
        ("MIN_CURRENCY_NAME_LENGTH", MIN_CURRENCY_NAME_LENGTH.to_string()),
        ("MAX_CURRENCY_NAME_LENGTH", MAX_CURRENCY_NAME_LENGTH.to_string()),
        ("MIN_CURRENCY_CODE_LENGTH", MIN_CURRENCY_CODE_LENGTH.to_string()),
        ("MAX_CURRENCY_CODE_LENGTH", MAX_CURRENCY_CODE_LENGTH.to_string()),
        ("MAX_CURRENCY_TOTAL_SUPPLY", MAX_CURRENCY_TOTAL_SUPPLY.to_string()),
        ("CURRENCY_MINT_FEE (NQT)", CURRENCY_MINT_FEE.to_string()),
        ("", String::new()),
        ("MIN_NUMBER_OF_SHUFFLING_PARTICIPANTS", MIN_NUMBER_OF_SHUFFLING_PARTICIPANTS.to_string()),
        ("MAX_NUMBER_OF_SHUFFLING_PARTICIPANTS", MAX_NUMBER_OF_SHUFFLING_PARTICIPANTS.to_string()),
        ("MAX_SHUFFLING_REGISTRATION_PERIOD", MAX_SHUFFLING_REGISTRATION_PERIOD.to_string()),
        ("", String::new()),
        ("MAX_TAGGED_DATA_NAME_LENGTH", MAX_TAGGED_DATA_NAME_LENGTH.to_string()),
        ("MAX_TAGGED_DATA_DESCRIPTION_LENGTH", MAX_TAGGED_DATA_DESCRIPTION_LENGTH.to_string()),
        ("MAX_TAGGED_DATA_TAGS_LENGTH", MAX_TAGGED_DATA_TAGS_LENGTH.to_string()),
        ("MAX_TAGGED_DATA_TYPE_LENGTH", MAX_TAGGED_DATA_TYPE_LENGTH.to_string()),
        ("MAX_TAGGED_DATA_CHANNEL_LENGTH", MAX_TAGGED_DATA_CHANNEL_LENGTH.to_string()),
        ("MAX_TAGGED_DATA_FILENAME_LENGTH", MAX_TAGGED_DATA_FILENAME_LENGTH.to_string()),
        ("MAX_TAGGED_DATA_DATA_LENGTH", MAX_TAGGED_DATA_DATA_LENGTH.to_string()),
        ("", String::new()),
        ("MAX_CONTRACT_NAME_LENGTH", MAX_CONTRACT_NAME_LENGTH.to_string()),
        ("MAX_CONTRACT_PARAMS_LENGTH", MAX_CONTRACT_PARAMS_LENGTH.to_string()),
        ("", String::new()),
        ("TRANSPARENT_FORGING_BLOCK_3", TRANSPARENT_FORGING_BLOCK_3.to_string()),
        ("TRANSPARENT_FORGING_BLOCK_7", TRANSPARENT_FORGING_BLOCK_7.to_string()),
        ("MAX_REFERENCED_TRANSACTION_TIMESPAN", MAX_REFERENCED_TRANSACTION_TIMESPAN.to_string()),
        ("", String::new()),
        ("MAX_ROLLBACK", MAX_ROLLBACK.to_string()),
        ("GUARANTEED_BALANCE_CONFIRMATIONS", GUARANTEED_BALANCE_CONFIRMATIONS.to_string()),
        ("LEASING_DELAY", LEASING_DELAY.to_string()),
        ("FORGING_DELAY", FORGING_DELAY.to_string()),
        ("FORGING_SPEEDUP", FORGING_SPEEDUP.to_string()),
        ("BATCH_COMMIT_SIZE", BATCH_COMMIT_SIZE.to_string()),
        ("MIN_PRUNABLE_LIFETIME", MIN_PRUNABLE_LIFETIME.to_string()),
        ("", String::new()),
        ("UNCONFIRMED_POOL_DEPOSIT_NQT", UNCONFIRMED_POOL_DEPOSIT_NQT.to_string()),
        ("UNCONFIRMED_POOL_DEPOSIT_FQT", UNCONFIRMED_POOL_DEPOSIT_FQT.to_string()),
        ("SHUFFLING_DEPOSIT_NQT", SHUFFLING_DEPOSIT_NQT.to_string()),
        ("", String::new()),
        ("MAX_KNOWN_PEERS", MAX_KNOWN_PEERS.to_string()),
        ("MIN_KNOWN_PEERS", MIN_KNOWN_PEERS.to_string()),
        ("MAX_CONNECTIONS", MAX_CONNECTIONS.to_string()),
        ("MAX_INBOUND_CONNECTIONS", MAX_INBOUND_CONNECTIONS.to_string()),
        ("MAX_OUTBOUND_CONNECTIONS", MAX_OUTBOUND_CONNECTIONS.to_string()),
        ("MAX_REQUEST_SIZE", MAX_REQUEST_SIZE.to_string()),
        ("MAX_RESPONSE_SIZE", MAX_RESPONSE_SIZE.to_string()),
        ("MAX_MESSAGE_SIZE", MAX_MESSAGE_SIZE.to_string()),
        ("MIN_COMPRESS_SIZE", MIN_COMPRESS_SIZE.to_string()),
        ("CONNECT_TIMEOUT_MS", CONNECT_TIMEOUT_MS.to_string()),
        ("READ_TIMEOUT_MS", READ_TIMEOUT_MS.to_string()),
        ("BLACKLISTING_PERIOD_SECS", BLACKLISTING_PERIOD_SECS.to_string()),
        ("BLACKLISTING_THRESHOLD", BLACKLISTING_THRESHOLD.to_string()),
        ("DEFAULT_API_PORT", DEFAULT_API_PORT.to_string()),
        ("DEFAULT_API_SSL_PORT", DEFAULT_API_SSL_PORT.to_string()),
    ];

    match args.format.as_deref() {
        Some("json") => {
            let mut map = serde_json::Map::new();
            for (key, value) in &constants {
                if !key.is_empty() {
                    map.insert(key.to_string(), serde_json::Value::String(value.clone()));
                }
            }
            println!("{}", serde_json::to_string_pretty(&map)?);
        }
        _ => {
            println!("NRCS Blockchain Constants");
            println!("=========================");
            for (key, value) in &constants {
                if key.is_empty() {
                    println!();
                } else {
                    println!("  {:<45} {}", key, value);
                }
            }
        }
    }

    tracing::info!("Exported {} constants", constants.iter().filter(|(k, _)| !k.is_empty()).count());
    Ok(())
}

// ============================================================================
// CompactDatabase
// ============================================================================

pub async fn compact_database(args: CompactDatabaseArgs) -> anyhow::Result<()> {
    let db_dir = args.db_dir.as_deref().unwrap_or("nrcs_db");
    let db_url = args.db_url.as_deref().unwrap_or("jdbc:h2:./nrcs_db/nrcs;DB_CLOSE_ON_EXIT=FALSE");

    println!("Database Compaction Tool");
    println!("========================");
    println!();
    println!("WARNING: The NRCS node must NOT be running during compaction.");
    println!();
    println!("Database directory: {}", db_dir);
    println!("Database URL:       {}", db_url);
    println!();

    let db_path = std::path::Path::new(db_dir);
    if !db_path.exists() {
        println!("INFO: Database directory '{}' does not exist.", db_dir);
        println!("      If using a remote database, this is expected.");
        println!("      The compaction will be performed via the database URL.");
    }

    // Determine the probable database file name from the URL
    let db_file_name = if db_url.contains("nrcs") { "nrcs" } else { "nrcs" };
    let db_file_mv = db_path.join(format!("{}.mv.db", db_file_name));
    let db_file_h2 = db_path.join(format!("{}.h2.db", db_file_name));

    let db_file = if db_file_mv.exists() {
        println!("Found database file: {}", db_file_mv.display());
        db_file_mv
    } else if db_file_h2.exists() {
        println!("Found database file: {}", db_file_h2.display());
        db_file_h2
    } else {
        println!("NOTE: No local database file found at '{}/{}.*.db'", db_dir, db_file_name);
        println!("      If using a remote database server, compaction must be");
        println!("      performed directly on the database server.");
        println!();
        println!("For H2 database compaction, run these SQL commands:");
        println!("  SHUTDOWN COMPACT");
        println!();
        println!("For SQLite database compaction, run:");
        println!("  VACUUM;");
        return Ok(());
    };

    println!();
    println!("Compaction process (simulated):");
    println!("  1. Create SQL backup script:  backup.sql.gz");
    println!("  2. Rename current DB:          {} -> {}.bak", db_file.display(), db_file.display());
    println!("  3. Recreate DB from script");
    println!("  4. Run ANALYZE");
    println!("  5. Run SHUTDOWN COMPACT");
    println!("  6. Delete backup script and old file");
    println!();
    println!("To perform actual compaction:");
    println!("  1. Stop the NRCS node");
    println!("  2. Connect to the database using a SQL client");
    println!("  3. Run: SHUTDOWN COMPACT (H2) or VACUUM (SQLite)");
    println!("  4. Restart the NRCS node");

    tracing::info!("Compaction info displayed for db_dir={}", db_dir);
    Ok(())
}

// ============================================================================
// RecoverPassphrase
// ============================================================================

/// Build a character dictionary based on the specified range
fn build_dictionary(kind: &str, custom: Option<&str>) -> Vec<char> {
    match kind {
        "ascii" => (32..127).map(|c| c as u8 as char).collect(),
        "asciiall" => (0..256).map(|c| c as u8 as char).collect(),
        "unicode" => (0..=0xFFFFu32)
            .filter_map(|c| char::from_u32(c))
            .collect(),
        "" => (32..127).map(|c| c as u8 as char).collect(),
        _ => {
            if let Some(custom_str) = custom {
                custom_str.chars().collect()
            } else {
                kind.chars().collect()
            }
        }
    }
}

pub async fn recover_passphrase(args: RecoverPassphraseArgs) -> anyhow::Result<()> {
    println!("Passphrase Recovery Tool");
    println!("=========================");
    println!();
    println!("Wildcard:    {}", args.wildcard);
    println!();

    // Parse known public keys if provided
    let known_pubkeys: Vec<Vec<u8>> = match &args.known_keys {
        Some(keys_str) => keys_str
            .split(',')
            .map(|s| hex::decode(s.trim()).unwrap_or_default())
            .filter(|v| !v.is_empty())
            .collect(),
        None => Vec::new(),
    };

    // Parse known account IDs if provided
    let known_accounts: Vec<u64> = match &args.known_accounts {
        Some(accts_str) => accts_str
            .split(',')
            .filter_map(|s| s.trim().parse::<u64>().ok())
            .collect(),
        None => Vec::new(),
    };

    // Parse positions (1-indexed in input, convert to 0-indexed)
    let positions: Vec<usize> = match &args.positions {
        Some(pos_str) if !pos_str.is_empty() => pos_str
            .split(',')
            .filter_map(|s| s.trim().parse::<usize>().ok())
            .map(|p| p.saturating_sub(1))
            .collect(),
        _ => Vec::new(),
    };

    // Build dictionary
    let dict_kind = args.dictionary.as_deref().unwrap_or("ascii");
    let dictionary = build_dictionary(dict_kind, args.dictionary.as_deref());
    let total_permutations = (dictionary.len() as f64).powi(positions.len() as i32);

    println!("Positions:   {:?} (0-indexed: {:?})",
        positions.iter().map(|p| p + 1).collect::<Vec<_>>(),
        positions);
    println!("Dictionary:  {} ({} characters)", dict_kind, dictionary.len());
    println!("Permutations to try: {:.0}", total_permutations);
    println!();

    if known_pubkeys.is_empty() && known_accounts.is_empty() {
        println!("No known public keys or account IDs provided.");
        println!("Use --known-keys or --known-accounts to specify targets to match against.");
        println!();
        println!("Example:");
        println!("  nrcs-cli recover-passphrase \\");
        println!("    --wildcard \"my secret XhraZe here\" \\");
        println!("    --positions 11 \\");
        println!("    --dictionary ascii \\");
        println!("    --known-accounts 12345678901234567890");
        return Ok(());
    }

    if positions.is_empty() {
        println!("No positions specified. Scanning for a single typo at each position...");
        println!("(This will try replacing each character one at a time)");
        println!();

        let mut wildcard_chars: Vec<char> = args.wildcard.chars().collect();
        for pos in 0..wildcard_chars.len() {
            let original = wildcard_chars[pos];
            let mut found = false;

            for &c in &dictionary {
                if c == original {
                    continue;
                }
                wildcard_chars[pos] = c;
                let candidate: String = wildcard_chars.iter().collect();

                // Check if this candidate produces a matching public key or account ID
                if check_candidate(&candidate, &known_pubkeys, &known_accounts) {
                    println!("  FOUND at position {}: char '{}' -> '{}'", pos + 1, original, c);
                    println!("  Recovered passphrase: {}", candidate);
                    found = true;
                    break;
                }
            }

            wildcard_chars[pos] = original;

            if !found {
                println!("  Position {}: no match found", pos + 1);
            }
        }
    } else {
        println!("Scanning positions {:?}...", positions.iter().map(|p| p + 1).collect::<Vec<_>>());
        println!();

        let mut wildcard_chars: Vec<char> = args.wildcard.chars().collect();

        // Recursive scan for multi-position recovery
        fn scan_positions(
            wildcard: &mut [char],
            positions: &[usize],
            pos_idx: usize,
            dictionary: &[char],
            known_pubkeys: &[Vec<u8>],
            known_accounts: &[u64],
            counter: &mut u64,
        ) -> bool {
            if pos_idx >= positions.len() {
                *counter += 1;
                if *counter % 100_000 == 0 {
                    eprintln!("  ... tried {} permutations", counter);
                }
                let candidate: String = wildcard.iter().collect();
                return check_candidate(&candidate, known_pubkeys, known_accounts);
            }

            let target_pos = positions[pos_idx];
            for &c in dictionary {
                wildcard[target_pos] = c;
                if scan_positions(
                    wildcard, positions, pos_idx + 1, dictionary,
                    known_pubkeys, known_accounts, counter,
                ) {
                    return true;
                }
            }
            false
        }

        let mut counter = 0u64;
        let found = scan_positions(
            &mut wildcard_chars,
            &positions,
            0,
            &dictionary,
            &known_pubkeys,
            &known_accounts,
            &mut counter,
        );

        if found {
            let recovered: String = wildcard_chars.iter().collect();
            println!();
            println!("SOLUTION FOUND!");
            println!("Recovered passphrase: {}", recovered);

            // Display derived public key and account info
            match crypto::derive_public_key(&recovered) {
                Ok(pubkey) => {
                    println!("Public key: {}", hex::encode(&pubkey));
                    let account_id = crypto::account_id_from_public_key(&pubkey);
                    println!("Account ID: {}", account_id);
                    println!("RS Address: NRCS-{}", crypto::reed_solomon::encode(account_id));
                }
                Err(e) => {
                    println!("Could not derive public key: {}", e);
                }
            }
        } else {
            println!();
            println!("No solution found after {} permutations.", counter);
            println!("Try adjusting the wildcard, positions, or dictionary.");
        }
    }

    Ok(())
}

/// Check if a candidate passphrase produces a matching public key or account ID
fn check_candidate(
    passphrase: &str,
    known_pubkeys: &[Vec<u8>],
    known_accounts: &[u64],
) -> bool {
    let pubkey = match crypto::derive_public_key(passphrase) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    // Check against known public keys
    for kp in known_pubkeys {
        if pubkey == *kp {
            return true;
        }
    }

    // Check against known account IDs
    if !known_accounts.is_empty() {
        let account_id = crypto::account_id_from_public_key(&pubkey);
        for &ka in known_accounts {
            if account_id == ka {
                return true;
            }
        }
    }

    false
}

// ============================================================================
// BaseTargetTest
// ============================================================================

pub async fn base_target_test(args: BaseTargetTestArgs) -> anyhow::Result<()> {
    use blockchain_types::constants;

    let previous_base_target = args.previous_base_target;
    let blocktime_ema = args.blocktime_ema;

    let min_base_target = constants::MIN_BASE_TARGET;
    let max_base_target = if args.testnet {
        constants::INITIAL_BASE_TARGET * constants::MAX_BALANCE_FXT
    } else {
        constants::INITIAL_BASE_TARGET * 50
    };
    let gamma = constants::BASE_TARGET_GAMMA as i64;
    let block_time = constants::BLOCK_TIME as i64;

    let min_blocktime_limit = constants::MIN_BLOCKTIME_LIMIT as i64;
    let max_blocktime_limit = constants::MAX_BLOCKTIME_LIMIT as i64;

    println!("Base Target Calculation Test");
    println!("=============================");
    println!();
    println!("Input parameters:");
    println!("  Previous base target: {}", previous_base_target);
    println!("  Block time EMA:       {} seconds", blocktime_ema);
    println!();
    println!("Constants:");
    println!("  BLOCK_TIME:           {} seconds", block_time);
    println!("  INITIAL_BASE_TARGET:  {}", constants::INITIAL_BASE_TARGET);
    println!("  MIN_BASE_TARGET:      {}", min_base_target);
    println!("  MAX_BASE_TARGET:      {}", max_base_target);
    println!("  GAMMA:                {}", gamma);
    println!("  MIN_BLOCKTIME_LIMIT:  {}", min_blocktime_limit);
    println!("  MAX_BLOCKTIME_LIMIT:  {}", max_blocktime_limit);
    println!();

    // Calculate new base target using the Java formula from BaseTargetTest
    let base_target = if blocktime_ema as i64 > block_time {
        // Block time is too slow: increase difficulty (lower base target)
        let clamped_ema = (blocktime_ema as i64).min(max_blocktime_limit) as u64;
        (previous_base_target * clamped_ema) / block_time as u64
    } else {
        // Block time is too fast: decrease difficulty (raise base target)
        let clamped_ema = (blocktime_ema as i64).max(min_blocktime_limit) as u64;
        let reduction = previous_base_target * gamma as u64
            * (block_time as u64 - clamped_ema)
            / (100 * block_time as u64);
        previous_base_target.saturating_sub(reduction)
    };

    let base_target = base_target.clamp(min_base_target, max_base_target);

    println!("Result:");
    println!("  New base target:      {}", base_target);
    println!();

    // Additional analysis
    let ratio = previous_base_target as f64 / base_target as f64;
    println!("Analysis:");
    println!("  Previous/New ratio:   {:.6}", ratio);

    if blocktime_ema > block_time as u64 {
        println!(
            "  Block time is {}s slower than target {}s => difficulty increased",
            blocktime_ema - block_time as u64,
            block_time
        );
    } else if blocktime_ema < block_time as u64 {
        println!(
            "  Block time is {}s faster than target {}s => difficulty decreased",
            block_time as u64 - blocktime_ema,
            block_time
        );
    } else {
        println!("  Block time is exactly on target => no change");
    }

    // Show what happens with adjacent values
    println!();
    println!("Sensitivity analysis (nearby EMA values):");
    for offset in [-5i64, -3, -1, 0, 1, 3, 5] {
        let test_ema = (blocktime_ema as i64 + offset).max(0) as u64;
        let test_bt = if test_ema as i64 > block_time {
            let clamped_ema = (test_ema as i64).min(max_blocktime_limit) as u64;
            (previous_base_target * clamped_ema) / block_time as u64
        } else {
            let clamped_ema = (test_ema as i64).max(min_blocktime_limit) as u64;
            let reduction = previous_base_target * gamma as u64
                * (block_time as u64 - clamped_ema)
                / (100 * block_time as u64);
            previous_base_target.saturating_sub(reduction)
        };
        let test_bt = test_bt.clamp(min_base_target, max_base_target);
        let delta = test_bt as i64 - base_target as i64;
        println!(
            "  EMA={:>3}s => base_target={:>12} (delta: {:>+12})",
            test_ema, test_bt, delta
        );
    }

    tracing::info!(
        "Base target calculated: prev={} ema={} => new={}",
        previous_base_target, blocktime_ema, base_target
    );
    Ok(())
}
