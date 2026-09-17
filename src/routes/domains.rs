use crate::database::domains::{get_domain_details, list_org_domains, update_domain_by_name};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, get, patch, web};
use crate::models::errors::{ApiResponse, AppError};
use crate::models::domain::DomainEditRequest;
use crate::models::api_key::ApiSession;
use crate::models::QueryParams;
use crate::state::AppState;



#[get("/list")]
async fn list_domains(request: HttpRequest, query: web::Query<QueryParams>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    // Check if the key has enough permissions to list department information and validate query parameters
    session_user.has_permissions(&["domain:view"])?;
    query.validate()?;

    let domains = list_org_domains(&state.pg_pool, &session_user.organization_id, query.limit, query.offset).await?;

    Ok(HttpResponse::Ok().json(domains))
}


#[get("/info/{domain_name}")]
async fn get_domain(request: HttpRequest, path: web::Path<String>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    // Check if the key has enough permissions to view department information
    session_user.has_permissions(&["domain:view"])?;

    let domain_info = get_domain_details(&state.pg_pool, &session_user.organization_id, &path).await?;

    Ok(HttpResponse::Ok().json(domain_info))
}


#[patch("/update/{domain_name}")]
async fn edit_domain(request: HttpRequest, path: web::Path<String>, body: web::Json<DomainEditRequest>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    // Check if the key has enough permissions to update department information
    session_user.has_permissions(&["domain:edit"])?;

    // We only allow editing few fields of the domain, remaining fields should be managed from Admin panel only

    let result = update_domain_by_name(
        &state.pg_pool,
        &session_user.organization_id,
        &path,
        &body.details,
        body.is_active,
        body.filter_policy_id,
        body.attachment_policy_id,
        body.disclaimer_id,
        body.caution_id
    ).await?;
    if result == 0 {
        return Err(AppError::NotFound("Domain not found".into()));
    }

    Ok(HttpResponse::Ok().json(result))
}
