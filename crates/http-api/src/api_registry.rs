//! API 注册表
//!
//! 管理所有 API Handler 的注册和查找

use crate::api_tag::ApiTag;
use crate::request_handler::HandlerPtr;
use std::collections::{HashMap, HashSet};
use parking_lot::RwLock;

/// 归一化 requestType，使 API 注册/查找对大小写不敏感
/// "GetAccount" → "getAccount", "getAccount" 保持不变
fn normalize_request_type(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut result = String::with_capacity(name.len());
            result.push(first.to_lowercase().next().unwrap_or(first));
            result.push_str(chars.as_str());
            result
        }
    }
}

pub struct ApiRegistry {
    handlers: RwLock<HashMap<String, HandlerPtr>>,
    disabled_handlers: RwLock<HashMap<String, HandlerPtr>>,
    tags_map: RwLock<HashMap<ApiTag, Vec<String>>>,
    disabled_apis: HashSet<String>,
    disabled_tags: HashSet<ApiTag>,
}

impl ApiRegistry {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
            disabled_handlers: RwLock::new(HashMap::new()),
            tags_map: RwLock::new(HashMap::new()),
            disabled_apis: HashSet::new(),
            disabled_tags: HashSet::new(),
        }
    }
    
    pub fn register(&self, name: &str, handler: HandlerPtr) {
        let normalized = normalize_request_type(name);
        let tags = handler.api_tags();

        if self.disabled_apis.contains(&normalized) {
            self.disabled_handlers.write().insert(normalized, handler);
            return;
        }

        for tag in &tags {
            if self.disabled_tags.contains(tag) {
                self.disabled_handlers.write().insert(normalized, handler);
                return;
            }
        }

        self.handlers.write().insert(normalized, handler);

        for tag in tags {
            self.tags_map
                .write()
                .entry(tag)
                .or_default()
                .push(name.to_string());
        }
    }

    pub fn get_handler(&self, name: &str) -> Option<HandlerPtr> {
        let normalized = normalize_request_type(name);
        self.handlers.read().get(&normalized).cloned()
    }
    
    pub fn get_disabled_handler(&self, name: &str) -> Option<HandlerPtr> {
        let normalized = normalize_request_type(name);
        self.disabled_handlers.read().get(&normalized).cloned()
    }

    pub fn is_disabled(&self, name: &str) -> bool {
        let normalized = normalize_request_type(name);
        self.disabled_handlers.read().contains_key(&normalized)
    }
    
    pub fn get_apis_by_tag(&self, tag: ApiTag) -> Vec<String> {
        self.tags_map.read().get(&tag).cloned().unwrap_or_default()
    }
    
    pub fn all_api_names(&self) -> Vec<String> {
        self.handlers.read().keys().cloned().collect()
    }
    
    pub fn all_handlers(&self) -> HashMap<String, HandlerPtr> {
        self.handlers.read().clone()
    }
    
    pub fn all_tags(&self) -> Vec<ApiTag> {
        self.tags_map.read().keys().copied().collect()
    }
    
    pub fn set_disabled_apis(&mut self, apis: HashSet<String>) {
        self.disabled_apis = apis;
    }
    
    pub fn set_disabled_tags(&mut self, tags: HashSet<ApiTag>) {
        self.disabled_tags = tags;
    }
}

impl Default for ApiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub static API_REGISTRY: once_cell::sync::Lazy<ApiRegistry> = once_cell::sync::Lazy::new(ApiRegistry::new);

pub fn register_api(name: &str, handler: HandlerPtr) {
    API_REGISTRY.register(name, handler);
}

pub fn get_api_handler(name: &str) -> Option<HandlerPtr> {
    API_REGISTRY.get_handler(name)
}

pub fn get_all_handlers() -> HashMap<String, HandlerPtr> {
    API_REGISTRY.all_handlers()
}

pub fn get_apis_by_tag(tag: ApiTag) -> Vec<String> {
    API_REGISTRY.get_apis_by_tag(tag)
}

pub fn is_api_disabled(name: &str) -> bool {
    API_REGISTRY.is_disabled(name)
}
