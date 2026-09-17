use crate::database::identities::{list_domain_identities, get_org_identity};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, get, web};
use crate::database::domains::get_available_domains;
use crate::models::errors::{ApiResponse, AppError};
use crate::models::api_key::ApiSession;
use crate::models::QueryParams;
use crate::state::AppState;



#[get("/list/{domain_name}")]
async fn list_identities(request: HttpRequest, path: web::Path<String>, query: web::Query<QueryParams>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    // Check if the key has enough permissions to list department information and validate query parameters
    session_user.has_permissions(&["identity:view"])?;
    query.validate()?;

    // Check if the session user has access to the specified domain
    let domain_name = path.into_inner();
    let available_domains = get_available_domains(&state.pg_pool, &session_user.organization_id).await?;
    if !available_domains.contains(&domain_name) {
        return Err(AppError::Forbidden("Access to the specified domain is not allowed".into()));
    }

    let identities = list_domain_identities(&state.pg_pool, &domain_name, query.limit, query.offset).await?;

    Ok(HttpResponse::Ok().json(identities))
}


#[get("/info/{email_id}")]
async fn get_identity(request: HttpRequest, path: web::Path<String>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    // Check if the key has enough permissions to view identity information
    session_user.has_permissions(&["identity:view"])?;

    let email_id = path.into_inner();
    let identity = get_org_identity(&state.pg_pool, &session_user.organization_id, &email_id).await?;

    Ok(HttpResponse::Ok().json(identity))
}
