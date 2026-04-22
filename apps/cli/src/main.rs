//! NRCS CLI 工具
//!
//! 对应 Java: nrcs.tools.* 工具集
//!
//! 包含：
//! - generate-keypair: 生成密钥对
//! - generate-passphrase: 生成助记词
//! - sign-tx: 签名交易
//! - compact-db: 压缩数据库
//! - verify-trace: 验证跟踪文件

mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nrcs-cli")]
#[command(about = "NRCS blockchain command line tools", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    GenerateKeypair(commands::GenerateKeypairArgs),
    GeneratePassphrase(commands::GeneratePassphraseArgs),
    SignTransaction(commands::SignTransactionArgs),
    VerifyAddress(commands::VerifyAddressArgs),
    ConvertHex(commands::ConvertHexArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateKeypair(args) => commands::generate_keypair(args).await?,
        Commands::GeneratePassphrase(args) => commands::generate_passphrase(args).await?,
        Commands::SignTransaction(args) => commands::sign_transaction(args).await?,
        Commands::VerifyAddress(args) => commands::verify_address(args).await?,
        Commands::ConvertHex(args) => commands::convert_hex(args).await?,
    }

    Ok(())
}
