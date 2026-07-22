#[must_use]
pub fn sql_state(error: &sqlx::Error) -> Option<String> {
    match error {
        sqlx::Error::Database(db) => db.code().map(std::borrow::Cow::into_owned),
        _ => None,
    }
}

#[must_use]
pub fn is_unique_violation(error: &sqlx::Error) -> bool {
    sql_state(error).as_deref() == Some("23505")
}

#[must_use]
pub fn is_foreign_key_violation(error: &sqlx::Error) -> bool {
    sql_state(error).as_deref() == Some("23503")
}
