use crate::database::departments::{get_department_details, list_org_departments};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, get, web};
use crate::models::api_key::ApiSession;
use crate::models::errors::ApiResponse;
use crate::models::QueryParams;
use crate::state::AppState;



#[get("/list")]
async fn list_departments(request: HttpRequest, query: web::Query<QueryParams>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    // Check if the key has enough permissions to list department information
    session_user.has_permissions(&["department:view"])?;

    let departments = list_org_departments(&state.pg_pool, &session_user.organization_id, query.limit, query.offset).await?;

    Ok(HttpResponse::Ok().json(departments))
}


#[get("/info/{department_id}")]
async fn get_department_info(request: HttpRequest, path: web::Path<uuid::Uuid>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<ApiSession>().unwrap();

    // Check if the key has enough permissions to view department information
    session_user.has_permissions(&["department:view"])?;

    let department_info = get_department_details(&state.pg_pool, &session_user.organization_id, &path).await?;

    Ok(HttpResponse::Ok().json(department_info))
}
