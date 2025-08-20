use sled::Error;
use warp::filters::fs::file;

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
        let key = preference.unwrap_or_else(|| url.clone() + "key");
        let result = match self.db.get(key.as_bytes()) {
            Ok(value) => value,
            Err(_) => return UrlPostResult::DbError,
        };

        match result {
            Some(_) => return UrlPostResult::Taken,
            None => {
                let res = self.db.insert(key.as_bytes(), url.as_bytes());
                println!("Inserted key: {}, value: {}", key, url);
                match res {
                    Ok(_) => return UrlPostResult::Success(url),
                    Err(_) => return UrlPostResult::DbError,
                }
            }
        }
    }
}
