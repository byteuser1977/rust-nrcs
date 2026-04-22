//! CLI 命令实现

use clap::Args;
use std::io::{self, BufRead, Write};

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
    };
    
    let secret_key_hex = match secret_key {
        crypto::SecretKey::Ed25519(bytes) => hex::encode(bytes),
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
    
    if word_count != 12 && word_count != 24 {
        anyhow::bail!("Word count must be 12 or 24");
    }
    
    let mnemonic = crypto::generate_mnemonic(word_count)?;
    
    let account_id = crypto::derive_account_id(&mnemonic)?;
    let public_key = crypto::derive_public_key(&mnemonic)?;
    
    println!("Generated Passphrase:");
    println!("====================");
    println!("Passphrase: {}", mnemonic);
    println!("Account ID: {}", account_id);
    println!("Public Key: {}", hex::encode(public_key));
    
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
