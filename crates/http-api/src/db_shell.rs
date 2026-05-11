//! Database Shell HTTP handler
//!
//! 与 Java 版本 DbShellServlet 完全对齐
//! 参考: com.bytechain.nrcs.http.DbShellServlet
//!
//! 提供基于 Web 的数据库 SQL 交互终端，支持：
//! - 密码保护（含防暴力破解锁定）
//! - 任意 SQL 执行（SELECT/INSERT/UPDATE/DELETE 等）
//! - 终端式连续交互体验
//! - 多数据库支持（通过 ORM DbMetaRepository 接口）

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::Response,
    Form,
};
use parking_lot::Mutex;
use serde::Deserialize;
use sqlx::{Column, Row};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::ApiState;
use orm::repository::traits::DbMetaRepository;
use orm::repository::sqlite::AnyDbMetaRepository;
use orm::connection::DatabaseType;

const MAX_INCORRECT_ATTEMPTS: u32 = 25;
const LOCK_DURATION_SECS: u64 = 3600;
const MAX_TRACKED_IPS: usize = 1000;

struct PasswordCount {
    count: u32,
    time: u64,
}

static INCORRECT_PASSWORDS: once_cell::sync::Lazy<Mutex<HashMap<String, PasswordCount>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 列出数据库中所有表（使用 ORM 多数据库支持）
async fn list_tables(pool: &sqlx::AnyPool) -> String {
    // 创建 SQLite 元信息仓库实例
    // 注意：这里使用 AnyPool，实际应用中应该根据数据库类型选择合适的实现
    match create_db_meta_repository(pool) {
        Ok(repo) => {
            match repo.list_tables().await {
                Ok(tables) => {
                    if tables.is_empty() {
                        return "No tables found in database.".to_string();
                    }
                    
                    let mut output = String::from("Database Tables:\n");
                    output.push_str(&format!("{:-20} {:>10}\n", "Name", "Type"));
                    output.push_str(&format!("{:-20} {:>10}\n", "--------------------", "----------"));
                    
                    for table in &tables {
                        output.push_str(&format!("{:-20} {:>10}\n", table.name, table.table_type));
                    }
                    
                    output.push_str(&format!("\nTotal: {} table(s)", tables.len()));
                    output
                }
                Err(e) => format!("Error listing tables: {}", e),
            }
        }
        Err(e) => format!("Error creating metadata repository: {}", e),
    }
}

/// 显示表结构（使用 ORM 多数据库支持）
async fn show_table_schema(pool: &sqlx::AnyPool, table_name: &str) -> String {
    match create_db_meta_repository(pool) {
        Ok(repo) => {
            match repo.get_table_schema(table_name).await {
                Ok(schema) => {
                    let mut output = String::new();
                    output.push_str(&format!("Schema for table '{}':\n\n", schema.table_name));
                    
                    if schema.columns.is_empty() {
                        output.push_str("Table not found or has no columns.\n");
                        return output;
                    }
                    
                    // 显示列信息
                    output.push_str(&format!("{:>4} | {:-20} | {:>12} | {:>8} | {:>10} | {}\n", 
                        "CID", "Name", "Type", "Nullable", "Default", "PK"));
                    output.push_str(&format!("{:-4}-+-{:-20}-+-{:>-12}-+-{:>-8}-+-{:>-10}-+{}\n", 
                        "----", "--------------------", "------------", "--------", "----------", "--"));
                    
                    for col in &schema.columns {
                        output.push_str(&format!("{:>4} | {:-20} | {:>12} | {:>8} | {:>10} | {}\n", 
                            col.column_id,
                            col.name,
                            col.data_type,
                            if col.nullable { "YES" } else { "NO" },
                            col.default_value.as_deref().unwrap_or("NULL"),
                            if col.is_primary_key { "YES" } else { "NO" }));
                    }
                    
                    output.push_str(&format!("\nTotal: {} column(s)\n", schema.columns.len()));
                    
                    // 显示索引信息
                    if !schema.indexes.is_empty() {
                        output.push_str("\nIndexes:\n");
                        output.push_str(&format!("{:-30} | {:>6} | {}\n", 
                            "Index Name", "Unique", "Columns"));
                        output.push_str(&format!("{:-30}-+-{:>-6}-+{}\n", 
                              "------------------------------", "------", "------"));
                        
                        for idx in &schema.indexes {
                            let columns = idx.columns.join(", ");
                            output.push_str(&format!("{:-30} | {:>6} | {}\n", 
                                idx.index_name,
                                if idx.is_unique { "YES" } else { "NO" },
                                columns));
                        }
                        
                        output.push_str(&format!("\nTotal: {} index(es)\n", schema.indexes.len()));
                    } else {
                        output.push_str("\nNo indexes found.\n");
                    }
                    
                    // 显示行数
                    output.push_str(&format!("\nRow count: {}", schema.row_count));
                    
                    output
                }
                Err(e) => format!("Error getting schema for '{}': {}", table_name, e),
            }
        }
        Err(e) => format!("Error creating metadata repository: {}", e),
    }
}

