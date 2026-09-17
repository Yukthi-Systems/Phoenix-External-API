use crate::database::identities::{list_domain_identities, get_org_identity, update_identity_by_email};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, get, patch, web};
use crate::database::domains::get_available_domains;
use crate::models::errors::{ApiResponse, AppError};
use crate::models::identity::IdentityEditRequest;
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


#[patch("/update")]
async fn update_identity(request: HttpRequest, body: web::Json<IdentityEditRequest>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    // Check if the key has enough permissions to update identity information
    session_user.has_permissions(&["identity:edit"])?;

    let edit_request = body.into_inner();
    edit_request.validate()?;

    // Check if the session user has access to the domain of the identity being updated
    let available_domains = get_available_domains(&state.pg_pool, &session_user.organization_id).await?;
    if !available_domains.contains(&edit_request.domain_name) {
        return Err(AppError::Forbidden("Access to the specified domain is not allowed".into()));
    }

    let result = update_identity_by_email(
        &state.pg_pool,
        &edit_request.email,
        &edit_request.domain_name,
        &edit_request.first_name,
        &edit_request.last_name,
        &edit_request.primary_phone,
        &edit_request.secondary_email,
        &edit_request.is_app_2fa_enabled,
        &edit_request.is_sms_2fa_enabled,
        &edit_request.is_email_2fa_enabled,
        &edit_request.restriction_policy_id,
        &edit_request.department_id,
        &edit_request.is_enabled,
    ).await?;
    if result == 0 {
        return Err(AppError::NotFound("Identity not found".into()));
    }

    Ok(HttpResponse::Ok().json(result))
}
