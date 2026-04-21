//! API 测试页面
//!
//! 与 Java 版本 APITestServlet 完全对齐

use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};
use std::collections::HashMap;

use crate::api_registry::{get_all_handlers, get_apis_by_tag};
use crate::api_tag::ApiTag;

pub async fn api_test_page(
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let request_tag = params.get("requestTag").map(|s| s.as_str()).unwrap_or("");
    let request_type = params.get("requestType");
    
    let html = generate_test_html(request_tag, request_type);
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=UTF-8")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate, private")
        .header(header::PRAGMA, "no-cache")
        .header(header::EXPIRES, "0")
        .body(html.into())
        .unwrap()
}

fn generate_test_html(request_tag: &str, request_type: Option<&String>) -> String {
    let mut html = String::new();
    
    html.push_str(HEADER_1);
    html.push_str(&build_links(request_tag));
    html.push_str(HEADER_2);
    
    if let Some(rt) = request_type {
        if let Some(handler) = get_all_handlers().get(rt) {
            html.push_str(&form(rt, true, handler.parameters(), handler.require_post(), handler.file_parameter()));
        }
    } else {
        let apis = if request_tag.is_empty() {
            get_all_handlers()
                .keys()
                .map(|s| s.clone())
                .collect::<Vec<_>>()
        } else {
            if let Ok(tag) = parse_tag(request_tag) {
                get_apis_by_tag(tag)
            } else {
                get_all_handlers()
                    .keys()
                    .map(|s| s.clone())
                    .collect::<Vec<_>>()
            }
        };
        
        let mut apis_sorted = apis;
        apis_sorted.sort();
        
        for api_name in apis_sorted {
            if let Some(handler) = get_all_handlers().get(&api_name) {
                html.push_str(&form(&api_name, false, handler.parameters(), handler.require_post(), handler.file_parameter()));
            }
        }
    }
    
    html.push_str(FOOTER_1);
    html.push_str(&build_js_calls(request_type, request_tag));
    html.push_str(FOOTER_2);
    
    html
}

fn parse_tag(s: &str) -> Result<ApiTag, ()> {
    for tag in ApiTag::all() {
        if tag.display_name() == s || format!("{:?}", tag) == s {
            return Ok(*tag);
        }
    }
    Err(())
}

fn build_links(request_tag: &str) -> String {
    let mut buf = String::new();
    
    buf.push_str("<li");
    if request_tag.is_empty() {
        buf.push_str(" class='active'");
    }
    buf.push_str("><a href='/test'>ALL</a></li>\n");
    
    buf.push_str("<li><a href='/test?requestTypes=' id='navi-selected'>SELECTED</a></li>\n");
    
    for tag in ApiTag::all() {
        let apis = get_apis_by_tag(*tag);
        if !apis.is_empty() {
            buf.push_str("<li");
            if request_tag == tag.display_name() {
                buf.push_str(" class='active'");
            }
            buf.push_str("><a href='/test?requestTag=");
            buf.push_str(tag.display_name());
            buf.push_str("'>");
            buf.push_str(tag.display_name());
            buf.push_str("</a></li>\n");
        }
    }
    
    buf
}