/// 快速统计表的行数（使用 ORM 多数据库支持）
async fn count_table_rows(pool: &sqlx::AnyPool, table_name: &str) -> String {
    match create_db_meta_repository(pool) {
        Ok(repo) => {
            match repo.count_table_rows(table_name).await {
                Ok(count) => format!("Table '{}' contains {} row(s)", table_name, count),
                Err(e) => format!("Error counting rows from table '{}': {}", table_name, e),
            }
        }
        Err(e) => format!("Error creating metadata repository: {}", e),
    }
}

/// 创建数据库元信息仓库实例（根据数据库类型自动选择实现）
fn create_db_meta_repository(pool: &sqlx::AnyPool) -> Result<AnyDbMetaRepository, String> {
    // 从连接池 URL 检测数据库类型
    // 注意：这里简化处理，默认使用 SQLite
    // 在生产环境中，应该从配置或连接字符串中获取准确的数据库类型
    
    // 由于 AnyPool 不直接暴露连接 URL，
    // 我们暂时使用 SQLite 作为默认值
    // 未来可以通过在 ApiState 中存储 db_type 来改进
    let db_type = DatabaseType::SQLite;
    
    Ok(AnyDbMetaRepository::new(pool.clone(), db_type))
}

fn verify_admin_password(provided: &str, admin_password: &str, remote_host: &str) -> Result<(), String> {
    let mut map = INCORRECT_PASSWORDS.lock();
    let now = now_epoch_secs();

    if let Some(pc) = map.get(remote_host) {
        if pc.count >= MAX_INCORRECT_ATTEMPTS && now - pc.time < LOCK_DURATION_SECS {
            return Err("Too many incorrect admin password attempts, please try again later".to_string());
        }
    }

    if provided.is_empty() {
        return Err("Missing adminPassword parameter".to_string());
    }

    if provided != admin_password {
        let pc = map.entry(remote_host.to_string()).or_insert(PasswordCount { count: 0, time: 0 });
        pc.count += 1;
        pc.time = now;
        if map.len() > MAX_TRACKED_IPS {
            let key = map.keys().next().cloned();
            if let Some(k) = key {
                map.remove(&k);
            }
        }
        return Err("Incorrect adminPassword".to_string());
    }

    map.remove(remote_host);
    Ok(())
}

#[derive(Deserialize)]
pub struct DbShellQuery {
    #[serde(default)]
    pub admin_password: Option<String>,
    #[serde(default)]
    pub line: Option<String>,
    #[serde(default)]
    pub show_shell: Option<String>,
}

#[derive(Deserialize)]
pub struct DbShellForm {
    #[serde(default, rename = "adminPassword")]
    pub admin_password: Option<String>,
    #[serde(default)]
    pub line: Option<String>,
    #[serde(default, rename = "showShell")]
    pub show_shell: Option<String>,
}

