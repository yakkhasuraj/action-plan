use sea_orm::Database;
use std::net::SocketAddr;

use crate::routes::create_routes;

pub async fn run(database_uri: &str) {
    let database = Database::connect(database_uri).await.unwrap();
    let app = create_routes(database).await;

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