fn form(
    request_type: &str,
    single_view: bool,
    parameters: Vec<&'static str>,
    require_post: bool,
    file_parameter: Option<&'static str>,
) -> String {
    let mut buf = String::new();
    
    buf.push_str("<div class='panel panel-default api-call-All' ");
    buf.push_str("id='api-call-");
    buf.push_str(request_type);
    buf.push_str("'>\n");
    
    buf.push_str("<div class='panel-heading'>\n");
    buf.push_str("<h4 class='panel-title'>\n");
    buf.push_str("<a data-toggle='collapse' class='collapse-link' data-target='#collapse");
    buf.push_str(request_type);
    buf.push_str("' href='#'>");
    buf.push_str(request_type);
    buf.push_str("</a>\n");
    
    buf.push_str("<span style='float:right;font-weight:normal;font-size:14px;'>\n");
    if !single_view {
        buf.push_str("<a href='/test?requestType=");
        buf.push_str(request_type);
        buf.push_str("' target='_blank' style='font-weight:normal;font-size:14px;color:#777;'>\n<span class='glyphicon glyphicon-new-window'></span>\n</a>");
        buf.push_str(" &nbsp;&nbsp;\n");
    }
    
    buf.push_str("&nbsp;&nbsp;&nbsp;\n<input type='checkbox' class='api-call-sel-ALL' ");
    buf.push_str("id='api-call-sel-");
    buf.push_str(request_type);
    buf.push_str("'>\n");
    buf.push_str("</span>\n");
    buf.push_str("</h4>\n");
    buf.push_str("</div> <!-- panel-heading -->\n");
    
    buf.push_str("<div id='collapse");
    buf.push_str(request_type);
    buf.push_str("' class='panel-collapse collapse");
    if single_view {
        buf.push_str(" in");
    }
    buf.push_str("'>\n");
    
    buf.push_str("<div class='panel-body'>\n");
    buf.push_str("<form action='/nrcs' method='POST' ");
    if file_parameter.is_some() {
        buf.push_str("enctype='multipart/form-data' ");
    }
    buf.push_str("onsubmit='return ATS.submitForm(this)'>\n");
    
    buf.push_str("<input type='hidden' name='requestType' value='");
    buf.push_str(request_type);
    buf.push_str("'/>\n");
    
    buf.push_str("<div class='col-xs-12 col-lg-6' style='min-width: 40%;'>\n");
    buf.push_str("<table class='table'>\n");
    
    if let Some(fp) = file_parameter {
        buf.push_str("<tr class='api-call-input-tr'>\n");
        buf.push_str("<td>");
        buf.push_str(fp);
        buf.push_str(":</td>\n");
        buf.push_str("<td><input type='file' name='");
        buf.push_str(fp);
        buf.push_str("' id='");
        buf.push_str(fp);
        buf.push_str(request_type);
        buf.push_str("' style='width:100%;min-width:200px;'/></td>\n");
        buf.push_str("</tr>\n");
    }
    
    for parameter in &parameters {
        buf.push_str("<tr class='api-call-input-tr'>\n");
        buf.push_str("<td>");
        buf.push_str(parameter);
        buf.push_str(":</td>\n");
        
        if is_textarea(parameter) {
            buf.push_str("<td><textarea name='");
            buf.push_str(parameter);
            buf.push_str("' style='width:100%;min-width:200px;'></textarea></td>\n");
        } else if is_password(parameter) {
            buf.push_str("<td><input type='password' name='");
            buf.push_str(parameter);
            buf.push_str("' style='width:100%;min-width:200px;'/></td>\n");
        } else {
            buf.push_str("<td><input type='text' name='");
            buf.push_str(parameter);
            buf.push_str("' style='width:100%;min-width:200px;'/></td>\n");
        }
        
        buf.push_str("</tr>\n");
    }
    
    buf.push_str("<tr>\n");
    buf.push_str("<td colspan='2'><input type='submit' class='btn btn-default' value='submit'/></td>\n");
    buf.push_str("</tr>\n");
    buf.push_str("</table>\n");
    buf.push_str("</div>\n");
    
    buf.push_str("<div class='col-xs-12 col-lg-6' style='min-width: 50%;'>\n");
    buf.push_str("<h5 style='margin-top:0px;'>\n");
    if require_post {
        buf.push_str("<span style='float:right;font-size:12px;font-weight:normal;'>POST only</span>\n");
    } else {
        buf.push_str("<span style='float:right;' class='uri-link'></span>\n");
    }
    buf.push_str("Response</h5>\n");
    buf.push_str("<pre class='hljs json'><code class='result'>JSON response</code></pre>\n");
    buf.push_str("</div>\n");
    
    buf.push_str("</form>\n");
    buf.push_str("</div> <!-- panel-body -->\n");
    buf.push_str("</div> <!-- panel-collapse -->\n");
    buf.push_str("</div> <!-- panel -->\n");
    
    buf
}

fn is_password(parameter: &str) -> bool {
    parameter == "secretPhrase" || parameter == "adminPassword" || parameter == "recipientSecretPhrase"
}

fn is_textarea(parameter: &str) -> bool {
    parameter == "website"
}

fn build_js_calls(request_type: Option<&String>, _request_tag: &str) -> String {
    let mut buf = String::new();
    
    buf.push_str("\n    $('#nodeType').val('Full Node');\n");
    buf.push_str("    $('#servletPath').val('/test');\n");
    
    if let Some(rt) = request_type {
        buf.push_str("    ATS.apiCalls.push('");
        buf.push_str(rt);
        buf.push_str("');\n");
    }
    
    buf
}