pub async fn db_shell_get(
    State(state): State<ApiState>,
    Query(_params): Query<DbShellQuery>,
) -> Response {
    let disable_admin_password = is_admin_password_disabled(&state);
    let admin_password = get_admin_password();

    let body = if disable_admin_password {
        FORM.to_string()
    } else if admin_password.is_empty() {
        ERROR_NO_PASSWORD.to_string()
    } else {
        PASSWORD_FORM.to_string()
    };

    build_html_response(&body)
}

pub async fn db_shell_post(
    State(state): State<ApiState>,
    Form(params): Form<DbShellForm>,
) -> Response {
    let disable_admin_password = is_admin_password_disabled(&state);
    let admin_password = get_admin_password();

    if !disable_admin_password {
        if admin_password.is_empty() {
            return build_html_response(ERROR_NO_PASSWORD);
        }

        let provided = params.admin_password.as_deref().unwrap_or("");
        if let Err(err) = verify_admin_password(provided, &admin_password, "127.0.0.1") {
            let error_form = format_password_form(&format!("<p style=\"color:red\">{}</p>", err));
            return build_html_response(&error_form);
        }

        if params.show_shell.as_deref() == Some("true") {
            let encoded_pw = urlencoding::encode(provided);
            let shell_form = FORM.replace("{adminPassword}", &encoded_pw);
            return build_html_response(&shell_form);
        }
    }

    let line = params.line.as_deref().unwrap_or("");
    if line.is_empty() {
        return build_text_response("\n> ".to_string());
    }

    let result = execute_sql(&state, line).await;
    
    // 对于特殊命令，直接返回结果，不添加前缀
    let sql_trimmed = line.trim().to_lowercase();
    if sql_trimmed == "clear" || sql_trimmed == "save" || sql_trimmed == "history" || sql_trimmed == ".history" {
        build_text_response(result)
    } else {
        build_text_response(format!("\n> {}\n{}", line, result))
    }
}

fn is_admin_password_disabled(state: &ApiState) -> bool {
    std::env::var("NRCS_DISABLE_ADMIN_PASSWORD")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
        || state.db_pool.is_none()
}

fn get_admin_password() -> String {
    std::env::var("NRCS_ADMIN_PASSWORD").unwrap_or_default()
}

fn get_cell_value(row: &sqlx::any::AnyRow, col_name: &str) -> Option<String> {
    if let Ok(Some(v)) = row.try_get::<Option<String>, _>(col_name) {
        return Some(v);
    }
    if let Ok(v) = row.try_get::<String, _>(col_name) {
        return Some(v);
    }
    if let Ok(Some(v)) = row.try_get::<Option<i64>, _>(col_name) {
        return Some(v.to_string());
    }
    if let Ok(v) = row.try_get::<i64, _>(col_name) {
        return Some(v.to_string());
    }
    if let Ok(Some(v)) = row.try_get::<Option<f64>, _>(col_name) {
        return Some(v.to_string());
    }
    if let Ok(v) = row.try_get::<f64, _>(col_name) {
        return Some(v.to_string());
    }
    if let Ok(Some(v)) = row.try_get::<Option<bool>, _>(col_name) {
        return Some(if v { "1" } else { "0" }.to_string());
    }
    if let Ok(v) = row.try_get::<bool, _>(col_name) {
        return Some(if v { "1" } else { "0" }.to_string());
    }
    if let Ok(Some(v)) = row.try_get::<Option<Vec<u8>>, _>(col_name) {
        return Some(format!("{:?}", v));
    }
    if let Ok(v) = row.try_get::<Vec<u8>, _>(col_name) {
        return Some(format!("{:?}", v));
    }
    None
}

