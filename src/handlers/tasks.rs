use std::sync::Arc;

use crate::database::tasks::{self, Entity as Tasks};
use crate::database::users::{self, Entity as Users};
use crate::routes::AppState;
use axum::{
    extract::{Path, Query, State},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    Json, TypedHeader,
};
use sea_orm::{
    prelude::DateTimeWithTimeZone, ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition,
    EntityTrait, IntoActiveModel, QueryFilter,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

pub async fn hello_world() -> &'static str {
    "Hello, World!"
}

#[derive(Deserialize, Validate, Debug)]
pub struct User {
    #[validate(email(message = "Email must be valid"))]
    username: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    password: String,
}

pub async fn custom_json_extractor(user: User) {
    dbg!(user);
}

#[derive(Deserialize)]
pub struct CreateTask {
    title: String,
    description: Option<String>,
    priority: Option<String>,
}

pub async fn create_task(
    // Extension(database): Extension<DatabaseConnection>,
    State(state): State<Arc<AppState>>,
    authorization: TypedHeader<Authorization<Bearer>>,
    Json(body): Json<CreateTask>,
) -> Result<(), StatusCode> {
    let token = authorization.token();

    let user = if let Some(user) = Users::find()
        .filter(users::Column::Token.eq(Some(token)))
        .one(&state.database)
        .await
        .map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        user
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let new_task = tasks::ActiveModel {
        priority: Set(body.priority),
        title: Set(body.title),
        description: Set(body.description),
        user_id: Set(Some(user.id)),
        ..Default::default()
    };

    let _result = new_task.insert(&state.database).await.unwrap();

    Ok(())
}

#[derive(Serialize)]
pub struct Task {
    id: i32,
    title: String,
    priority: Option<String>,
    description: Option<String>,
    deleted_at: Option<DateTimeWithTimeZone>,
    user_id: Option<i32>,
}

pub async fn get_one_task(
    Path(id): Path<i32>,
    // Extension(database): Extension<DatabaseConnection>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Task>, StatusCode> {
    let task = Tasks::find_by_id(id)
        .filter(tasks::Column::DeletedAt.is_null())
        .one(&state.database)
        .await
        .unwrap();

    if let Some(task) = task {
        Ok(Json(Task {
            id: task.id,
            title: task.title,
            priority: task.priority,
            description: task.description,
            deleted_at: task.deleted_at,
            user_id: task.user_id,
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[derive(Deserialize)]
pub struct GetTaskQueryParams {
    priority: Option<String>,
}

pub async fn get_all_tasks(
    // Extension(database): Extension<DatabaseConnection>,
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetTaskQueryParams>,
) -> Result<Json<Vec<Task>>, StatusCode> {
    let mut priority_filter = Condition::all();
    if let Some(priority) = params.priority {
        dbg!(&priority);
        priority_filter = priority_filter.add(tasks::Column::Priority.eq(priority))
    }

    let tasks = Tasks::find()
        // .filter(tasks::Column::Priority.eq(params.priority))
        .filter(priority_filter)
        .filter(tasks::Column::DeletedAt.is_null())
        .all(&state.database)
        .await
        .map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|db_task| Task {
            id: db_task.id,
            title: db_task.title,
            priority: db_task.priority,
            description: db_task.description,
            deleted_at: db_task.deleted_at,
            user_id: db_task.user_id,
        })
        .collect();

    Ok(Json(tasks))
}

#[derive(Deserialize)]
pub struct UpdateTask {
    pub id: Option<i32>,
    pub priority: Option<String>,
    pub title: String,
    pub completed_at: Option<DateTimeWithTimeZone>,
    pub description: Option<String>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub user_id: Option<i32>,
    pub is_default: Option<bool>,
}

pub async fn update_task(
    Path(id): Path<i32>,
    // Extension(database): Extension<DatabaseConnection>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateTask>,
) -> Result<(), StatusCode> {
    let update_task = tasks::ActiveModel {
        id: Set(id),
        priority: Set(body.priority),
        title: Set(body.title),
        completed_at: Set(body.completed_at),
        description: Set(body.description),
        deleted_at: Set(body.deleted_at),
        user_id: Set(body.user_id),
        is_default: Set(body.is_default),
    };

    Tasks::update(update_task)
        .filter(tasks::Column::Id.eq(id))
        .exec(&state.database)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

#[derive(Deserialize)]
pub struct PartialUpdateTask {
    pub id: Option<i32>,
    #[serde(
        default,    // <- important for deserialization
        skip_serializing_if = "Option::is_none",    // <- important for serialization
        with = "::serde_with::rust::double_option",
    )]
    pub priority: Option<Option<String>>,
    pub title: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub completed_at: Option<Option<DateTimeWithTimeZone>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub description: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub deleted_at: Option<Option<DateTimeWithTimeZone>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub user_id: Option<Option<i32>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub is_default: Option<Option<bool>>,
}

pub async fn partial_update_task(
    Path(id): Path<i32>,
    // Extension(database): Extension<DatabaseConnection>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PartialUpdateTask>,
) -> Result<(), StatusCode> {
    let mut patch_task = if let Some(task) = Tasks::find_by_id(id)
        .one(&state.database)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        task.into_active_model()
    } else {
        return Err(StatusCode::NOT_FOUND);
    };

    if let Some(priority) = body.priority {
        patch_task.priority = Set(priority);
    }

    if let Some(title) = body.title {
        patch_task.title = Set(title);
    }

    if let Some(completed_at) = body.completed_at {
        patch_task.completed_at = Set(completed_at);
    }

    if let Some(description) = body.description {
        patch_task.description = Set(description);
    }

    if let Some(deleted_at) = body.deleted_at {
        patch_task.deleted_at = Set(deleted_at);
    }

    // let update_task = tasks::ActiveModel {
    //     id: Set(id),
    //     priority: Set(body.priority),
    //     title: Set(body.title),
    //     completed_at: Set(body.completed_at),
    //     description: Set(body.description),
    //     deleted_at: Set(body.deleted_at),
    //     user_id: Set(body.user_id),
    //     is_default: Set(body.is_default),
    // };

    Tasks::update(patch_task)
        .filter(tasks::Column::Id.eq(id))
        .exec(&state.database)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

#[derive(Deserialize)]
pub struct DeleteTaskQueryParams {
    soft: bool,
}
pub async fn delete_task(
    Path(id): Path<i32>,
    // Extension(database): Extension<DatabaseConnection>,
    State(state): State<Arc<AppState>>,
    Query(params): Query<DeleteTaskQueryParams>,
) -> Result<(), StatusCode> {
    // let task = if let Some(task) = Tasks::find_by_id(id)
    //     .one(&database)
    //     .await
    //     .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    // {
    //     task.into_active_model()
    // } else {
    //     return Err(StatusCode::NOT_FOUND);
    // };

    // Tasks::delete(task)
    //     .exec(&database)
    //     .await
    //     .map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)?;

    if params.soft {
        let mut task = if let Some(task) = Tasks::find_by_id(id)
            .one(&state.database)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            task.into_active_model()
        } else {
            return Err(StatusCode::NOT_FOUND);
        };

        let now = chrono::Utc::now();

        task.deleted_at = Set(Some(now.into()));
        Tasks::update(task)
            .exec(&state.database)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        Tasks::delete_by_id(id)
            .exec(&state.database)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(())
}
