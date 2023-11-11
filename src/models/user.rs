use bb8_redis::redis::cmd;
use serde::{Deserialize, Serialize};

pub enum Role {
    Admin = 0,
    User = 1,
}

impl From<i32> for Role {
    fn from(i: i32) -> Self {
        match i {
            0 => Role::Admin,
            1 => Role::User,
            _ => panic!("Unknown role: {}", i),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub email: Option<String>,
    pub password: String,
    pub role_id: i32,
}

impl User {
    pub async fn check(pool: &crate::DbPool, check_name: &str, check_password: &str) -> bool {
        let user = User::get_by_name(&pool, check_name).await;
        if let Some(user) = user {
            user.password == check_password
        } else {
            false
        }
    }

    pub async fn get_by_name(pool: &crate::DbPool, name: &str) -> Option<User> {
        let mut conn = pool.get().await.expect("no connection available");
        let user: Option<String> = cmd("HGET").arg("users").arg(name)
            .query_async(&mut *conn)
            .await
            .expect("Cannot query users from redis");
        if let Some(user) = user {
            let user: User = serde_json::from_str(&user).expect("Cannot deserialize user");
            Some(user)
        } else {
            None
        }
    }

    pub async fn list(pool: &crate::DbPool) -> Vec<User> {
        let mut conn = pool.get().await.expect("no connection available");
        let mut users_hash: Vec<String> = cmd("HGETALL").arg("users")
            .query_async(&mut *conn)
            .await
            .expect("Cannot query users from redis");
        let mut users: Vec<User> = Vec::new();
        while let Some(user) = users_hash.pop() {
            let user: User = serde_json::from_str(&user).expect("Cannot deserialize user");
            users.push(user);
            let _ = users_hash.pop();
        }
        users
    }

    pub async fn insert(&self, pool: &crate::DbPool) {
        let mut conn = pool.get().await.expect("no connection available");
        let user_json = serde_json::to_string(&self).expect("Cannot serialize user");
        let _: i32 = cmd("HSET").arg("users").arg(&self.name).arg(user_json)
            .query_async(&mut *conn)
            .await
            .expect("Cannot insert user");
    }

    pub async fn delete(pool: &crate::DbPool, name: &str) -> bool {
        let mut conn = pool.get().await.expect("no connection available");
        let res: Result<i32, redis::RedisError> = cmd("HDEL").arg("users").arg(name)
            .query_async(&mut *conn)
            .await;
        match res {
            Ok(i) => i > 0,
            Err(_) => false,
        }
    }
}
