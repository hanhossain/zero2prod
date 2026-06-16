use crate::helpers::{assert_is_redirect_to, spawn_app};
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn you_must_be_logged_in_to_see_the_change_password_form(pool: PgPool) {
    let app = spawn_app(pool).await;
    let response = app.get_change_password().await;
    assert_is_redirect_to(&response, "/login");
}

#[sqlx::test]
async fn you_must_be_logged_in_to_change_your_password(pool: PgPool) {
    let app = spawn_app(pool).await;
    let new_password = Uuid::new_v4().to_string();
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": Uuid::new_v4().to_string(),
            "new_password": &new_password,
            "new_password_check": &new_password
        }))
        .await;
    assert_is_redirect_to(&response, "/login");
}
