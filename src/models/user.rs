use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password: String,
}
