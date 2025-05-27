use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use rocket::request::{FromRequest, Outcome, Request};
use std::env;
use std::ops::{Deref, DerefMut};

pub struct RdsConn(pub MultiplexedConnection);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RdsConn {
    type Error = ();
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let rconn = request.rocket().state::<MultiplexedConnection>().unwrap();
        Outcome::Success(RdsConn(rconn.clone()))
    }
}

impl Clone for RdsConn {
    fn clone(&self) -> Self {
        RdsConn(self.0.clone())
    }
}

impl Deref for RdsConn {
    type Target = MultiplexedConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RdsConn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub async fn set_config(
    connection: &mut MultiplexedConnection,
    key: &str,
    value: &str,
) -> Result<(), redis::RedisError> {
    let mut connection = connection.clone();
    let _: () = redis::cmd("CONFIG")
        .arg("SET")
        .arg(key)
        .arg(value)
        .query_async(&mut connection)
        .await?;
    Ok(())
}
/// Configure Redis for LRU eviction policies
pub async fn configure_redis_lru(
    mut connection: MultiplexedConnection,
) -> Result<(), redis::RedisError> {
    // Set maximum memory limit (adjust as needed for your environment)
    // Example: 256MB - you should adjust this based on your server capacity
    let maxmemory = env::var("REDIS_MAXMEMORY").unwrap_or_else(|_| "256mb".to_string());
    let _: () = set_config(&mut connection, "maxmemory", &maxmemory).await?;

    // Set LRU eviction policy
    // Available policies:
    // - allkeys-lru: evict any key using LRU algorithm
    // - volatile-lru: evict keys with expiration using LRU algorithm
    // - allkeys-lfu: evict any key using LFU algorithm (Redis 4.0+)
    // - volatile-lfu: evict keys with expiration using LFU algorithm (Redis 4.0+)
    let eviction_policy =
        env::var("REDIS_EVICTION_POLICY").unwrap_or_else(|_| "allkeys-lru".to_string());
    let _: () = set_config(&mut connection, "maxmemory-policy", &eviction_policy).await?;

    // Optional: Set LRU sample size (default is 5, higher values are more accurate but slower)
    let lru_samples = env::var("REDIS_LRU_SAMPLES").unwrap_or_else(|_| "5".to_string());
    let _: () = set_config(&mut connection, "maxmemory-samples", &lru_samples).await?;

    println!("Redis LRU configuration applied:");
    println!("  maxmemory: {}", maxmemory);
    println!("  maxmemory-policy: {}", eviction_policy);
    println!("  maxmemory-samples: {}", lru_samples);

    Ok(())
}

pub async fn init_rds_client() -> MultiplexedConnection {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = redis::Client::open(redis_url).expect("connect to redis fail");
    let connection = client.get_multiplexed_async_connection().await.unwrap();

    // Configure LRU policies if enabled
    if env::var("ENABLE_REDIS_LRU_CONFIG").unwrap_or_else(|_| "false".to_string()) == "true" {
        if let Err(e) = configure_redis_lru(connection.clone()).await {
            eprintln!("Warning: Failed to configure Redis LRU settings: {}", e);
            eprintln!("Redis will use default configuration.");
        }
    }

    connection
}
