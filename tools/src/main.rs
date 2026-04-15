//! Schema Analysis Tool
//!
//! Parses H2 DDL SQL script and generates structured JSON description
//! of all tables, columns, constraints, indexes, and foreign keys.

use std::collections::HashMap;
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
    let unique_idx_re = Regex::new(r"(?i)CREATE\s+UNIQUE\s+INDEX\s+IF\s+NOT\s+EXISTS\s+(\w+)\s+ON\s+(\w+)\s*\(([^)]+)\)").unwrap();
    let index_re = Regex::new(r"(?i)CREATE\s+INDEX\s+IF\s+NOT\s+EXISTS\s+(\w+)\s+ON\s+(\w+)\s*\(([^)]+)\)").unwrap();
    let foreign_key_re = Regex::new(r"(?i)CONSTRAINT\s+\w+\s+FOREIGN\s+KEY\s*\(([^)]+)\)\s+REFERENCES\s+(\w+)\s*\(([^)]+)\)(?:\s+ON\s+DELETE\s+(\w+))?").unwrap();

    let mut current_table: Option<String> = None;
    let mut column_defs: Vec<(String, String, String)> = Vec::new();
    let mut all_constraints: String = String::new();

    for stmt in &statements {
        let stmt = stmt.trim();
        if stmt.is_empty() {
            continue;
        }
        all_constraints.push_str("\n");
        all_constraints.push_str(stmt);

        // Check for CREATE TABLE
        if let Some(caps) = create_table_re.captures(stmt) {
            // Process previous table if exists
            if let Some(table_name) = &current_table {
                process_table(&mut schema, table_name, column_defs);
            }

            // Start new table
            current_table = Some(caps[1].to_uppercase());
            column_defs = Vec::new();
            continue;
        }

        // If we're inside a table definition, try to parse column lines
        if let Some(_) = &current_table {
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
            }
        }
    }

    // Process last table
    if let Some(table_name) = current_table {
        process_table(&mut schema, &table_name, column_defs);
    }

    // Parse indexes from constraints string of each table
    // We need to re-parse the full SQL for indexes, as they may be separate statements
    for stmt in &statements {
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
    for table in schema.tables.values_mut() {
        let fk_captures = foreign_key_re.captures_iter(&all_constraints);
        for caps in fk_captures {
            let fk_cols = caps[1].trim();
            let ref_table = caps[2].to_string().to_uppercase();
            let ref_cols = caps[3].trim();
            let on_delete = caps.get(4).map(|m| m.as_str().to_string());

            let fk_cols_vec: Vec<&str> = fk_cols.split(',').map(|s| s.trim()).collect();
            let ref_cols_vec: Vec<&str> = ref_cols.split(',').map(|s| s.trim()).collect();

            for (fk_col, ref_col) in fk_cols_vec.iter().zip(ref_cols_vec.iter()) {
                if table.columns.iter().any(|c| c.name == *fk_col) {
                    table.foreign_keys.push(ForeignKeyInfo {
                        column: fk_col.to_string(),
                        referenced_table: ref_table.clone(),
                        referenced_column: ref_col.to_string(),
                        on_delete: on_delete.clone(),
                    });
                }
            }
        }
    }

    schema.total_tables = schema.tables.len();
    schema
}

fn process_table(schema: &mut SchemaAnalysis, table_name: &str, column_defs: Vec<(String, String, String)>) {
    let mut columns = Vec::new();
    let mut primary_key_cols = Vec::new();
    let mut unique_groups = Vec::new();

    let mut pk_cols_from_inline: Vec<String> = Vec::new();

    for (col_name, data_type, rest) in column_defs {
        let mut is_nullable = true;
        let mut default_value = None;
        let mut is_auto_increment = false;

        if rest.to_uppercase().contains("NOT NULL") {
            is_nullable = false;
        }

        if rest.to_uppercase().contains("AUTO_INCREMENT") {
            is_auto_increment = true;
        }

        if let Some(default_match) = Regex::new(r"(?i)DEFAULT\s+([^,\n]+)").unwrap().find(&rest) {
            let default_str = default_match.as_str().to_string();
            default_value = Some(default_str);
        }

        let mut is_pk = false;
        if rest.to_uppercase().contains("PRIMARY KEY") {
            is_pk = true;
            pk_cols_from_inline.push(col_name.clone());
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

    if !pk_cols_from_inline.is_empty() {
        primary_key_cols = pk_cols_from_inline;
    }

    let table = TableSchema {
        name: table_name.to_string(),
        columns,
        primary_key: primary_key_cols,
        unique_constraints: unique_groups,
        indexes: Vec::new(),
        foreign_keys: Vec::new(),
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
