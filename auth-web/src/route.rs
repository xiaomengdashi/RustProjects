// use actix_web::web;
// use actix_web_httpauth::middleware::HttpAuthentication;


// use crate::auth::validator;
// use crate::handler::{login, register, index};

// pub fn login_routes(cfg: &mut web::ServiceConfig) {
//     cfg.service(
//         web::scope("/api")
//            .service(web::resource("/register").route(web::post().to(register)))
//            .service(web::resource("/login").route(web::post().to(login)))
//          .service(web::resource("/protected").wrap(HttpAuthentication::bearer(validator)).route(web::get().to(index)))
//          .service(web::resource("/").route(web::get().to(index)))
//     );
// }

// pub fn base_routes(cfg: &mut web::ServiceConfig) {
//     cfg.service(
//         web::scope("/api")
//        .service(web::resource("/protected").wrap(HttpAuthentication::bearer(validator)).route(web::get().to(index)))
//     );
// }