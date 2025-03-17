use aide::{
    axum::{routing::ApiMethodRouter, ApiRouter},
    generate::GenContext,
    openapi::{
        CookieStyle::Form, HeaderStyle::Simple, MediaType, Operation, Parameter, ParameterData,
        ParameterSchemaOrContent::Schema, Response as OpenApiResponse, SchemaObject,
    },
    transform::TransformOperation,
    OperationOutput,
};
use axum::{
    body::Bytes,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response as AxumResponse},
};
use paste::paste;

/// Wraps an API router to add tags to all its routes
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

    pub fn nest(mut self, path: &str, router: ApiRouter<S>) -> Self {
        self.inner = self.inner.nest(path, router);
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

/// Responds with a `204 No Content` status code when returned from Axum routes.
pub struct NoContent;

impl IntoResponse for NoContent {
    fn into_response(self) -> AxumResponse {
        (StatusCode::NO_CONTENT, ()).into_response()
    }
}

impl OperationOutput for NoContent {
    type Inner = Self;

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<u16>, OpenApiResponse)> {
        vec![(
            Some(StatusCode::NO_CONTENT.as_u16()),
            <() as OperationOutput>::operation_response(ctx, operation).unwrap(),
        )]
    }
}

/// Returns a header parameter to be used with [aide::operation::add_parameters].
pub fn simple_header(
    name: String,
    description: String,
    required: bool,
    ctx: &mut GenContext,
) -> Parameter {
    Parameter::Header {
        parameter_data: simple_parameter_data(name, description, required, ctx),
        style: Simple,
    }
}

/// Returns a cookie parameter to be used with [aide::operation::add_parameters].
pub fn simple_cookie(
    name: String,
    description: String,
    required: bool,
    ctx: &mut GenContext,
) -> Parameter {
    Parameter::Cookie {
        parameter_data: simple_parameter_data(name, description, required, ctx),
        style: Form,
    }
}

/// Returns parameter data to be used with [aide::openapi::parameter::Parameter]
pub fn simple_parameter_data(
    name: String,
    description: String,
    required: bool,
    ctx: &mut GenContext,
) -> ParameterData {
    let s = ctx.schema.subschema_for::<String>();
    ParameterData {
        name,
        required,
        description: Some(description),
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
    }
}

/// When a route uses the proc macro `#[aide_docs]`, calling this macro with the router's name
/// expands to an ApiMethodRouter to be used in [aide::axum::ApiRouter::api_route].
#[macro_export]
macro_rules! with_aide_docs {
    ($method:ident, $handler:ident) => {
        $crate::paste! {
            aide::axum::routing::[< $method _with>]($handler, [<__aide_docs_ $handler>]())
        }
    };
}

/// To be returned in Axum routes, for routes that returns headers.
pub struct WithHeaderMap<T: IntoResponse + OperationOutput>(pub HeaderMap, pub T);

impl<T: IntoResponse + OperationOutput> IntoResponse for WithHeaderMap<T> {
    fn into_response(self) -> AxumResponse {
        (self.0, self.1).into_response()
    }
}

impl<T: IntoResponse + OperationOutput> OperationOutput for WithHeaderMap<T> {
    type Inner = T;

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<u16>, OpenApiResponse)> {
        T::inferred_responses(ctx, operation)
    }
}

/// Responds with the specified status code when returned from Axum routes.
pub struct WithStatusCode<T: IntoResponse + OperationOutput>(pub StatusCode, pub T);

impl<T: IntoResponse + OperationOutput> IntoResponse for WithStatusCode<T> {
    fn into_response(self) -> AxumResponse {
        (self.0, self.1).into_response()
    }
}

impl<T: IntoResponse + OperationOutput> OperationOutput for WithStatusCode<T> {
    type Inner = T;

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<u16>, OpenApiResponse)> {
        T::inferred_responses(ctx, operation)
    }
}

/// Specifies "text/plain" content type in the documentation when returned from Axum routes.
pub struct TextPlain<T: IntoResponse + OperationOutput = Bytes>(pub T);

impl<T: IntoResponse + OperationOutput> IntoResponse for TextPlain<T> {
    fn into_response(self) -> AxumResponse {
        self.0.into_response()
    }
}

impl<T: IntoResponse + OperationOutput> OperationOutput for TextPlain<T> {
    type Inner = T;

    fn operation_response(
        ctx: &mut GenContext,
        _operation: &mut Operation,
    ) -> Option<OpenApiResponse> {
        let s = ctx.schema.subschema_for::<String>();
        Some(OpenApiResponse {
            description: "plain text".into(),
            content: From::from([(
                "text/plain".into(),
                MediaType {
                    schema: Some(SchemaObject {
                        json_schema: s,
                        example: Default::default(),
                        external_docs: Default::default(),
                    }),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        })
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<u16>, OpenApiResponse)> {
        Vec::from([(Some(200), Self::operation_response(ctx, operation).unwrap())])
    }
}
