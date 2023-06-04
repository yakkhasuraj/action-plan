use std::sync::Arc;

use axum::{
    async_trait,
    body::HttpBody,
    extract::{FromRequest, State},
    headers::{authorization::Bearer, Authorization},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    BoxError, Json, RequestExt, TypedHeader,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use validator::Validate;

use crate::{
    database::users::{Column, Entity as Users},
    handlers::tasks::User,
    routes::AppState,
    utils::{error::AppError, token::verify_jwt},
};

#[async_trait]
impl<S, B> FromRequest<S, B> for User
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(request: Request<B>, _state: &S) -> Result<Self, Self::Rejection> {
        let Json(user) = request
            .extract::<Json<User>, _>()
            .await
            .map_err(|error| (StatusCode::BAD_REQUEST, format!("{}", error)))?;

        if let Err(errors) = user.validate() {
            return Err((StatusCode::BAD_REQUEST, format!("{}", errors)));
        }

        Ok(user)
    }
}

pub async fn auth_middleware<T>(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    mut request: Request<T>,
    next: Next<T>,
) -> Result<Response, AppError> {
    // let token = request
    //     .headers()
    //     .typed_get::<Authorization<Bearer>>()
    //     .ok_or_else(|| AppError::new(StatusCode::BAD_REQUEST, "Token not found"))?
    //     .token()
    //     .to_owned();

    let token = authorization.token().to_owned();

    verify_jwt(&token)?;

    // let database = request
    //     .extensions()
    //     .get::<DatabaseConnection>()
    //     .ok_or_else(|| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"))?;

    let user = Users::find()
        .filter(Column::Token.eq(Some(token)))
        .one(&state.database)
        .await
        .map_err(|_error| {
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        })?;

    let Some(user) = user else {return Err (AppError::new(StatusCode::UNAUTHORIZED, "User not found"))};

    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}
