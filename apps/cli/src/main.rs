//! NRCS CLI 工具
//!
//! 对应 Java: nrcs.tools.* 工具集
//!
//! 包含：
//! - generate-keypair: 生成密钥对
//! - generate-passphrase: 生成助记词
//! - sign-tx: 签名交易
//! - verify-address: 验证账户地址
//! - convert-hex: 十六进制转换
//! - export-constants: 导出区块链常量
//! - compact-db: 压缩数据库
//! - recover-passphrase: 恢复密码短语
//! - base-target-test: 测试基础目标值计算

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
    ExportConstants(commands::ExportConstantsArgs),
    CompactDatabase(commands::CompactDatabaseArgs),
    RecoverPassphrase(commands::RecoverPassphraseArgs),
    BaseTargetTest(commands::BaseTargetTestArgs),
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
        Commands::ExportConstants(args) => commands::export_constants(args).await?,
        Commands::CompactDatabase(args) => commands::compact_database(args).await?,
        Commands::RecoverPassphrase(args) => commands::recover_passphrase(args).await?,
        Commands::BaseTargetTest(args) => commands::base_target_test(args).await?,
    }

    Ok(())
}
