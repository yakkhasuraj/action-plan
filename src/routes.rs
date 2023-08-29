use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use sea_orm::DatabaseConnection;

use crate::{
    handlers::{
        tasks::{
            create_task, custom_json_extractor, delete_task, get_all_tasks, get_one_task,
            hello_world, partial_update_task, update_task,
        },
        users::{create_user, login, logout},
    },
    middlewares::auth_middleware,
};

pub struct AppState {
    pub database: DatabaseConnection,
}

pub async fn create_routes(database: DatabaseConnection) -> Router {
    let app_state = Arc::new(AppState { database });

    Router::new()
        // Task Routes
        .route("/tasks", post(create_task))
        .route("/tasks/:id", put(update_task))
        .route("/tasks/:id/partials", patch(partial_update_task))
        .route("/tasks/:id", delete(delete_task))
        // User Routes
        .route("/users/logout", post(logout))
        // .route_layer(middleware::from_fn(auth_middleware))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
        // Test routes
        .route("/", get(hello_world))
        .route("/custom-json-extractor", post(custom_json_extractor))
        // Task Routes
        .route("/tasks", get(get_all_tasks))
        .route("/tasks/:id", get(get_one_task))
        // User Routes
        .route("/users", post(create_user))
        .route("/users/login", post(login))
        // .layer(Extension(database))
        .with_state(app_state)
}