async fn execute_sql(state: &ApiState, sql: &str) -> String {
    let pool = match &state.db_pool {
        Some(p) => p.clone(),
        None => return "Error: Database pool not available".to_string(),
    };

    let sql_trimmed = sql.trim();
    
    // 处理特殊命令（不执行 SQL）
    match sql_trimmed.to_lowercase().as_str() {
        "help" => return HELP_TEXT.to_string(),
        "clear" => return "[CLEAR]".to_string(),
        "save" => return "[SAVE]".to_string(),
        "tables" | ".tables" => return list_tables(&pool).await,
        "history" | ".history" => return "[HISTORY]".to_string(),
        _ => {}
    }

    // 处理带参数的命令
    if sql_trimmed.eq_ignore_ascii_case("help") {
        return HELP_TEXT.to_string();
    }
    
    if sql_trimmed.eq_ignore_ascii_case("clear") {
        return "[CLEAR]".to_string();
    }

    if sql_trimmed.eq_ignore_ascii_case("save") {
        return "[SAVE]".to_string();
    }

    // tables 命令（支持别名）
    if sql_trimmed.eq_ignore_ascii_case("tables") 
        || sql_trimmed.eq_ignore_ascii_case(".tables")
        || sql_trimmed.eq_ignore_ascii_case("show tables") 
        || sql_trimmed.eq_ignore_ascii_case("\\dt") {
        return list_tables(&pool).await;
    }

    // schema / describe 命令（查看表结构）
    let sql_lower = sql_trimmed.to_lowercase();
    
    // 检查是否是 schema/describe 命令
    let is_schema_cmd = sql_lower.starts_with("schema ") 
        || sql_lower.starts_with("describe ")
        || sql_lower.starts_with("\\d ")
        || sql_lower.starts_with(".schema ");
        
    if is_schema_cmd {
        // 提取表名（取最后一个空格后的部分）
        let table_name = sql_trimmed.split_whitespace().last().unwrap_or("");
        if !table_name.is_empty() {
            return show_table_schema(&pool, table_name).await;
        }
    }

    // count 命令（快速统计行数）
    let is_count_cmd = sql_lower.starts_with("count ")
        || sql_lower.starts_with("count from ");
        
    if is_count_cmd {
        // 提取表名
        if let Some(table_name) = sql_trimmed.split_whitespace().last() {
            if table_name.eq_ignore_ascii_case("from") {
                // 处理 "count from table" 格式
                if let Some(actual_table) = sql_trimmed.split_whitespace().nth(2) {
                    return count_table_rows(&pool, actual_table).await;
                }
            } else {
                return count_table_rows(&pool, table_name).await;
            }
        }
    }

    // history 命令
    if sql_trimmed.eq_ignore_ascii_case("history") 
        || sql_trimmed.eq_ignore_ascii_case(".history") {
        return "[HISTORY]".to_string();
    }

    let sql_upper = sql_trimmed.to_uppercase();
    let is_query = sql_upper.starts_with("SELECT")
        || sql_upper.starts_with("PRAGMA")
        || sql_upper.starts_with("EXPLAIN")
        || sql_upper.starts_with("WITH")
        || sql_upper.starts_with("SHOW")
        || sql_upper.starts_with("DESCRIBE")
        || sql_upper.starts_with("TABLES");

    if is_query {
        match sqlx::query(sql_trimmed).fetch_all(&pool).await {
            Ok(rows) => {
                if rows.is_empty() {
                    return "(empty result set)".to_string();
                }
                let mut output = String::new();
                let cols = rows[0].columns();
                
                // 计算每列的最大宽度（基于列名和所有数据内容）
                let mut col_widths: Vec<usize> = cols.iter().map(|c| c.name().len()).collect();
                
                // 遍历所有行，更新每列的最大宽度
                for row in &rows {
                    for (i, col) in cols.iter().enumerate() {
                        let val = get_cell_value(row, col.name());
                        let display = val.unwrap_or_else(|| "NULL".to_string());
                        let display_len = display.len();
                        if display_len > col_widths[i] {
                            col_widths[i] = display_len;
                        }
                    }
                }
                
                // 确保最小宽度为8
                for w in col_widths.iter_mut() {
                    *w = (*w).max(8);
                }

                for (i, col) in cols.iter().enumerate() {
                    if i > 0 {
                        output.push_str(" | ");
                    }
                    let name = col.name();
                    let pad = col_widths[i].saturating_sub(name.len());
                    output.push_str(name);
                    for _ in 0..pad {
                        output.push(' ');
                    }
                }
                output.push('\n');

                for (i, w) in col_widths.iter().enumerate() {
                    if i > 0 {
                        output.push_str("-+-");
                    }
                    for _ in 0..*w {
                        output.push('-');
                    }
                }
                output.push('\n');

                for row in &rows {
                    for (i, col) in cols.iter().enumerate() {
                        if i > 0 {
                            output.push_str(" | ");
                        }
                        let val = get_cell_value(row, col.name());
                        let display = val.unwrap_or_else(|| "NULL".to_string());
                        let pad = col_widths[i].saturating_sub(display.len());
                        output.push_str(&display);
                        for _ in 0..pad {
                            output.push(' ');
                        }
                    }
                    output.push('\n');
                }

                output.push_str(&format!("({} row(s))", rows.len()));
                output
            }
            Err(e) => format!("Error: {}", e),
        }
    } else {
        match sqlx::query(sql_trimmed).execute(&pool).await {
            Ok(result) => {
                let affected = result.rows_affected();
                format!("Update count: {}", affected)
            }
            Err(e) => format!("Error: {}", e),
        }
    }
}

