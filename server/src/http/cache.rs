use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::http::{Request, Response};
use redis::{AsyncCommands, Client};
use regex::Regex;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tower::{Layer, Service, ServiceExt};

/// Redis cache layer for GraphQL requests
#[derive(Clone)]
pub struct CacheLayer {
    cache_client: Client,
    ttl_seconds: u64,
    skip_pattern: Option<Regex>,
}

impl CacheLayer {
    pub fn new(
        cache_url: &str,
        ttl_seconds: u64,
        skip_pattern: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let cache_client = Client::open(cache_url)?;

        // Compile regex pattern if provided
        let compiled_pattern = match skip_pattern {
            Some(pattern) => {
                let regex = Regex::new(&pattern).map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;
                Some(regex)
            }
            None => None,
        };

        Ok(Self {
            cache_client,
            ttl_seconds,
            skip_pattern: compiled_pattern,
        })
    }
}

impl<S> Layer<S> for CacheLayer {
    type Service = CacheService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CacheService {
            inner,
            cache_client: self.cache_client.clone(),
            ttl_seconds: self.ttl_seconds,
            skip_pattern: self.skip_pattern.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CacheService<S> {
    inner: S,
    cache_client: Client,
    ttl_seconds: u64,
    skip_pattern: Option<Regex>,
}

impl<S> Service<Request<Body>> for CacheService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
{
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let inner = self.inner.clone();
        let cache_client = self.cache_client.clone();
        let ttl_seconds = self.ttl_seconds;
        let skip_pattern = self.skip_pattern.clone();

        Box::pin(async move {
            // Only cache GraphQL POST requests
            if req.method() != axum::http::Method::POST {
                return inner.oneshot(req).await;
            }

            // Extract the request body to generate cache key
            let (parts, body) = req.into_parts();
            let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
                Ok(bytes) => bytes,
                Err(_) => {
                    // If we can't read the body, skip caching and forward the request
                    let req = Request::from_parts(parts, Body::empty());
                    return inner.oneshot(req).await;
                }
            };

            // Parse GraphQL request to extract query for cache key generation
            let cache_key = match generate_cache_key(&body_bytes, &skip_pattern) {
                Some(key) => key,
                None => {
                    // If we can't generate a cache key, skip caching
                    let req = Request::from_parts(parts, Body::from(body_bytes));
                    return inner.oneshot(req).await;
                }
            };

            // Try to get cached response
            let mut cache_conn = match cache_client.get_multiplexed_async_connection().await {
                Ok(conn) => conn,
                Err(_) => {
                    // If Redis connection fails, skip caching and forward the request
                    let req = Request::from_parts(parts, Body::from(body_bytes));
                    return inner.oneshot(req).await;
                }
            };

            // Check cache
            if let Ok(cached_response) = cache_conn.get::<_, String>(&cache_key).await {
                if let Ok(_response_data) = serde_json::from_str::<Value>(&cached_response) {
                    // Return cached response
                    let response = Response::builder()
                        .status(200)
                        .header("content-type", "application/json")
                        .header("x-cache", "HIT")
                        .body(Body::from(cached_response))
                        .unwrap();
                    return Ok(response);
                }
            }

            // Cache miss - forward request to GraphQL handler
            let req = Request::from_parts(parts, Body::from(body_bytes));
            let response = inner.oneshot(req).await?;

            // Cache successful GraphQL responses
            if response.status().is_success() {
                let (parts, body) = response.into_parts();

                // Try to read the body for caching
                match axum::body::to_bytes(body, usize::MAX).await {
                    Ok(response_body) => {
                        let response_str = String::from_utf8_lossy(&response_body);

                        // Only cache if it's a valid JSON response without errors
                        if let Ok(json_response) = serde_json::from_str::<Value>(&response_str) {
                            if !check_has_errors(&json_response) {
                                // Cache the response
                                let _: Result<(), redis::RedisError> =
                                    cache_conn.set_ex(&cache_key, response_str.as_ref(), ttl_seconds).await;
                            }
                        }

                        // Reconstruct response with cached indicator
                        let mut response = Response::from_parts(parts, Body::from(response_body));
                        response.headers_mut().insert("x-cache", "MISS".parse().unwrap());
                        Ok(response)
                    }
                    Err(_) => {
                        // If we can't read the response body, create an empty body response
                        let mut response = Response::from_parts(parts, Body::empty());
                        response.headers_mut().insert("x-cache", "MISS".parse().unwrap());
                        Ok(response)
                    }
                }
            }
            else {
                // For non-successful responses, don't cache and add header
                let (mut parts, body) = response.into_parts();
                parts.headers.insert("x-cache", "MISS".parse().unwrap());
                Ok(Response::from_parts(parts, body))
            }
        })
    }
}

/// Generate a cache key from the GraphQL request body
fn generate_cache_key(body: &[u8], skip_pattern: &Option<Regex>) -> Option<String> {
    // Parse the GraphQL request
    let request: Value = serde_json::from_slice(body).ok()?;

    // Extract query, variables, and operation name for cache key
    let query = request.get("query")?.as_str()?;
    let variables = request.get("variables").unwrap_or(&Value::Null);
    let operation_name = request.get("operationName").unwrap_or(&Value::Null);

    // Check if we should skip caching based on regex pattern
    if should_skip_caching(query, operation_name, skip_pattern) {
        return None;
    }

    // Create a normalized string for hashing
    let cache_input = serde_json::json!({
        "query": query.trim(),
        "variables": variables,
        "operationName": operation_name
    });

    let cache_string = serde_json::to_string(&cache_input).ok()?;

    // Generate SHA256 hash as cache key
    let mut hasher = Sha256::new();
    hasher.update(cache_string.as_bytes());
    let hash = hasher.finalize();

    Some(format!("graphql:{:x}", hash))
}

/// Check if a GraphQL response contains errors
fn check_has_errors(json_response: &Value) -> bool {
    json_response
        .get("errors")
        .and_then(|errors| errors.as_array())
        .map(|errors| !errors.is_empty())
        .unwrap_or(false)
}

/// Check if we should skip caching for this GraphQL request
fn should_skip_caching(query: &str, operation_name: &Value, skip_pattern: &Option<Regex>) -> bool {
    if let Some(pattern) = skip_pattern {
        // Check operation name first if it exists
        if let Some(op_name) = operation_name.as_str() {
            if pattern.is_match(op_name) {
                return true;
            }
        }

        // Check for pattern matching in the query
        if pattern.is_match(query) {
            return true;
        }
    }

    false
}
