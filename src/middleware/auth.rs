use actix_web::{
    dev::{
        ServiceRequest,
        ServiceResponse
    },
    body::MessageBody,
    middleware::Next,
    HttpResponse,
    HttpMessage,
    Error,
    web,
};
use crate::cache::handler::{get_redis_cache, set_redis_cache};
use crate::models::api_key::ApiSession;
use crate::database::get_api_key_info;
use crate::state::AppState;


const MAX_SESSION_DURATION: u64 = 7 * 60 * 60;  // 7 hours in seconds


/// Check for valid API key based on x-api-key header
/// Inserts SessionUser into request extensions if valid
/// Returns true if valid, false otherwise
async fn session_check(req: &ServiceRequest) -> bool {
    // Look for X-API-Key header
    let api_key: Option<uuid::Uuid> = req
        .headers()
        .get("x-api-key")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| uuid::Uuid::parse_str(s).ok());

    // If no session access token, return false
    if api_key.is_none() {
        return false;
    }

    // Check cache for session
    let state = req.app_data::<web::Data<AppState>>().unwrap();
    let api_key = api_key.unwrap();

    // See if the access token exists in Redis cache and get the associated SessionUser
    let session_user: Option<ApiSession> = get_redis_cache(state.redis_cache.clone(), &api_key.to_string()).await.unwrap();

    // If session user is not found in cache, check the database
    if session_user.is_none() {
        // See if the API key exists in the database
        let db_session_user = get_api_key_info(&state.pg_pool, &api_key).await.unwrap();
        if db_session_user.is_none() {
            return false;
        }
        let db_session_user = db_session_user.unwrap();

        // Insert the database session user into the Redis cache for future requests
        set_redis_cache(state.redis_cache.clone(), &api_key.to_string(), &db_session_user, MAX_SESSION_DURATION).await.unwrap();

        // Insert the database session user into request extensions for further use
        req.extensions_mut().insert(db_session_user);
    } else {
        // If session user is found in cache, use it
        let session_user = session_user.unwrap();
        // Insert user into request extensions for further use
        req.extensions_mut().insert(session_user);
    }

    // If we reach here, the session is valid
    true
}


/// Authentication middleware
/// Short-circuits with 401 Unauthorized if checks fail
/// Otherwise calls the next service in the chain
pub async fn auth_check<B>(req: ServiceRequest, next: Next<B>) -> Result<ServiceResponse, Error>
    where B: MessageBody + 'static
{
    // See if session is valid
    if !session_check(&req).await {
        // Short-circuit and return 401 Unauthorized
        let resp = HttpResponse::Unauthorized()
            .append_header(("content-type", "text/plain; charset=utf-8"))
            .body("Unauthorized: Invalid API Key");

        // Convert into a ServiceResponse with a boxed body to satisfy types
        return Ok(req.into_response(resp).map_into_boxed_body());
    }

    // authorized -> call the next service
    let res = next.call(req).await?;
    Ok(res.map_into_boxed_body())
}