fn build_html_response(body: &str) -> Response {
    let html = format!("{}{}{}", HEADER, body, FOOTER);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=UTF-8")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate, private")
        .header(header::PRAGMA, "no-cache")
        .header(header::EXPIRES, "0")
        .body(html.into())
        .unwrap()
}

fn build_text_response(text: String) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=UTF-8")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate, private")
        .header(header::PRAGMA, "no-cache")
        .header(header::EXPIRES, "0")
        .body(text.into())
        .unwrap()
}

fn format_password_form(message: &str) -> String {
    format!(
        "<form action=\"/dbshell\" method=\"POST\">\
         <table class=\"table\">\
         <tr><td colspan=\"3\">{}</td></tr>\
         <tr>\
         <td>Password:</td>\
         <td><input type=\"password\" name=\"adminPassword\"/>\
         <input type=\"submit\" value=\"Go!\"/></td>\
         </tr>\
         </table>\
         <input type=\"hidden\" name=\"showShell\" value=\"true\"/>\
         </form>",
        message
    )
}

const HELP_TEXT: &str = r#"╔════════════════════════════════════════════════════╗
║           NRCS Database Shell Help                    ║
╠════════════════════════════════════════════════════╣
║                                                      ║
║  📝 SQL Commands (Standard):                        ║
║    SELECT ...          - Query data                 ║
║    INSERT/UPDATE/DELETE - Modify data               ║
║    CREATE TABLE ...     - Create new table           ║
║    DROP TABLE ...       - Drop existing table        ║
║    PRAGMA ...          - SQLite configuration        ║
║    EXPLAIN ...         - Query execution plan       ║
║                                                      ║
║  🔧 Special Commands (Meta Information):             ║
║    tables / .tables     - List all tables            ║
║    show tables          - Alias for tables           ║
║    \dt                  - PostgreSQL style (alias)    ║
║    schema <table>       - Show table structure       ║
║    describe <table>     - Alias for schema           ║
║    \d <table>           - PostgreSQL style (alias)    ║
║    count <table>        - Quick row count            ║
║    .schema <table>      - SQLite style (alias)       ║
║                                                      ║
║  💾 Utility Commands (Shell Operations):             ║
║    help                 - Show this help text        ║
║    clear                - Clear the result area      ║
║    save                 - Save results to file       ║
║    history / .history   - Show command history      ║
║                                                      ║
║  📌 Usage Examples:                                   ║
║    > SELECT * FROM block LIMIT 10;                  ║
║    > tables                                         ║
║    > describe account                                ║
║    > count transaction                               ║
║    > clear                                          ║
║    > save                                           ║
║                                                      ║
╚════════════════════════════════════════════════════╝

