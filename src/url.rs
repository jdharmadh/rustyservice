use base62;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Deserialize, Serialize, Debug)]
pub struct TinyUrlHttpRequest {
    pub url: String,
    pub preference: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TinyUrlHttpResponse {
    pub tiny_url: String,
}

impl TinyUrlHttpResponse {
    pub fn from(tiny_url: String) -> Self {
        TinyUrlHttpResponse { tiny_url: tiny_url }
    }
}

impl From<UrlPostResult> for (warp::http::StatusCode, String) {
    fn from(result: UrlPostResult) -> Self {
        match result {
            UrlPostResult::Success(tiny_url) => (warp::http::StatusCode::OK, tiny_url),
            UrlPostResult::Taken => (warp::http::StatusCode::CONFLICT, String::from("")),
            UrlPostResult::DbError => (
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                String::from(""),
            ),
        }
    }
}

pub struct TinyUrlService {
    db: sled::Db,
}

pub enum UrlPostResult {
    Success(String),
    Taken,
    DbError,
}

impl TinyUrlService {
    pub fn from(file_path: &str) -> Self {
        let db = sled::open(file_path).unwrap();
        Self { db }
    }

    pub fn post(&self, url: String, preference: Option<String>) -> UrlPostResult {
        let key = match preference {
            Some(value) => value,
            None => self.generate_unique_key(url.clone()),
        };

        let result = match self.db.get(key.as_bytes()) {
            Ok(value) => value,
            Err(_) => return UrlPostResult::DbError,
        };

        match result {
            Some(value) => {
                if value == url.as_bytes() {
                    return UrlPostResult::Success(key);
                } else {
                    return UrlPostResult::Taken;
                }
            }
            None => {
                let res = self.db.insert(key.as_bytes(), url.as_bytes());
                match res {
                    Ok(_) => return UrlPostResult::Success(key),
                    Err(_) => return UrlPostResult::DbError,
                }
            }
        }
    }

    pub fn get(&self, key: String) -> Result<String, String> {
        match self
            .db
            .get(key.as_bytes())
            .map_err(|_| "Database error".to_string())?
        {
            Some(value) => Ok(String::from_utf8(value.to_vec()).unwrap()),
            None => Err("Key not found".to_string()),
        }
    }

    fn generate_unique_key(&self, string: String) -> String {
        let mut hasher = Sha256::new();
        hasher.update(string.as_bytes());
        let hash_result = hasher.finalize();
        let num = u64::from_be_bytes(hash_result[0..8].try_into().unwrap());
        let mut result = base62::encode(num);
        let mut rng = rand::thread_rng();
        while let Some(value) = self.db.get(result.clone()).unwrap() {
            if value == string {
                return result;
            } else {
                result.push(rng.gen_range('a'..='z'));
            }
        }
        return result;
    }
}
