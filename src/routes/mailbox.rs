use crate::database::mailboxes::{list_domain_mailboxes, get_org_mailbox, update_mailbox_info, create_new_mailbox};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, get, patch, post, web};
use crate::models::mailbox::{MailBoxEditRequest, MailBoxCreateRequest};
use crate::database::domains::get_available_domains;
use crate::models::errors::{ApiResponse, AppError};
use crate::handlers::assign_new_mailbox_server;
use crate::database::orgs::get_org_info;
use crate::models::api_key::ApiSession;
use crate::models::QueryParams;
use crate::state::AppState;



#[get("/list/{domain_name}")]
async fn list_mailboxes(request: HttpRequest, path: web::Path<String>, query: web::Query<QueryParams>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    // Check if the key has enough permissions to list mailboxes information and validate query parameters
    session_user.has_permissions(&["mailbox:view"])?;
    query.validate()?;

    // Check if the session user has access to the specified domain
    let domain_name = path.into_inner();
    let available_domains = get_available_domains(&state.pg_pool, &session_user.organization_id).await?;
    if !available_domains.contains(&domain_name) {
        return Err(AppError::Forbidden("Access to the specified domain is not allowed".into()));
    }

    let mailboxes = list_domain_mailboxes(&state.pg_pool, &domain_name, query.limit, query.offset).await?;

    Ok(HttpResponse::Ok().json(mailboxes))
}


#[get("/info/{email_id}")]
async fn get_mailbox(request: HttpRequest, path: web::Path<String>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    // Check if the key has enough permissions to view identity information
    session_user.has_permissions(&["mailbox:view"])?;

    let mailbox = get_org_mailbox(&state.pg_pool, &session_user.organization_id, &path).await?;

    Ok(HttpResponse::Ok().json(mailbox))
}


#[patch("/update")]
async fn update_mailbox(request: HttpRequest, edit_request: web::Json<MailBoxEditRequest>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    // Check if the key has enough permissions to edit mailbox information
    session_user.has_permissions(&["mailbox:edit"])?;

    // Extract and validate the edit request data
    let edit_data = edit_request.into_inner();
    edit_data.validate()?;

    // Check if the session user has access to the specified domain
    let available_domains = get_available_domains(&state.pg_pool, &session_user.organization_id).await?;
    if !available_domains.contains(&edit_data.domain_name) {
        return Err(AppError::Forbidden("Access to the specified domain is not allowed".into()));
    }

    let result = update_mailbox_info(
        &state.pg_pool,
        &edit_data.email,
        edit_data.is_enabled,
        edit_data.forwarding_policy_id,
        edit_data.distribution_policy_id,
        edit_data.general_policy_id,
    ).await?;
    if result == 0 {
        return Err(AppError::NotFound("Mailbox not found".into()));
    }

    Ok(HttpResponse::Ok().json(result))
}


#[post("/create")]
async fn create_mailbox(request: HttpRequest, create_request: web::Json<MailBoxCreateRequest>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    // Check if the key has enough permissions to create mailbox information
    session_user.has_permissions(&["mailbox:create"])?;

    // Extract and validate the create request data
    let create_data = create_request.into_inner();
    create_data.validate()?;

    // Check if the session user has access to the specified domain
    let available_domains = get_available_domains(&state.pg_pool, &session_user.organization_id).await?;
    if !available_domains.contains(&create_data.domain_name) {
        return Err(AppError::Forbidden("Access to the specified domain is not allowed".into()));
    }

    // Check Organization-level quota restrictions
    let org_info = get_org_info(&state.pg_pool, &session_user.organization_id).await?;

    // Validate if it can create a new mailbox with given quota
    org_info.can_create_new_mailbox(create_data.quota_allocated)?;

    let result = create_new_mailbox(
        &state.pg_pool,
        &create_data.email,
        &create_data.domain_name,
        &session_user.organization_id,
        create_data.forwarding_policy_id,
        create_data.distribution_policy_id,
        create_data.general_policy_id,
        create_data.quota_allocated,
    ).await?;
    if result == 0 {
        return Err(AppError::Conflict("Failed to create mailbox".into()));
    }

    // If everything goes well, make a RMQ call to create the mailbox in actual server
    tokio::spawn(assign_new_mailbox_server(create_data.email.clone()));

    Ok(HttpResponse::Ok().json(result))
}