Type any SQL statement or special command above and press Enter.
For more information, visit the NRCS documentation.
"#;

const HEADER: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8"/>
    <title>Nrcs Database Shell</title>
    <script type="text/javascript">
        // 全局命令历史存储
        var dbshellHistory = [];
        var maxHistorySize = 100;
        
        function submitForm(form, adminPassword) {
            var url = '/dbshell';
            var params = '';
            var sqlInput = '';
            
            for (var i = 0; i < form.elements.length; i++) {
                if (!form.elements[i].name) {
                    continue;
                }
                if (params.length > 0) {
                    params += '&';
                }
                params += encodeURIComponent(form.elements[i].name);
                params += '=';
                params += encodeURIComponent(form.elements[i].value);
                
                // 记录 SQL 输入（用于历史记录）
                if (form.elements[i].name === 'line') {
                    sqlInput = form.elements[i].value.trim();
                }
            }
            if (adminPassword && form.elements.length > 0) {
                params += '&adminPassword=' + adminPassword;
            }
            
            // 将非空命令添加到历史记录
            if (sqlInput && sqlInput.length > 0) {
                addToHistory(sqlInput);
            }
            
            var request = new XMLHttpRequest();
            request.open("POST", url, false);
            request.setRequestHeader("Content-type", "application/x-www-form-urlencoded");
            request.send(params);

            var responseText = request.responseText;

            // 处理特殊命令
            if (responseText.trim() === '[CLEAR]') {
                form.getElementsByClassName('result')[0].textContent = 'This is a database shell. Enter SQL to be evaluated, or "help" for help:\n';
                return false;
            }

            if (responseText.trim() === '[SAVE]') {
                var resultContent = form.getElementsByClassName('result')[0].textContent;
                var blob = new Blob([resultContent], { type: 'text/plain;charset=utf-8' });
                var url = window.URL.createObjectURL(blob);
                var link = document.createElement('a');
                link.href = url;
                link.download = 'dbshell_result_' + new Date().toISOString().slice(0,19).replace(/:/g,'-') + '.txt';
                document.body.appendChild(link);
                link.click();
                document.body.removeChild(link);
                window.URL.revokeObjectURL(url);
                form.getElementsByClassName('result')[0].textContent += '\n> Result saved to file.\n';
                return false;
            }

            if (responseText.trim() === '[HISTORY]') {
                displayHistory(form);
                return false;
            }

            form.getElementsByClassName("result")[0].textContent += responseText;
            return false;
        }
        
        // 添加到历史记录
        function addToHistory(command) {
            // 避免重复添加相同的连续命令
            if (dbshellHistory.length > 0 && dbshellHistory[dbshellHistory.length - 1] === command) {
                return;
            }
            
            dbshellHistory.push(command);
            
            // 限制历史记录大小
            if (dbshellHistory.length > maxHistorySize) {
                dbshellHistory.shift();
            }
        }
        
        // 显示历史记录
        function displayHistory(form) {
            var resultArea = form.getElementsByClassName('result')[0];
            
            if (dbshellHistory.length === 0) {
                resultArea.textContent += '\n> No command history yet.\n';
                return;
            }
            
            // 计算最长的命令长度
            var maxCmdLen = 0;
            for (var i = 0; i < dbshellHistory.length; i++) {
                if (dbshellHistory[i].length > maxCmdLen) {
                    maxCmdLen = dbshellHistory[i].length;
                }
            }
            
            // 计算合适的宽度（至少 50，命令长度 + 15 左右的边距）
            var contentWidth = Math.max(50, maxCmdLen + 15);
            
            // 绘制顶部边框
            resultArea.textContent += '\n╔';
            for (var i = 0; i < contentWidth; i++) { resultArea.textContent += '═'; }
            resultArea.textContent += '╗\n';
            
            // 标题行
            var title = '       Command History       ';
            var titlePadLeft = Math.floor((contentWidth - title.length) / 2);
            var titlePadRight = contentWidth - title.length - titlePadLeft;
            resultArea.textContent += '║' + ' '.repeat(titlePadLeft) + title + ' '.repeat(titlePadRight) + '║\n';
            
            // 分隔线
            resultArea.textContent += '╠';
            for (var i = 0; i < contentWidth; i++) { resultArea.textContent += '═'; }
            resultArea.textContent += '╣\n';
            
            // 内容行
            for (var i = 0; i < dbshellHistory.length; i++) {
                var num = String(i + 1).padStart(3, ' ');
                var lineContent = num + '. ' + dbshellHistory[i];
                var padLen = contentWidth - lineContent.length;
                resultArea.textContent += '║ ' + lineContent + ' '.repeat(Math.max(0, padLen - 1)) + ' ║\n';
            }
            
            // 底部边框
            resultArea.textContent += '╚';
            for (var i = 0; i < contentWidth; i++) { resultArea.textContent += '═'; }
            resultArea.textContent += '╝\n';
            
            resultArea.textContent += '\n> Total: ' + dbshellHistory.length + ' command(s)\n';
        }
    </script>
    <style type="text/css">
        body { font-family: monospace; margin: 20px; background: #1e1e1e; color: #d4d4d4; }
        pre.result { background: #000; color: #0f0; padding: 10px; height: calc(100vh - 150px); white-space: pre; overflow-x: auto; overflow-y: auto; border: 1px solid #333; }
        input[type="text"] { font-family: monospace; font-size: 14px; background: #2d2d2d; color: #d4d4d4; border: 1px solid #555; padding: 5px; }
        input[type="password"] { font-family: monospace; font-size: 14px; background: #2d2d2d; color: #d4d4d4; border: 1px solid #555; padding: 5px; }
        input[type="submit"] { font-family: monospace; font-size: 14px; background: #0e639c; color: #fff; border: none; padding: 5px 15px; cursor: pointer; }
        input[type="submit"]:hover { background: #1177bb; }
        table { border-collapse: collapse; width: 90%; }
        td { padding: 5px; }
        b { color: #569cd6; }
        a { color: #569cd6; }
    </style>
</head>
<body>
"#;

const FOOTER: &str = r#"</body>
</html>
"#;

const FORM: &str = r#"<form action="/dbshell" method="POST" onsubmit="return submitForm(this, '{adminPassword}');">
<table class="table" style="width:90%;">
<tr><td><pre class="result" style="float:top;width:90%;">This is a database shell. Enter SQL to be evaluated, or "help" for help:
</pre></td></tr>
<tr><td><b>&gt;</b> <input type="text" name="line" style="width:90%;" autofocus/></td></tr>
</table>
</form>"#;

const ERROR_NO_PASSWORD: &str = "This page is password-protected, but no password is configured. \
    Please set NRCS_ADMIN_PASSWORD environment variable or disable the password protection with \
    NRCS_DISABLE_ADMIN_PASSWORD=true";

const PASSWORD_FORM: &str = r#"<form action="/dbshell" method="POST">
<table class="table">
<tr><td colspan="3"><p>This page is password-protected. Please enter the administrator's password</p></td></tr>
<tr>
<td>Password:</td>
<td><input type="password" name="adminPassword"/>
<input type="submit" value="Go!"/></td>
</tr>
</table>
<input type="hidden" name="showShell" value="true"/>
</form>"#;