const HEADER_1: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset='UTF-8'/>
    <meta http-equiv='X-UA-Compatible' content='IE=edge'>
    <meta name='viewport' content='width=device-width, initial-scale=1'>
    <title>Nrcs http API</title>
    <link href='ui/css/bootstrap.min.css' rel='stylesheet' type='text/css' />
    <link href='ui/css/font-awesome.min.css' rel='stylesheet' type='text/css' />
    <link href='ui/css/highlight.style.css' rel='stylesheet' type='text/css' />
    <style type='text/css'>
        table {border-collapse: collapse;}
        td {padding: 10px;}
        .result {white-space: pre; font-family: monospace; overflow: auto;}
    </style>
</head>
<body>
<div class='navbar navbar-default' role='navigation'>
   <div class='container' style='min-width: 90%;'>
       <div class='navbar-header'>
           <a class='navbar-brand' href='/test'>Nrcs http API</a>
       </div>
       <div class='navbar-collapse collapse'>
           <ul class='nav navbar-nav navbar-right'>
               <li><input type='text' class='form-control' id='nodeType' readonly style='margin-top:8px;'></li>
               <li><input type='text' class='form-control' id='servletPath' readonly style='margin-top:8px;'></li>
               <li><input type='text' class='form-control' id='search' placeholder='Search' style='margin-top:8px;'></li>
           </ul>
       </div>
   </div>
</div>
<div class='container' style='min-width: 90%;'>
<div class='row'>
  <div class='col-xs-12' style='margin-bottom:10px;'>
    <div class='pull-right'>
      <div class='btn-group'>
        <button type='button' class='btn btn-default btn-sm dropdown-toggle' data-toggle='dropdown'>
          <i class='fa fa-check-circle-o'></i> <i class='fa fa-circle-o'></i>
        </button>
        <ul class='dropdown-menu' role='menu' style='font-size:12px;'>
          <li><a href='#' id='navi-select-all-d-add-btn'>Select All Displayed (Add)</a></li>
          <li><a href='#' id='navi-select-all-d-replace-btn'>Select All Displayed (Replace)</a></li>
          <li><a href='#' id='navi-deselect-all-d-btn'>Deselect All Displayed</a></li>
          <li><a href='#' id='navi-deselect-all-btn'>Deselect All</a></li>
        </ul>
      </div>
      <button type='button' id='navi-show-fields' data-navi-val='ALL' class='btn btn-default btn-sm' style='width:165px;'>Show Non-Empty Fields</button>
      <button type='button' id='navi-show-tabs' data-navi-val='ALL' class='btn btn-default btn-sm' style='width:130px;'>Show Open Tabs</button>
    </div>
  </div>
</div>
<div class='row' style='margin-bottom:15px;'>
<div class='col-xs-4 col-sm-3 col-md-2'>
<ul class='nav nav-pills nav-stacked'>
"#;

const HEADER_2: &str = r#"</ul>
</div> <!-- col -->
<div  class='col-xs-8 col-sm-9 col-md-10'>
<div class='panel-group' id='accordion'>
"#;

const FOOTER_1: &str = r#"</div> <!-- panel-group -->
</div> <!-- col -->
</div> <!-- row -->
</div> <!-- container -->
<script src='ui/js/3rdparty/jquery.js'></script>
<script src='ui/js/3rdparty/bootstrap.js' type='text/javascript'></script>
<script src='ui/js/3rdparty/highlight.pack.js' type='text/javascript'></script>
<script>
var ATS = {
    apiCalls: [],
    submitForm: function(form) {
        var formData = new FormData(form);
        var resultEl = $(form).find('.result');
        resultEl.text('Loading...');
        
        $.ajax({
            url: $(form).attr('action'),
            type: 'POST',
            data: formData,
            processData: false,
            contentType: false,
            success: function(data) {
                resultEl.text(JSON.stringify(data, null, 2));
                hljs.highlightBlock(resultEl[0]);
            },
            error: function(xhr, status, error) {
                resultEl.text('Error: ' + error);
            }
        });
        return false;
    }
};

$(document).ready(function() {
"#;

const FOOTER_2: &str = r#"});
</script>
</body>
</html>
"#;
