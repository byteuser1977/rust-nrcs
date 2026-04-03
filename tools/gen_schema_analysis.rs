//! Schema Analysis Tool
//!
//! Parses H2 DDL SQL script and generates structured JSON description
//! of all tables, columns, constraints, indexes, and foreign keys.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ColumnInfo {
    name: String,
    data_type: String,
    is_nullable: bool,
    default_value: Option<String>,
    is_auto_increment: bool,
    is_primary_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexInfo {
    name: String,
    columns: Vec<String>,
    is_unique: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ForeignKeyInfo {
    column: String,
    referenced_table: String,
    referenced_column: String,
    on_delete: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TableSchema {
    name: String,
    columns: Vec<ColumnInfo>,
    primary_key: Vec<String>,
    unique_constraints: Vec<Vec<String>>,
    indexes: Vec<IndexInfo>,
    foreign_keys: Vec<ForeignKeyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SchemaAnalysis {
    tables: HashMap<String, TableSchema>,
    total_tables: usize,
}

fn parse_sql(sql: &str) -> SchemaAnalysis {
    let mut schema = SchemaAnalysis {
        tables: HashMap::new(),
        total_tables: 0,
    };

    // Normalize whitespace
    let sql = sql.replace("\r\n", "\n").replace("\r", "\n");

    // Split into statements
    let statements: Vec<&str> = sql.split(';').collect();

    // Regex patterns
    let create_table_re = Regex::new(r"(?i)CREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+(\w+)").unwrap();
    let column_def_re = Regex::new(r"(?i)^\s*(\w+)\s+([\w\(\)]+)(.*)$").unwrap();
    let primary_key_re = Regex::new(r"(?i)PRIMARY\s+KEY\s+\(([^)]+)\)").unwrap();
    let unique_idx_re = Regex::new(r"(?i)CREATE\s+UNIQUE\s+INDEX\s+IF\s+NOT\s+EXISTS\s+(\w+)\s+ON\s+(\w+)\s*\(([^)]+)\)").unwrap();
    let index_re = Regex::new(r"(?i)CREATE\s+INDEX\s+IF\s+NOT\s+EXISTS\s+(\w+)\s+ON\s+(\w+)\s*\(([^)]+)\)").unwrap();
    let foreign_key_re = Regex::new(r"(?i)CONSTRAINT\s+\w+\s+FOREIGN\s+KEY\s*\(([^)]+)\)\s+REFERENCES\s+(\w+)\s*\(([^)]+)\)(?:\s+ON\s+DELETE\s+(\w+))?").unwrap();

    let mut current_table: Option<String> = None;
    let mut column_defs: Vec<(String, String, String)> = Vec::new(); // (name, type, rest)
    let mut constraints: String = String::new();

    for stmt in statements {
        let stmt = stmt.trim();
        if stmt.is_empty() {
            continue;
        }

        // Check for CREATE TABLE
        if let Some(caps) = create_table_re.captures(stmt) {
            // Process previous table if exists
            if let Some(table_name) = &current_table {
                process_table(&mut schema, table_name, column_defs, constraints);
            }

            // Start new table
            current_table = Some(caps[1].to_uppercase());
            column_defs = Vec::new();
            constraints = stmt.to_string();
            continue;
        }

        // If we're inside a table definition, try to parse column lines
        if let Some(table_name) = &current_table {
            let trimmed = stmt.trim_start();
            if trimmed.starts_with(',') {
                // Continuation - parse column definition
                let col_part = trimmed[1..].trim_start();
                if let Some(caps) = column_def_re.captures(col_part) {
                    let col_name = caps[1].to_string();
                    let data_type = caps[2].to_string();
                    let rest = caps[3].to_string();
                    column_defs.push((col_name, data_type, rest));
                }
                constraints.push_str("\n");
                constraints.push_str(stmt);
            } else if trimmed.to_uppercase().starts_with("CONSTRAINT") {
                constraints.push_str("\n");
                constraints.push_str(stmt);
            } else if trimmed.to_uppercase().starts_with("CREATE") {
                // This is an index or constraint after table definition
                constraints.push_str("\n");
                constraints.push_str(stmt);
            }
        }
    }

    // Process last table
    if let Some(table_name) = current_table {
        process_table(&mut schema, &table_name, column_defs, constraints);
    }

    // Parse indexes from constraints string of each table
    // We need to re-parse the full SQL for indexes, as they may be separate statements
    for stmt in statements {
        let stmt = stmt.trim();
        if stmt.is_empty() {
            continue;
        }

        // Parse UNIQUE indexes
        if let Some(caps) = unique_idx_re.captures(stmt) {
            let idx_name = caps[1].to_string();
            let table_name = caps[2].to_string().to_uppercase();
            let columns: Vec<String> = caps[3].split(',').map(|s| s.trim().to_string()).collect();

            if let Some(table) = schema.tables.get_mut(&table_name) {
                table.indexes.push(IndexInfo {
                    name: idx_name,
                    columns,
                    is_unique: true,
                });
            }
        }

        // Parse regular indexes
        if let Some(caps) = index_re.captures(stmt) {
            let idx_name = caps[1].to_string();
            let table_name = caps[2].to_string().to_uppercase();
            let columns: Vec<String> = caps[3].split(',').map(|s| s.trim().to_string()).collect();

            if let Some(table) = schema.tables.get_mut(&table_name) {
                table.indexes.push(IndexInfo {
                    name: idx_name,
                    columns,
                    is_unique: false,
                });
            }
        }
    }

    // Parse foreign keys from constraints
    // Need to associate with tables by column names
    // We'll parse from each table's constraints
    for table in schema.tables.values_mut() {
        // Extract foreign keys from constraint definitions within this table's DDL
        // This is tricky, but we can parse all foreign key constraints and match by column
        // For simplicity, we'll parse the entire SQL once more for foreign keys
    }

    // Better approach: parse all foreign key constraints globally
    let mut fk_map: HashMap<String, Vec<ForeignKeyInfo>> = HashMap::new();

    for stmt in statements {
        let stmt = stmt.trim();
        if stmt.is_empty() {
            continue;
        }

        // Find all FOREIGN KEY constraints
        // Pattern: CONSTRAINT constraint_name FOREIGN KEY (column) REFERENCES table (column) ON DELETE CASCADE
        let fk_captures = foreign_key_re.captures_iter(stmt);
        for caps in fk_captures {
            let fk_cols = caps[1].trim();
            let ref_table = caps[2].to_string().to_uppercase();
            let ref_cols = caps[3].trim();
            let on_delete = caps.get(4).map(|m| m.as_str().to_string());

            // Handle multiple columns (composite FK) - for simplicity we'll store as comma-separated
            let fk_cols_vec: Vec<&str> = fk_cols.split(',').map(|s| s.trim()).collect();
            let ref_cols_vec: Vec<&str> = ref_cols.split(',').map(|s| s.trim()).collect();

            for (fk_col, ref_col) in fk_cols_vec.iter().zip(ref_cols_vec.iter()) {
                let fk_info = ForeignKeyInfo {
                    column: fk_col.to_string(),
                    referenced_table: ref_table.clone(),
                    referenced_column: ref_col.to_string(),
                    on_delete: on_delete.clone(),
                };

                // Track which table this FK belongs to by looking up column in our schema
                // We need to find which table contains this column as a foreign key
                // We can infer from the CREATE TABLE statement context
                // For now, we'll collect all FKs and then assign them later
                fk_map.entry(fk_col.to_string()).or_insert_with(Vec::new).push(fk_info);
            }
        }
    }

    // Assign FKs to tables by matching column names present in tables
    // This is a simplification - assumes column names are unique across tables (they're not really)
    // But in practice, columns like BLOCK_ID, ACCOUNT_ID etc. are specific enough
    for (col_name, fk_list) in fk_map {
        for table in schema.tables.values_mut() {
            if table.columns.iter().any(|c| c.name == col_name) {
                for fk in &fk_list {
                    if !table.foreign_keys.contains(fk) {
                        table.foreign_keys.push(fk.clone());
                    }
                }
            }
        }
    }

    schema.total_tables = schema.tables.len();
    schema
}

fn process_table(schema: &mut SchemaAnalysis, table_name: &str, column_defs: Vec<(String, String, String)>, constraints: String) {
    let mut columns = Vec::new();
    let mut primary_key_cols = Vec::new();
    let mut unique_groups = Vec::new();

    // Parse primary key from last column definition if present
    let mut pk_cols: Vec<&str> = Vec::new();

    for (col_name, data_type, rest) in column_defs {
        let mut is_nullable = true;
        let mut default_value = None;
        let mut is_auto_increment = false;

        // Check NOT NULL
        if rest.to_uppercase().contains("NOT NULL") {
            is_nullable = false;
        }

        // Check AUTO_INCREMENT
        if rest.to_uppercase().contains("AUTO_INCREMENT") {
            is_auto_increment = true;
        }

        // Extract DEFAULT value
        if let Some(default_match) = Regex::new(r"(?i)DEFAULT\s+([^,\n]+)").unwrap().find(&rest) {
            let default_str = default_match.as_str().to_string();
            if default_str.eq_ignore_ascii_case("TRUE") || default_str.eq_ignore_ascii_case("FALSE") {
                // keep as string for bool parsing
                default_value = Some(default_str);
            } else {
                default_value = Some(default_str);
            }
        }

        // Check if this column is part of PRIMARY KEY
        // In H2, PRIMARY KEY can be defined inline: "X BIGINT AUTO_INCREMENT PRIMARY KEY NOT NULL"
        let mut is_pk = false;
        if rest.to_uppercase().contains("PRIMARY KEY") {
            is_pk = true;
            pk_cols.push(col_name.as_str());
        }

        columns.push(ColumnInfo {
            name: col_name.clone(),
            data_type,
            is_nullable,
            default_value,
            is_auto_increment,
            is_primary_key: is_pk,
        });
    }

    // Parse PRIMARY KEY constraint from the table DDL
    if let Some(pk_match) = Regex::new(r"(?i)PRIMARY\s+KEY\s*\(([^)]+)\)").unwrap().find(&constraints) {
        let pk_str = pk_match.as_str();
        let pk_cols_str = pk_str.split('(').nth(1).unwrap().split(')').next().unwrap();
        primary_key_cols = pk_cols_str.split(',').map(|s| s.trim().to_string()).collect();
        // Mark columns as PK
        for col in &mut columns {
            if primary_key_cols.contains(&col.name) {
                col.is_primary_key = true;
            }
        }
    } else if !pk_cols.is_empty() {
        primary_key_cols = pk_cols.iter().map(|&s| s.to_string()).collect();
    }

    // Parse UNIQUE constraints (multiple columns)
    let unique_re = Regex::new(r"(?i)UNIQUE\s+INDEX\s+IF\s+NOT\s+EXISTS\s+\w+\s+ON\s+\w+\s*\(([^)]+)\)").unwrap();
    for cap in unique_re.captures_iter(&constraints) {
        let uniq_cols = cap[1].split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>();
        if !uniq_cols.is_empty() {
            unique_groups.push(uniq_cols);
        }
    }

    // Also check UNIQUE constraints defined inline? H2 typically uses CREATE UNIQUE INDEX

    let table = TableSchema {
        name: table_name.to_string(),
        columns,
        primary_key: primary_key_cols,
        unique_constraints: unique_groups,
        indexes: Vec::new(), // Will be filled later with global pass
        foreign_keys: Vec::new(), // Will be filled later
    };

    schema.tables.insert(table_name.to_string(), table);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read SQL file
    let sql_path = "D:/workspace/git/nrcs/nrcs-sql/src/main/resources/sql-scripts-h2/0.sql";
    let sql = std::fs::read_to_string(sql_path)?;

    println!("Parsing schema from {}...", sql_path);
    let schema = parse_sql(&sql);

    let output_path = "D:/workspace/git/rust-nrcs/crates/orm/schema_analysis.json";
    let json = serde_json::to_string_pretty(&schema)?;
    std::fs::write(output_path, json)?;

    println!("Schema analysis complete:");
    println!("  Total tables: {}", schema.total_tables);
    for (name, table) in &schema.tables {
        println!("  - {}: {} columns", name, table.columns.len());
    }
    println!("Output written to: {}", output_path);

    Ok(())
}
