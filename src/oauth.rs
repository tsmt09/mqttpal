use std::collections::HashMap;

use actix_session::Session;
use actix_web::{
    web::{self, Path},
    HttpRequest, HttpResponse, Responder,
};
use openidconnect::{
    core::{
        CoreClient, CoreProviderMetadata, CoreResponseType, CoreTokenResponse, CoreUserInfoClaims,
    },
    reqwest::async_http_client,
    AccessToken, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    IssuerUrl, Nonce, OAuth2TokenResponse, RedirectUrl, Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};

use crate::models::user::User;

pub type OauthConfigs = web::Data<HashMap<String, OauthConfig>>;

#[derive(Deserialize, Debug)]
pub struct OauthConfig {
    pub ui_name: String,
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    pub scope: String,
    #[serde(default)]
    pub username_field: UsernameField,
}

#[derive(Default, Deserialize, Debug)]
pub enum UsernameField {
    PreferredUsername,
    #[default]
    Email,
}

pub fn oauth_login_scoped(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("oauth/{config_name}")
            .service(web::resource("/").route(web::get().to(oauth_login)))
            .service(web::resource("/callback").route(web::get().to(google_auth_callback_handler))),
    );
}

pub async fn oauth_login(
    req: HttpRequest,
    config_name: Path<String>,
    configs: OauthConfigs,
) -> impl Responder {
    let config_name = config_name.into_inner();
    let Some(config) = configs.get(&config_name) else {
        return HttpResponse::NotFound().body("cannot find desired oauth configuration");
    };
    let redirect_uri = format!(
        // TODO: use correct protocol here
        "http://{}/oauth/{}/callback",
        req.connection_info().host(),
        &config_name
    );
    log::info!(
        "Logging in user on oauth provider {} with redirect uri: {}",
        &config_name,
        &redirect_uri
    );
    let auth_url = get_auth_url(config, redirect_uri).await;
    HttpResponse::Found()
        .append_header(("Location", auth_url.as_str()))
        .finish()
}

pub async fn google_auth_callback_handler(
    db: web::Data<crate::DbPool>,
    req: HttpRequest,
    config_name: Path<String>,
    configs: OauthConfigs,
    params: web::Query<AuthCallbackParams>,
    session: Session,
) -> impl Responder {
    // Extract authorization code from the callback parameters
    let config_name = config_name.into_inner();
    let Some(config) = configs.get(&config_name) else {
        return HttpResponse::NotFound().body("cannot find desired oauth configuration");
    };
    let redirect_uri = format!(
        // TODO: use correct protocol here
        "http://{}/oauth/{}/callback",
        req.connection_info().host(),
        &config_name
    );
    log::info!(
        "Authenticating user on oauth provider {} with redirect uri '{}' and code '{}'",
        &config_name,
        &redirect_uri,
        params.code.clone()
    );
    log::debug!("{:?}", params);
    // Exchange the authorization code for an access token
    let token_response =
        exchange_code_for_token(config, redirect_uri.clone(), params.code.clone()).await;

    match token_response {
        Ok(token_response) => {
            // Extract user info from the access token
            log::info!("{:?}", token_response);
            let token = token_response.access_token().to_owned();
            let Ok(id_token_json) = serde_json::to_string(&token_response.id_token()) else {
                return HttpResponse::InternalServerError().body("unable to serialize id_token");
            };
            let user_info = fetch_user_info(config, redirect_uri, token).await.unwrap();
            log::info!("{:?}", user_info);
            let user_name = match config.username_field {
                UsernameField::PreferredUsername => {
                    user_info.preferred_username().map(|x| x.to_string())
                }
                UsernameField::Email => user_info.email().map(|x| x.to_string()),
            };
            // fallback to sub
            let user_name = user_name.unwrap_or(user_info.subject().to_string());

            // Check user in database
            let user = match User::insert_if_not_exist(
                &db,
                &user_name,
                crate::models::user::UserSource::OAuth("Google".into()),
            )
            .await
            {
                Ok(u) => u,
                Err(e) => {
                    log::error!("cannot login user: {}", e.to_string());
                    return HttpResponse::InternalServerError().body(format!("{e}"));
                }
            };
            let _ = session.insert("loggedin", "true");
            let _ = session.insert("username", user.name);
            let _ = session.insert("jwt", id_token_json);
            let _ = session.insert("oauth_config", &config_name);
            HttpResponse::Found()
                .append_header(("Location", "/"))
                .finish()
        }
        Err(e) => HttpResponse::InternalServerError().body(format!(
            "Failed to exchange authorization code for access token {}",
            e
        )),
    }
}

async fn get_auth_url(config: &OauthConfig, redirect_uri: String) -> String {
    // define OIDC Parameters
    let issuer = IssuerUrl::new(config.issuer.clone()).unwrap();
    let client_id = ClientId::new(config.client_id.clone());
    let client_secret = ClientSecret::new(config.client_secret.clone());
    let scope = Scope::new(config.scope.clone());

    // discover metadata from issuer
    let metadata = CoreProviderMetadata::discover_async(issuer, async_http_client)
        .await
        .unwrap();

    // create client from metadata
    let client = CoreClient::from_provider_metadata(metadata, client_id, Some(client_secret))
        .set_redirect_uri(RedirectUrl::new(redirect_uri).unwrap());

    // get authorize_url
    let (auth_url, _, _) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(scope)
        .url();
    auth_url.to_string()
}

async fn exchange_code_for_token(
    config: &OauthConfig,
    redirect_uri: String,
    code: String,
) -> anyhow::Result<CoreTokenResponse> {
    // define OIDC Parameters
    let issuer = IssuerUrl::new(config.issuer.clone()).unwrap();
    let client_id = ClientId::new(config.client_id.clone());
    let client_secret = ClientSecret::new(config.client_secret.clone());

    let metadata = CoreProviderMetadata::discover_async(issuer, async_http_client)
        .await
        .unwrap();

    let client = CoreClient::from_provider_metadata(metadata, client_id, Some(client_secret))
        .set_redirect_uri(RedirectUrl::new(redirect_uri).unwrap());

    // exchange provided code for auth token
    let token = match client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
    {
        Ok(token) => token,
        Err(e) => {
            log::error!("{:?}", e);
            return Err(e.into());
        }
    };

    println!("{:#?}", serde_json::to_string(&token)?);

    Ok(token)
}

async fn fetch_user_info(
    config: &OauthConfig,
    redirect_uri: String,
    access_token: AccessToken,
) -> Result<CoreUserInfoClaims, Box<dyn std::error::Error>> {
    // Define OIDC parameters
    let issuer = IssuerUrl::new(config.issuer.clone()).unwrap();
    let client_id = ClientId::new(config.client_id.clone());
    let client_secret = ClientSecret::new(config.client_secret.clone());
    let metadata = CoreProviderMetadata::discover_async(issuer, async_http_client)
        .await
        .unwrap();

    // Create a Google OpenID Connect client
    let client = CoreClient::from_provider_metadata(metadata, client_id, Some(client_secret))
        .set_redirect_uri(RedirectUrl::new(redirect_uri).unwrap());

    // Fetch user info using the access token
    let user_info = client
        .user_info(access_token, None)
        .unwrap()
        .request_async(async_http_client)
        .await?;

    Ok(user_info)
}

#[derive(serde::Deserialize, Debug)]
pub struct AuthCallbackParams {
    // state: String,
    code: String,
    // scope: String,
    // authuser: u32,
    // prompt: String,
}
