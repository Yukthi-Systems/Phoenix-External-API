use actix_web::{HttpMessage, HttpRequest, HttpResponse, get, web};
use crate::database::orgs::get_org_info;
use crate::models::api_key::ApiSession;
use crate::models::errors::ApiResponse;
use crate::state::AppState;



#[get("/info")]
async fn get_organization_info(request: HttpRequest, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    // Check if the key has enough permissions to view organization information
    session_user.has_permissions(&["organization:view"])?;

    let org_info = get_org_info(&state.pg_pool, &session_user.organization_id).await?;

    Ok(HttpResponse::Ok().json(org_info))
}
