# Testing Redis LRU Functionality

## Prerequisites

1. Redis server running: `redis-server`
2. Set environment variable: `export REDIS_URL=redis://localhost:6379`
3. Add LRU configuration to `.env` file (see `config/redis_lru.env.example`)

## Manual Testing with redis-cli

```bash
# Check if key exists with TTL
redis-cli TTL user_reduced:123

# Set a test key manually
redis-cli SETEX user_reduced:999 60 '{"id":999,"name":"Test User"}'

# Get the test key
redis-cli GET user_reduced:999

# Check memory usage
redis-cli INFO memory

# Check eviction stats
redis-cli INFO stats | grep evicted

# Monitor all Redis commands
redis-cli MONITOR
```

## Application Testing

Use the updated functions in your Rust code:

```rust
use crate::rds_mutate::user_reduced::{save_user_reduced, get_user_reduced, refresh_user_reduced_ttl};
use crate::user_mutate::models::UserReduced;

async fn test_lru_functionality(conn: RdsConn) -> Result<(), Status> {
    let user = UserReduced {
        id: 123,
        name: "Test User".to_string(),
    };
    
    // Save with 1 hour TTL
    save_user_reduced(conn.clone(), user, Some(3600)).await?;
    
    // Retrieve user
    let retrieved = get_user_reduced(conn.clone(), 123).await?;
    println!("Retrieved: {} - {}", retrieved.id, retrieved.name);
    
    // Refresh TTL to 2 hours
    refresh_user_reduced_ttl(conn, 123, Some(7200)).await?;
    
    Ok(())
}
```

## Monitoring LRU Behavior

1. **Memory Usage**: `redis-cli INFO memory | grep used_memory`
2. **Eviction Events**: `redis-cli INFO stats | grep evicted_keys`
3. **Key TTL**: `redis-cli TTL user_reduced:YOUR_ID`
4. **Active Keys**: `redis-cli DBSIZE` 