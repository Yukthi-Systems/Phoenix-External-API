use actix_web::{HttpMessage, HttpRequest, HttpResponse, get, post, web};
use crate::cache::handler::{delete_redis_cache, set_redis_cache};
use crate::models::errors::{ApiResponse, AppError};
use crate::models::api_key::ApiSession;
use crate::database::get_api_key_info;
use crate::state::AppState;


const MAX_SESSION_DURATION: u64 = 7 * 60 * 60;  // 7 hours in seconds


#[get("/who-am-i")]
async fn who_am_i(request: HttpRequest) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    Ok(HttpResponse::Ok().json(session_user))
}


#[post("/refresh")]
async fn refresh_key(request: HttpRequest, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    let api_key = session_user.api_key.to_string();
    // Fetch the latest session information from the DB and then update the cache accordingly
    let db_session_user = get_api_key_info(&state.pg_pool, &session_user.api_key).await?;
    if db_session_user.is_none() {
        // Delete the session from the cache if it exists
        delete_redis_cache(state.redis_cache.clone(), &api_key).await?;

        return Err(AppError::Unauthorized("API key is no longer valid".into()));
    }
    let db_session_user = db_session_user.unwrap();

    // Update the cache with the latest session information
    set_redis_cache(state.redis_cache.clone(), &api_key, &db_session_user, MAX_SESSION_DURATION).await?;

    Ok(HttpResponse::Ok().json(db_session_user))
}
