use crate::middleware::htmx::HtmxHeaders;
use actix_session::Session;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, FromRequest, HttpMessage, HttpResponse,
};
use askama::Template;
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

#[derive(Template)]
#[template(path = "fullpage_render.html")]
pub struct FullPageTemplate {
    pub user: Option<String>,
    pub body: String,
}

pub struct FullPageRender;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S> Transform<S, ServiceRequest> for FullPageRender
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = FullPageRenderMiddleWare<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(FullPageRenderMiddleWare { service }))
    }
}

pub struct FullPageRenderMiddleWare<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for FullPageRenderMiddleWare<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            let (req, resp) = res.into_parts();
            let htmx = req.clone().extensions().get::<HtmxHeaders>().cloned();
            let username = if let Ok(session) = Session::extract(&req).into_inner() {
                session.get::<String>("username").unwrap_or(None)
            } else {
                None
            };
            if let Some(htmx) = htmx {
                if !htmx.request() {
                    let status = resp.status();
                    let resp_body = if let Ok(body) =
                        std::str::from_utf8(&actix_web::body::to_bytes(resp.into_body()).await?)
                    {
                        String::from(body)
                    } else {
                        String::from("")
                    };
                    let template = FullPageTemplate {
                        user: username,
                        body: resp_body,
                    };
                    let new_resp = HttpResponse::build(status).body(template.render().unwrap());
                    Ok(ServiceResponse::new(req, new_resp))
                } else {
                    Ok(ServiceResponse::new(req, resp))
                }
            } else {
                Ok(ServiceResponse::new(req, resp))
            }
        })
    }
}
