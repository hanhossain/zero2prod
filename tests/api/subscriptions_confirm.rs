use crate::helpers::spawn_app;
use sqlx::PgPool;

#[sqlx::test]
async fn confirmations_without_token_are_rejected_with_a_400(pool: PgPool) {
    let app = spawn_app(pool).await;
    let response = reqwest::get(format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400);
}
