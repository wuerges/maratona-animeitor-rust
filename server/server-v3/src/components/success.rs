use std::collections::BTreeMap;

use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use utoipa::{
    IntoResponses, ToSchema,
    openapi::{self, ContentBuilder, ResponseBuilder, ResponsesBuilder},
};

#[derive(Debug, IntoResponses)]
#[response(status = 200, description = "Ok")]
pub struct Success;

impl actix_web::Responder for Success {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        HttpResponse::Ok().finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data<T> {
    data: T,
}

impl<T> Data<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl<T> actix_web::Responder for Data<T>
where
    Data<T>: Serialize,
{
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        HttpResponse::Ok().json(self)
    }
}

impl<T> IntoResponses for Data<T>
where
    T: ToSchema,
{
    fn responses() -> BTreeMap<String, openapi::RefOr<openapi::response::Response>> {
        ResponsesBuilder::new()
            .response(
                "200",
                ResponseBuilder::new().description("Ok").content(
                    "application/json",
                    ContentBuilder::new().schema(Some(T::schema())).build(),
                ),
            )
            .build()
            .into()
    }
}
