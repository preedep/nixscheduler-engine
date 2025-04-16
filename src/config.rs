use std::env;

#[derive(Debug, Clone)]
pub enum ShardMode {
    Local,
    Distributed {
        shard_id: usize,
        total_shards: usize,
    },
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub shard_mode: ShardMode,
    pub database_url: String,
    pub tick_interval_secs: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {

        dotenv::dotenv().ok();

        let shard_mode = match env::var("SHARD_MODE").unwrap_or_else(|_| "local".to_string()).as_str() {
            "distributed" => {
                let shard_id = env::var("SHARD_ID").expect("SHARD_ID is required in distributed mode")
                    .parse::<usize>().expect("Invalid SHARD_ID");

                let total = env::var("TOTAL_SHARDS").unwrap_or_else(|_| "1".to_string())
                    .parse::<usize>().expect("Invalid TOTAL_SHARDS");

                ShardMode::Distributed {
                    shard_id,
                    total_shards: total,
                }
            }
            _ => ShardMode::Local,
        };

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://jobs.db".to_string());

        let tick_interval_secs = env::var("TICK_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        AppConfig {
            shard_mode,
            database_url,
            tick_interval_secs,
        }
    }
}