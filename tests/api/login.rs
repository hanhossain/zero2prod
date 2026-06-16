use crate::helpers::{assert_is_redirect_to, spawn_app};
use sqlx::PgPool;

#[sqlx::test]
async fn an_error_flash_message_is_set_on_failure(pool: PgPool) {
    // arrange
    let app = spawn_app(pool).await;

    // act - part 1 - try to login
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/login");

    // act - part 2 - follow the redirect
    let html_page = app.get_login_html().await;
    assert!(html_page.contains("<p><i>Authentication failed</i></p>"));

    // act - part 3 - reload the login page
    let html_page = app.get_login_html().await;
    assert!(!html_page.contains("Authentication failed"));
}

#[sqlx::test]
async fn redirect_to_admin_dashboard_after_login_success(pool: PgPool) {
    // arrange
    let app = spawn_app(pool).await;

    // act - part 1 - login
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // act - part 2 - follow the redirect
    let html_page = app.get_admin_dashboard().await;
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));
}
