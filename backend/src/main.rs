extern crate dotenv;

mod controllers;
mod helpers;
mod services;

use controllers::clips::ClipController;
use dotenv::dotenv;

use actix_cors::Cors;
use actix_web::{http, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://127.0.0.1:42069")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                http::header::ORIGIN,
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ]);

        let mut app = App::new().wrap(cors);

        app = app.configure(ClipController::register_routes);

        app
    })
    .workers(4)
    .bind(("127.0.0.1", 9011))?
    .run()
    .await
}
