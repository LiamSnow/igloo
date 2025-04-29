use bcrypt::BcryptResult;
use tokio_rusqlite::Connection;
use uuid::Uuid;

use super::Auth;

const TOKEN_EXPIRATION_DAYS: i32 = 30;

pub struct TokenDatabase(Connection);

impl TokenDatabase {
    pub async fn connect() -> Result<Self, tokio_rusqlite::Error> {
        let conn = Connection::open("./tokens.sqlite").await?;
        conn.call(|conn| {
            Ok(conn.execute(
                "CREATE TABLE IF NOT EXISTS tokens (
                token    TEXT PRIMARY KEY,
                username TEXT,
                created  DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
                (),
            )?)
        })
        .await?;
        Ok(Self(conn))
    }

    /// creates and returns new token for username
    pub async fn add(&self, username: String) -> Result<String, tokio_rusqlite::Error> {
        let token = Uuid::now_v7().to_string();
        let token_copy = token.clone();
        self.0
            .call(move |conn| {
                Ok(conn.execute(
                    "INSERT INTO tokens (token, username) VALUES (?1, ?2)",
                    (token_copy, username),
                )?)
            })
            .await?;
        Ok(token)
    }

    pub async fn remove(&self, token: String) -> Result<(), tokio_rusqlite::Error> {
        self.0.call(move |conn| {
            Ok(conn.execute("DELETE FROM tokens WHERE token = ?1", [token])?)
        }).await?;
        Ok(())
    }

    /// returns Some(username) if valid
    pub async fn validate(&self, token: String) -> Result<Option<String>, tokio_rusqlite::Error> {
        let token_copy = token.clone();
        let res = self.0.call(move |conn| {
            Ok(conn.query_row(
                "SELECT username, julianday(CURRENT_TIMESTAMP) - julianday(created) as days_diff FROM tokens WHERE token = ?1",
                [token_copy],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            )?)
        }).await;

        if let Err(e) = res {
            return match e {
                tokio_rusqlite::Error::Rusqlite(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                _ => Err(e)
            }
        }

        let (username, days_diff) = res.unwrap();
        if days_diff > TOKEN_EXPIRATION_DAYS as f64 {
            self.0.call(move |conn| {
                Ok(conn.execute("DELETE FROM tokens WHERE token = ?1", [token])?)
            }).await?;
            Ok(None)
        } else {
            Ok(Some(username))
        }
    }
}

impl Auth {
    /// returns Some(uid) if valid
    pub fn validate_login(&self, username: &str, password: &str) -> BcryptResult<Option<usize>> {
        let uid = match self.uid_lut.get(username) {
            Some(uid) => *uid,
            None => return Ok(None),
        };

        let correct_hash = &self.pw_hashes[uid];
        Ok(match bcrypt::verify(password, &correct_hash)? {
            true => Some(uid),
            false => None,
        })
    }
}
