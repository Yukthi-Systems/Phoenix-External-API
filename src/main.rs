use routes::{health, session, organization, department, domains, identity, mailbox};
use actix_web::web::scope as actix_scope;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer};
use std::env::var as env_var;
use actix_cors::Cors;

mod middleware;
mod database;
mod handlers;
mod models;
mod routes;
mod cache;
mod state;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = state::initialize().await;

    // Start the Actix web server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Cors::default()
                .allowed_methods(vec!["GET", "POST", "PATCH", "PUT", "DELETE"])
                .allow_any_origin()
                .allow_any_header()
                .max_age(420)
            )
            .service(
                actix_scope("/health")
                .service(health::api_health_check)
            )
            .service(
                actix_scope("/self")
                .wrap(from_fn(middleware::auth::auth_check))
                .service(session::refresh_key)
                .service(session::who_am_i)
            )
            .service(
                actix_scope("/organization")
                .wrap(from_fn(middleware::auth::auth_check))
                .service(organization::get_organization_info)
            )
            .service(
                actix_scope("/domain")
                .wrap(from_fn(middleware::auth::auth_check))
                .service(domains::list_domains)
                .service(domains::edit_domain)
                .service(domains::get_domain)
            )
            .service(
                actix_scope("/identity")
                .wrap(from_fn(middleware::auth::auth_check))
                .service(identity::list_identities)
                .service(identity::create_identity)
                .service(identity::update_identity)
                .service(identity::delete_identity)
                .service(identity::password_reset)
                .service(identity::get_identity)
            )
            .service(
                actix_scope("/department")
                .wrap(from_fn(middleware::auth::auth_check))
                .service(department::get_department_info)
                .service(department::create_department)
                .service(department::update_department)
                .service(department::delete_department)
                .service(department::list_departments)
            )
            .service(
                actix_scope("/mailbox")
                .wrap(from_fn(middleware::auth::auth_check))
                .service(mailbox::list_mailboxes)
                .service(mailbox::get_mailbox)
                // .service(mailbox::create_mailbox)
                // .service(mailbox::update_mailbox)
                // .service(mailbox::delete_mailbox)
                // .service(mailbox::quota_update)
            )
    })
    .bind(("0.0.0.0", 8686))?
    .workers(env_var("API_WORKERS_COUNT").unwrap_or("4".to_string()).parse().unwrap())
    .run().await
}
