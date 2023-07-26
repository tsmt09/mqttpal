use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{Header, HeaderMap, HeaderName, HeaderValue},
    Error, FromRequest, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

#[allow(unused_variables)]
#[derive(Default, Debug)]
pub struct HtmxHeaders {
    // request headers
    boosted: Option<bool>,
    current_url: Option<String>,
    history_restore_request: Option<bool>,
    prompt: Option<String>,
    request: Option<bool>,
    target: Option<String>,
    trigger_name: Option<String>,
    trigger: Option<String>,
    // response headers
    location: Option<String>,
    push_url: Option<String>,
    redirect: Option<String>,
    refresh: Option<String>,
    replace_url: Option<String>,
    reswap: Option<String>,
    retarget: Option<String>,
    reselect: Option<String>,
    response_trigger: Option<String>,
    trigger_after_settle: Option<String>,
    trigger_after_swap: Option<String>,
}

impl HtmxHeaders {
    fn from_headers_map(map: &HeaderMap) -> Self {
        HtmxHeaders {
            boosted: map
                .get("hx-boosted")
                .map(|v| v.to_str().unwrap().parse::<bool>().unwrap()),
            current_url: map
                .get("hx-current-url")
                .map(|v| v.to_str().unwrap().to_string()),
            history_restore_request: map
                .get("hx-history-restore-request")
                .map(|v| v.to_str().unwrap().parse::<bool>().unwrap()),
            prompt: map
                .get("hx-prompt")
                .map(|v| v.to_str().unwrap().to_string()),
            request: map
                .get("hx-request")
                .map(|v| v.to_str().unwrap().parse::<bool>().unwrap()),
            target: map
                .get("hx-target")
                .map(|v| v.to_str().unwrap().to_string()),
            trigger_name: map
                .get("hx-trigger-name")
                .map(|v| v.to_str().unwrap().to_string()),
            trigger: map
                .get("hx-trigger")
                .map(|v| v.to_str().unwrap().to_string()),
            location: map
                .get("hx-location")
                .map(|v| v.to_str().unwrap().to_string()),
            push_url: map
                .get("hx-push-url")
                .map(|v| v.to_str().unwrap().to_string()),
            redirect: map
                .get("hx-redirect")
                .map(|v| v.to_str().unwrap().to_string()),
            refresh: map
                .get("hx-refresh")
                .map(|v| v.to_str().unwrap().to_string()),
            replace_url: map
                .get("hx-replace-url")
                .map(|v| v.to_str().unwrap().to_string()),
            reswap: map
                .get("hx-reswap")
                .map(|v| v.to_str().unwrap().to_string()),
            retarget: map
                .get("hx-retarget")
                .map(|v| v.to_str().unwrap().to_string()),
            reselect: map
                .get("hx-reselect")
                .map(|v| v.to_str().unwrap().to_string()),
            response_trigger: map
                .get("hx-trigger")
                .map(|v| v.to_str().unwrap().to_string()),
            trigger_after_settle: map
                .get("hx-trigger-after-settle")
                .map(|v| v.to_str().unwrap().to_string()),
            trigger_after_swap: map
                .get("hx-trigger-after-swap")
                .map(|v| v.to_str().unwrap().to_string()),
        }
    }
    fn write_headers(&self, map: &mut HeaderMap) {
        if let Some(location) = &self.location {
            map.insert(
                HeaderName::from_static("hx-location"),
                HeaderValue::from_str(&location).unwrap(),
            );
        }
        if let Some(push_url) = &self.push_url {
            map.insert(
                HeaderName::from_static("hx-push-url"),
                HeaderValue::from_str(&push_url).unwrap(),
            );
        }
        if let Some(redirect) = &self.redirect {
            map.insert(
                HeaderName::from_static("hx-redirect"),
                HeaderValue::from_str(&redirect).unwrap(),
            );
        }
        if let Some(refresh) = &self.refresh {
            map.insert(
                HeaderName::from_static("hx-refresh"),
                HeaderValue::from_str(&refresh).unwrap(),
            );
        }
        if let Some(replace_url) = &self.replace_url {
            map.insert(
                HeaderName::from_static("hx-replace-url"),
                HeaderValue::from_str(&replace_url).unwrap(),
            );
        }
        if let Some(reswap) = &self.reswap {
            map.insert(
                HeaderName::from_static("hx-reswap"),
                HeaderValue::from_str(&reswap).unwrap(),
            );
        }
        if let Some(reselect) = &self.reselect {
            map.insert(
                HeaderName::from_static("hx-reselect"),
                HeaderValue::from_str(&reselect).unwrap(),
            );
        }
        if let Some(retarget) = &self.retarget {
            map.insert(
                HeaderName::from_static("hx-retarget"),
                HeaderValue::from_str(&retarget).unwrap(),
            );
        }
        if let Some(response_trigger) = &self.response_trigger {
            map.insert(
                HeaderName::from_static("hx-trigger"),
                HeaderValue::from_str(&response_trigger).unwrap(),
            );
        }
        if let Some(trigger_after_settle) = &self.trigger_after_settle {
            map.insert(
                HeaderName::from_static("hx-trigger-after-settle"),
                HeaderValue::from_str(&trigger_after_settle).unwrap(),
            );
        }
        if let Some(trigger_after_swap) = &self.trigger_after_swap {
            map.insert(
                HeaderName::from_static("hx-trigger-after-swap"),
                HeaderValue::from_str(&trigger_after_swap).unwrap(),
            );
        }
    }
    pub fn boosted(&self) -> bool {
        self.boosted.unwrap_or(false)
    }
    pub fn current_url(&self) -> &Option<String> {
        &self.current_url
    }
    pub fn history_restore_request(&self) -> bool {
        self.history_restore_request.unwrap_or(false)
    }
    pub fn prompt(&self) -> &Option<String> {
        &self.prompt
    }
    pub fn request(&self) -> bool {
        self.request.unwrap_or(false)
    }
    pub fn target(&self) -> &Option<String> {
        &self.target
    }
    pub fn trigger_name(&self) -> &Option<String> {
        &self.trigger_name
    }
    pub fn trigger(&self) -> &Option<String> {
        &self.trigger
    }
    pub fn set_location(&mut self, location: &str) {
        self.location = Some(location.to_string());
    }
    pub fn set_push_url(&mut self, push_url: &str) {
        self.push_url = Some(push_url.to_string());
    }
    pub fn set_redirect(&mut self, redirect: &str) {
        self.redirect = Some(redirect.to_string());
    }
    pub fn set_refresh(&mut self, refresh: &str) {
        self.refresh = Some(refresh.to_string());
    }
    pub fn set_replace_url(&mut self, replace_url: &str) {
        self.replace_url = Some(replace_url.to_string());
    }
    pub fn set_reswap(&mut self, reswap: &str) {
        self.reswap = Some(reswap.to_string());
    }
    pub fn set_reselect(&mut self, reselect: &str) {
        self.reselect = Some(reselect.to_string());
    }
    pub fn set_retarget(&mut self, retarget: &str) {
        self.retarget = Some(retarget.to_string());
    }
    pub fn set_trigger(&mut self, trigger: &str) {
        self.response_trigger = Some(trigger.to_string());
    }
    pub fn set_trigger_after_settle(&mut self, trigger_after_settle: &str) {
        self.trigger_after_settle = Some(trigger_after_settle.to_string());
    }
    pub fn set_trigger_after_swap(&mut self, trigger_after_swap: &str) {
        self.trigger_after_swap = Some(trigger_after_swap.to_string());
    }
}

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct Htmx;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for Htmx
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = HtmxMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(HtmxMiddleware { service }))
    }
}

pub struct HtmxMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for HtmxMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let headers = HtmxHeaders::from_headers_map(req.headers());
        req.extensions_mut().insert(headers);
        let fut = self.service.call(req);
        Box::pin(async move {
            let mut res = fut.await?;
            let req = res.request().to_owned();
            let ext = req.extensions_mut();
            let hx_hd = ext.get::<HtmxHeaders>().unwrap();
            hx_hd.write_headers(res.headers_mut());
            Ok(res)
        })
    }
}
