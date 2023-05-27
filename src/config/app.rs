use actix_web::web;
use log::debug;

use crate::api::*;

pub fn config_services(cfg: &mut web::ServiceConfig) {
    debug!("Configuring routes...");
    cfg.service(
        web::scope("/api")
            .service(ping_controller::ping)
            .service(
                web::scope("/auth")
                    .service(
                        web::resource("/signup").route(web::post().to(auth_controller::signup)),
                    )
                    .service(
                        web::resource("/login").route(web::post().to(auth_controller::login)),
                    )
                    .service(
                        web::resource("/refresh").route(web::post().to(auth_controller::refresh)),
                    )
                    .service(
                        web::resource("/logout").route(web::post().to(auth_controller::logout)),
                    )
                    .service(
                        web::resource("/me").route(web::get().to(auth_controller::me)),
                    ),
            )
            // .service(
            //     web::scope("/address-book")
            //         .service(
            //             web::resource("")
            //                 .route(web::get().to(address_book_controller::find_all))
            //                 .route(web::post().to(address_book_controller::insert)),
            //         )
            //         .service(
            //             web::resource("/{id}")
            //                 .route(web::get().to(address_book_controller::find_by_id))
            //                 .route(web::put().to(address_book_controller::update))
            //                 .route(web::delete().to(address_book_controller::delete)),
            //         )
            //         .service(
            //             web::resource("/filter")
            //                 .route(web::get().to(address_book_controller::filter)),
            //         ),
            // ),
    );
    debug!("Routes configured!")
}
