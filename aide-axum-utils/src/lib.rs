use aide::{
    axum::{routing::ApiMethodRouter, ApiRouter},
    generate::GenContext,
    openapi::{
        HeaderStyle::Simple, Operation, Parameter, ParameterData, ParameterSchemaOrContent::Schema,
        Response, SchemaObject,
    },
    transform::TransformOperation,
    OperationOutput,
};
use axum::{http::StatusCode, response::IntoResponse};

/// Wraps an API router which adds a tag to all routes
pub struct TagApiRouter<S> {
    inner: ApiRouter<S>,
    tag: &'static str,
}

impl<S: Clone + Send + Sync + 'static> TagApiRouter<S> {
    pub fn new(tag: &'static str) -> Self {
        Self {
            inner: ApiRouter::new(),
            tag,
        }
    }

    pub fn api_route(mut self, path: &str, method_router: ApiMethodRouter<S>) -> Self {
        self.inner = self
            .inner
            .api_route_with(path, method_router, |item| item.tag(self.tag));
        self
    }
}

impl<S> From<TagApiRouter<S>> for ApiRouter<S> {
    fn from(value: TagApiRouter<S>) -> Self {
        value.inner
    }
}

/// Returns a closure which adds a summary and description to a route.
pub fn route_info(
    summary: &'static str,
    description: &'static str,
) -> impl FnOnce(TransformOperation<'_>) -> TransformOperation<'_> {
    move |op| op.summary(summary).description(description)
}

/// Returns 204 No Content in Axum routes
pub struct NoContent;

impl IntoResponse for NoContent {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::NO_CONTENT, ()).into_response()
    }
}

impl OperationOutput for NoContent {
    type Inner = Self;

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<u16>, Response)> {
        vec![(
            Some(StatusCode::NO_CONTENT.as_u16()),
            <() as OperationOutput>::operation_response(ctx, operation).unwrap(),
        )]
    }
}

pub fn simple_header(name: String, description: String, ctx: &mut GenContext) -> Parameter {
    let s = ctx.schema.subschema_for::<String>();
    Parameter::Header {
        parameter_data: ParameterData {
            name,
            description: Some(description),
            required: true,
            deprecated: Default::default(),
            format: Schema(SchemaObject {
                json_schema: s,
                example: Default::default(),
                external_docs: Default::default(),
            }),
            example: Default::default(),
            examples: Default::default(),
            explode: Default::default(),
            extensions: Default::default(),
        },
        style: Simple,
    }
}

#[macro_export]
macro_rules! with_aide_docs {
    ($method:ident, $handler:ident) => {
        paste::paste! {
            aide::axum::routing::[< $method _with>]($handler, [<aide_docs_ $handler>]())
        }
    };
}
