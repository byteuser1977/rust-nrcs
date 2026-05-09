//! Database Shell HTTP handler
//!
//! 与 Java 版本 DbShellServlet 完全对齐
//! 参考: com.bytechain.nrcs.http.DbShellServlet
//!
//! 提供基于 Web 的数据库 SQL 交互终端，支持：
//! - 密码保护（含防暴力破解锁定）
//! - 任意 SQL 执行（SELECT/INSERT/UPDATE/DELETE 等）
//! - 终端式连续交互体验

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
    build_text_response(format!("\n> {}\n{}", line, result))
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
    if sql_trimmed.eq_ignore_ascii_case("help") {
        return HELP_TEXT.to_string();
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
                let col_widths: Vec<usize> = cols
                    .iter()
                    .map(|c| c.name().len().max(8))
                    .collect();

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

const HELP_TEXT: &str = r#"NRCS Database Shell Help:
  Enter any SQL statement to execute it against the database.
  Supported commands:
    SELECT ...          - Query data
    INSERT/UPDATE/DELETE - Modify data
    PRAGMA ...          - SQLite configuration (SQLite only)
    SHOW TABLES         - List tables
    help                - Show this help text
"#;

const HEADER: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8"/>
    <title>Nrcs Database Shell</title>
    <script type="text/javascript">
        function submitForm(form, adminPassword) {
            var url = '/dbshell';
            var params = '';
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
            }
            if (adminPassword && form.elements.length > 0) {
                params += '&adminPassword=' + adminPassword;
            }
            var request = new XMLHttpRequest();
            request.open("POST", url, false);
            request.setRequestHeader("Content-type", "application/x-www-form-urlencoded");
            request.send(params);
            form.getElementsByClassName("result")[0].textContent += request.responseText;
            return false;
        }
    </script>
    <style type="text/css">
        body { font-family: monospace; margin: 20px; background: #1e1e1e; color: #d4d4d4; }
        pre.result { background: #000; color: #0f0; padding: 10px; min-height: 200px; white-space: pre-wrap; word-wrap: break-word; border: 1px solid #333; }
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
