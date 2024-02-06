use std::fs;

use actix::{Actor, Context, Handler};
use oxide_auth::{
    endpoint::{Endpoint, OwnerConsent, OwnerSolicitor, Solicitation, WebResponse},
    frontends::simple::endpoint::{ErrorInto, FnSolicitor, Generic, Vacant},
    primitives::prelude::{AuthMap, ClientMap, RandomGenerator, Scope},
};

use crate::api::models::oauth2::OAuth2AuthorizationResult;
use crate::api::oauth2::config::OAuthConfig;
use crate::api::oauth2::oxide_auth_actix::{
    OAuthMessage, OAuthOperation, OAuthRequest, OAuthResponse, WebError
};
use crate::api::oauth2::token_issuer::JwtTokenIssuer;
use crate::auth::JwtTokenScope;
use crate::config::Config;
use crate::db::models::user::User;

pub struct OAuth2State {
    endpoint: Generic<
        ClientMap,
        AuthMap<RandomGenerator>,
        JwtTokenIssuer,
        Vacant,
        Vec<Scope>,
        fn() -> OAuthResponse,
    >,
}

#[derive(Debug, Clone)]
pub enum Extras {
    AuthGet,
    AuthPost(User, OAuth2AuthorizationResult),
    ClientCredentials,
    Nothing,
}

impl OAuth2State {
    pub fn preconfigured(config: Config) -> Self {
        let jwt_issuer = JwtTokenIssuer::new(config.clone());

        let oauth_config = match fs::read_to_string(&config.oauth_config_path) {
            Ok(oauth_config_toml_string) => toml::from_str::<OAuthConfig>(&oauth_config_toml_string)
                .map_err(|e| {
                    log::error!("Failed to read oauth configuration: {:?}", e)
                })
                .ok(),
            Err(e) => {
                log::warn!("Couldn't load oauth config from path {}: {:?}", &config.oauth_config_path, e);
                None
            }
        };

        OAuth2State {
            endpoint: Generic {
                // A registrar with one pre-registered client
                registrar: oauth_config.map_or(ClientMap::default(), |config|
                    config.clients.iter().map(|c| c.into()).collect()
                ),
                // Authorization tokens are 16 byte random keys to a memory hash map.
                authorizer: AuthMap::new(RandomGenerator::new(16)),
                // Bearer tokens are also random generated but 256-bit tokens, since they live longer
                // and this example is somewhat paranoid.
                //
                // We could also use a `TokenSigner::ephemeral` here to create signed tokens which can
                // be read and parsed by anyone, but not maliciously created. However, they can not be
                // revoked and thus don't offer even longer lived refresh tokens.
                issuer: jwt_issuer,

                solicitor: Vacant,

                // scopes: vec![],
                scopes: vec![JwtTokenScope::Profile.into_oauth_scope().unwrap()],

                response: OAuthResponse::ok,
            },
        }
    }

    pub fn with_solicitor<'a, S>(
        &'a mut self, solicitor: S,
    ) -> impl Endpoint<OAuthRequest, Error = WebError> + 'a
    where
        S: OwnerSolicitor<OAuthRequest> + 'static,
    {
        ErrorInto::new(Generic {
            authorizer: &mut self.endpoint.authorizer,
            registrar: &mut self.endpoint.registrar,
            issuer: &mut self.endpoint.issuer,
            solicitor,
            scopes: &mut self.endpoint.scopes,
            response: OAuthResponse::ok,
        })
    }
}

impl Actor for OAuth2State {
    type Context = Context<Self>;
}

impl<Operation> Handler<OAuthMessage<Operation, Extras>> for OAuth2State
where
    Operation: OAuthOperation,
{
    type Result = Result<Operation::Item, Operation::Error>;

    fn handle(&mut self, msg: OAuthMessage<Operation, Extras>, _: &mut Self::Context) -> Self::Result {
        let (op, ex) = msg.into_inner();
        match ex {
            Extras::AuthGet => {
                let solicitor = FnSolicitor(move |_: &mut OAuthRequest, solicitation: Solicitation| {
                    let grant = solicitation.pre_grant();
                    let state = solicitation.state();

                    let scope = grant.scope.to_string();
                    let mut extra = vec![
                        ("response_type", "code"),
                        ("client_id", grant.client_id.as_str()),
                        ("redirect_uri", grant.redirect_uri.as_str()),
                        ("scope", &scope),
                    ];

                    if let Some(state) = state {
                        extra.push(("state", state));
                    }

                    let public_url = std::env::var("PUBLIC_URL").unwrap();
                    let redirect_url = url::Url::parse_with_params(
                        &format!("{}/auth/authorize", &public_url), &extra
                    )
                    .unwrap();

                    let mut response = OAuthResponse::ok();
                    response.redirect(redirect_url)
                        .map_err(|_| WebError::InternalError(Some("Failed to redirect".to_string())))
                        .unwrap();

                    // This will display a page to the user asking for his permission to proceed. The submitted form
                    // will then trigger the other authorization handler which actually completes the flow.
                    OwnerConsent::InProgress(response)
                });

                op.run(self.with_solicitor(solicitor))
            },
            Extras::AuthPost(user, result) => {
                let solicitor = FnSolicitor(move |_: &mut OAuthRequest, _: Solicitation| {
                    match result {
                        OAuth2AuthorizationResult::Allow => OwnerConsent::Authorized(user.id.to_string()),
                        _ => OwnerConsent::Denied
                    }
                });

                op.run(self.with_solicitor(solicitor))
            },
            // Extras::ClientCredentials => {
            //     let solicitor = FnSolicitor(move |request: &mut OAuthRequest, solicitation: Solicitation| {
            //         // For the client credentials flow, the solicitor is consulted
            //         // to ensure that the resulting access token is issued to the
            //         // correct owner. This may be the client itself, if clients
            //         // and resource owners are from the same set of entities, but
            //         // may be distinct if that is not the case.
            //         OwnerConsent::Authorized(solicitation.pre_grant().client_id.clone())
            //     });
            //
            //     op.run(self.with_solicitor(solicitor))
            // },
            Extras::Nothing => {
                let result = op.run(&mut self.endpoint);
                result
            },
            _ => op.run(&mut self.endpoint),
        }
    }
}