//! 传统 API 路由处理器
//!
//! 兼容 Java 版本的 requestType 参数路由模式

use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::time::Instant;

use crate::api_registry::{get_api_handler, is_api_disabled};
use crate::error::ApiError;
use crate::request_handler::{responses, ApiRequest, RsRespWithData};
use crate::state::ApiState;

pub async fn handle_nrcs_api(
    State(state): State<ApiState>,
    method: Method,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> Response {
    let start_time = Instant::now();
    
    let mut params = query;
    
    if method == Method::POST {
        if let Ok(body_str) = std::str::from_utf8(&body) {
            if let Ok(form_params) = serde_urlencoded::from_str::<HashMap<String, String>>(body_str) {
                params.extend(form_params);
            }
        }
    }
    
    let request_type = match params.get("requestType") {
        Some(rt) => rt.clone(),
        None => {
            return json_response(responses::incorrect_request(), start_time.elapsed().as_millis() as u64);
        }
    };
    
    let handler = match get_api_handler(&request_type) {
        Some(h) => h,
        None => {
            if is_api_disabled(&request_type) {
                return json_response(responses::disabled_api(), start_time.elapsed().as_millis() as u64);
            } else {
                return json_response(responses::incorrect_request(), start_time.elapsed().as_millis() as u64);
            }
        }
    };
    
    if handler.require_post() && method != Method::POST {
        return json_response(responses::post_required(), start_time.elapsed().as_millis() as u64);
    }
    
    let api_request = ApiRequest::new(params);
    
    match handler.process_request(&api_request, &state).await {
        Ok(mut resp) => {
            resp.base.request_processing_time = start_time.elapsed().as_millis() as u64;
            json_response(resp, start_time.elapsed().as_millis() as u64)
        }
        Err(e) => {
            let resp = error_to_response(&e);
            json_response(resp, start_time.elapsed().as_millis() as u64)
        }
    }
}

fn error_to_response(e: &ApiError) -> RsRespWithData {
    match e {
        ApiError::MissingParameter(p) => responses::missing_parameter(p),
        ApiError::IncorrectValue(p) => responses::incorrect_value(p),
        ApiError::UnknownAccount => responses::unknown_account(),
        ApiError::UnknownBlock => responses::unknown_block(),
        ApiError::UnknownTransaction => responses::unknown_transaction(),
        ApiError::IncorrectAccount => responses::incorrect_account(),
        ApiError::IncorrectBlock => responses::incorrect_block(),
        ApiError::IncorrectHeight => responses::incorrect_height(),
        ApiError::IncorrectTimestamp => responses::incorrect_timestamp(),
        ApiError::NotFound(msg) => RsRespWithData::from(crate::request_handler::RsResp::error(6, msg.clone())),
        ApiError::Validation(msg) => RsRespWithData::from(crate::request_handler::RsResp::error(4, msg.clone())),
        ApiError::Unauthorized(msg) => RsRespWithData::from(crate::request_handler::RsResp::error(2, msg.clone())),
        ApiError::Internal(msg) => RsRespWithData::from(crate::request_handler::RsResp::error(4, msg.clone())),
        ApiError::Blockchain(err) => RsRespWithData::from(crate::request_handler::RsResp::error(4, err.to_string())),
        ApiError::Repository(err) => RsRespWithData::from(crate::request_handler::RsResp::error(4, err.to_string())),
        ApiError::Account(err) => RsRespWithData::from(crate::request_handler::RsResp::error(4, err.to_string())),
        ApiError::TxEngine(err) => RsRespWithData::from(crate::request_handler::RsResp::error(4, err.to_string())),
        ApiError::Io(err) => RsRespWithData::from(crate::request_handler::RsResp::error(4, err.to_string())),
    }
}

fn json_response(resp: RsRespWithData, _processing_time: u64) -> Response {
    let json = serde_json::to_string(&resp).unwrap_or_default();
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=UTF-8")
        .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate, private")
        .header(header::PRAGMA, "no-cache")
        .header(header::EXPIRES, "0")
        .body(json.into())
        .unwrap()
}

pub async fn handle_nrcs_get(
    State(state): State<ApiState>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    handle_nrcs_api(State(state), Method::GET, Query(query), Bytes::new()).await
}

pub async fn handle_nrcs_post(
    State(state): State<ApiState>,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> Response {
    handle_nrcs_api(State(state), Method::POST, Query(query), body).await
}
