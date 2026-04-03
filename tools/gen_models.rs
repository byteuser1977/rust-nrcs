//! Generate Rust ORM models from schema_analysis.json

use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
struct ColumnInfo {
    name: String,
    data_type: String,
    is_nullable: bool,
    default_value: Option<String>,
    is_auto_increment: bool,
    is_primary_key: bool,
}

#[derive(Debug, Deserialize)]
struct TableSchema {
    name: String,
    columns: Vec<ColumnInfo>,
    primary_key: Vec<String>,
    unique_constraints: Vec<Vec<String>>,
    indexes: Vec<IndexInfo>,
    foreign_keys: Vec<ForeignKeyInfo>,
}

#[derive(Debug, Deserialize)]
struct IndexInfo {
    name: String,
    columns: Vec<String>,
    is_unique: bool,
}

#[derive(Debug, Deserialize)]
struct ForeignKeyInfo {
    column: String,
    referenced_table: String,
    referenced_column: String,
    on_delete: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SchemaAnalysis {
    tables: std::collections::HashMap<String, TableSchema>,
    total_tables: usize,
}

// Existing model names (from models.rs)
const EXISTING_MODELS: &[&str] = &[
    "Block", "Transaction", "Account", "AccountAsset", "Asset", "Contract", "TransactionReceipt"
];

fn rust_type_name(table_name: &str) -> String {
    // Convert table name like ACCOUNT_ASSET -> AccountAssetModel
    let camel = table_name.split('_').map(|s| {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        }
    }).collect::<String>();
    format!("{}Model", camel)
}

fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_ascii_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

fn rust_field_name(col_name: &str) -> String {
    // Convert COLUMN_NAME to snake_case, but avoid Rust keywords
    let snake = to_snake_case(col_name);
    match snake.as_str() {
        "type" => "type_".to_string(),
        "match" => "match_".to_string(),
        _ => snake,
    }
}

fn map_h2_type_to_rust(data_type: &str, is_nullable: bool, is_auto_increment: bool) -> (String, String) {
    // Returns (rust_type, sqlx_rename_attr)
    let base_type = if data_type.starts_with("binary") || data_type == "varbinary" || data_type == "blob" {
        "Vec<u8>"
    } else if data_type == "bigint" {
        "i64"
    } else if data_type == "integer" {
        "i32"
    } else if data_type == "smallint" || data_type == "tinyint" {
        "i16" // H2 tinyint can be boolean? Actually sometimes boolean also uses tinyint/boolean. We'll handle boolean separately.
    } else if data_type == "boolean" {
        "bool"
    } else if data_type.starts_with("varchar") || data_type == "text" || data_type == "varchar" {
        "String"
    } else {
        // fallback
        "String"
    };

    // For auto_increment primary key, we keep as i64
    // For nullable, we may use Option<T>
    let rust_type = if is_nullable && base_type != "String" && base_type != "Vec<u8>" {
        format!("Option<{}>", base_type)
    } else {
        base_type.to_string()
    };

    // Column rename: we need to map to actual DB column name (may be same as field or need rename)
    // In existing models, they use #[sqlx(rename = "DB_ID")]
    let rename = if data_type == "boolean" {
        // H2 boolean is BOOLEAN, column name as is
        "".to_string()
    } else {
        "".to_string()
    };

    (rust_type, rename)
}

fn generate_model(table: &TableSchema) -> String {
    let struct_name = rust_type_name(&table.name);
    let mut out = String::new();

    // Derives
    out.push_str("#[derive(Debug, Clone, PartialEq, sqlx::FromRow, Serialize, Deserialize)]\n");
    out.push_str(&format!("pub struct {} {{\n", struct_name));

    for col in &table.columns {
        let field_name = rust_field_name(&col.name);
        // Determine Rust type
        let (rust_type, _) = map_h2_type_to_rust(&col.data_type, col.is_nullable, col.is_auto_increment);

        // Add rename attribute if column name differs (usually uppercase vs snake)
        let rename_attr = if col.name != field_name.to_uppercase() && col.name != field_name {
            format!("    #[sqlx(rename = \"{}\")]\n", col.name)
        } else {
            String::new()
        };

        if !rename_attr.is_empty() {
            out.push_str(&rename_attr);
        }
        out.push_str(&format!("    pub {}: {},\n", field_name, rust_type));
    }

    out.push_str("}\n\n");

    // Impl block with placeholder for to_domain/from_domain if needed
    out.push_str(&format!("impl {} {{\n", struct_name));
    out.push_str("    /// Convert to domain type (to be implemented as needed)\n");
    out.push_str("    pub fn to_domain(&self) -> Result<Block, BlockchainError> {\n");
    out.push_str("        unimplemented!()\n");
    out.push_str("    }\n\n");
    out.push_str("    /// Convert from domain type\n");
    out.push_str("    pub fn from_domain(_block: &Block) -> Result<Self, BlockchainError> {\n");
    out.push_str("        unimplemented!()\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    // Repository trait impl placeholder
    out.push_str(&format!("#[async_trait]\npub trait {}Repository: Repository<{}> {{\n", struct_name.trim_end_matches("Model"), struct_name));
    out.push_str("    async fn find_by_id(conn: &database::DbConnection, id: i64) -> sqlx::Result<Option<Self>> {\n");
    out.push_str("        sqlx::query_as!(Self, \"SELECT * FROM {} WHERE db_id = ?\", id)\n", table.name);
    out.push_str("            .fetch_optional(conn)\n");
    out.push_str("            .await\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    out
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema_path = "D:/workspace/git/rust-nrcs/crates/orm/schema_analysis.json";
    let output_path = "D:/workspace/git/rust-nrcs/crates/orm/src/generated_models.rs";

    let data = std::fs::read_to_string(schema_path)?;
    let schema: SchemaAnalysis = serde_json::from_str(&data)?;

    // Existing models set
    let existing: HashSet<String> = EXISTING_MODELS.iter().map(|&s| s.to_string()).collect();

    let mut output = String::new();
    output.push_str("//! Auto-generated ORM models. DO NOT EDIT MANUALLY.\n");
    output.push_str("//! Generated by tools/gen_models.rs\n\n");
    output.push_str("use sqlx::FromRow;\nuse serde::{Deserialize, Serialize};\nuse chrono::{DateTime, Utc};\nuse crate::database::DbConnection;\nuse crate::error::BlockchainError;\n\n");

    let mut generated_names = Vec::new();

    for (tbl_name, table) in &schema.tables {
        let struct_name = rust_type_name(tbl_name);
        if existing.contains(&struct_name.trim_end_matches("Model")) || generated_names.contains(&struct_name) {
            continue;
        }
        generated_names.push(struct_name.clone());
        output.push_str(&generate_model(table));
    }

    std::fs::write(output_path, output)?;
    println!("Generated {} models to {}", generated_names.len(), output_path);
    Ok(())
}
