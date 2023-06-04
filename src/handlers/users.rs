use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Extension, Json};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use serde::{Deserialize, Serialize};

use crate::{
    database::users::{self, Column, Entity as Users, Model},
    routes::AppState,
    utils::{
        hash::{hash_password, verify_password},
        token::create_jwt,
    },
};

#[derive(Deserialize)]
pub struct CreateUser {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct User {
    id: i32,
    username: String,
    token: String,
}

pub async fn create_user(
    // Extension(database): Extension<DatabaseConnection>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateUser>,
) -> Result<Json<User>, StatusCode> {
    let jwt = create_jwt()?;
    let new_user = users::ActiveModel {
        username: Set(body.username),
        password: Set(hash_password(body.password)?),
        token: Set(Some(jwt)),
        ..Default::default()
    };

    let result = new_user
        .insert(&state.database)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap();

    Ok(Json(User {
        id: result.id,
        username: result.username,
        token: result.token.unwrap(),
    }))
}

pub async fn login(
    // Extension(database): Extension<DatabaseConnection>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateUser>,
) -> Result<Json<User>, StatusCode> {
    let user = Users::find()
        .filter(Column::Username.eq(body.username))
        .one(&state.database)
        .await
        .map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(db_user) = user {
        if !verify_password(body.password, &db_user.password)? {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let new_token = create_jwt()?;
        let mut update_user = db_user.into_active_model();

        update_user.token = Set(Some(new_token));

        let result = update_user
            .save(&state.database)
            .await
            .map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(User {
            id: result.id.unwrap(),
            username: result.username.unwrap(),
            token: result.token.unwrap().unwrap(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn logout(
    // Extension(database): Extension<DatabaseConnection>,
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Model>,
) -> Result<(), StatusCode> {
    let mut user = user.into_active_model();

    user.token = Set(None);

    user.save(&state.database)
        .await
        .map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}
