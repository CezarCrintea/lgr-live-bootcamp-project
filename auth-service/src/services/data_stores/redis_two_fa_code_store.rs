use std::sync::Arc;

use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    Email,
};

pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(&email);

        let two_fa_tuple = TwoFATuple(
            login_attempt_id.as_ref().to_string(),
            code.as_ref().to_string(),
        );

        let serialized = serde_json::to_string(&two_fa_tuple)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
        let mut conn = self.conn.write().await;
        let ttl = TEN_MINUTES_IN_SECONDS
            .try_into()
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        let _: () = conn
            .set_ex(&key, serialized, ttl)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(email);

        let mut conn = self.conn.write().await;
        let _: () = conn
            .del(&key)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let key = get_key(email);

        let mut conn = self.conn.write().await;
        let value = match conn.get::<_, String>(&key) {
            Ok(value) => value,
            Err(_) => return Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        };

        let data: TwoFATuple = match serde_json::from_str(&value) {
            Ok(two_fa_tuple) => two_fa_tuple,
            Err(_) => return Err(TwoFACodeStoreError::UnexpectedError),
        };

        let login_attempt_id =
            LoginAttemptId::parse(data.0).map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        let two_fa_code =
            TwoFACode::parse(data.1).map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok((login_attempt_id, two_fa_code))
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
