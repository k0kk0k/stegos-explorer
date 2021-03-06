/// This module serve api

use std::io;
use std::sync::Arc;
use explorer_backend::api_schema;

use juniper;

use actix_web::{middleware, web, http::header, App, Error, HttpResponse, HttpServer};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use actix_cors::Cors;

use crate::api_schema::{create_schema, Schema};

async fn graphiql() -> HttpResponse {
    let uri = format!(
        "{}/graphql",
        std::env::var("SERVICE_ADDR").expect("SERVICE_ADDR to be set")
    );
    let html = graphiql_source(&uri);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn graphql(
    st: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let user = web::block(move || {
        let res = data.execute(&st, &());
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(user))
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    // Create Juniper schema
    let schema = std::sync::Arc::new(create_schema());

    // Start http server
    HttpServer::new(move || {
        App::new()        .wrap(
            Cors::new() // <- Construct CORS middleware builder
              .allowed_methods(vec!["GET", "POST"])
              .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
              .allowed_header(header::CONTENT_TYPE)
                .supports_credentials()
              .max_age(3600)
              .finish())
            .data(schema.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
    .bind("0.0.0.0:3000")?
        .run()
        .await
}
