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
            .service(
                web::scope("/reviews")
                    .service(
                        web::resource("/public_key").route(web::get().to(app_review_controller::get_public_key)),
                    )
                    .service(
                        web::resource("/sign").route(web::post().to(app_review_controller::sign))
                    )
                    .service(
                        web::resource("").route(web::get().to(app_review_controller::get))
                    )
                    .service(
                        web::resource("/delete").route(web::delete().to(app_review_controller::delete))
                    ),
            )
    );

    #[cfg(feature = "swagger")]
    {
        use utoipa::OpenApi;
        use utoipa_swagger_ui::SwaggerUi;

        cfg.service(
            SwaggerUi::new("/swagger-ui/{_:.*}")
                .url("/api-docs/openapi.json", doc::ApiDoc::openapi())
        );
    }

    debug!("Routes configured!")
}
