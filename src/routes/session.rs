use actix_web::{HttpMessage, HttpRequest, HttpResponse, get, post, web};
use crate::models::errors::{ApiResponse, AppError};
use crate::models::api_key::ApiSession;
use crate::state::AppState;



#[get("/who-am-i")]
async fn who_am_i(request: HttpRequest, state: web::Data<AppState>) -> ApiResponse {
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

    // Fetch the latest session information from the DB and then update the cache accordingly

    Ok(HttpResponse::Ok().json(session_user))
}
