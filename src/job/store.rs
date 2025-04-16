use crate::job::model::Job;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use std::fs;
use std::path::Path;
use std::sync::Arc;

#[async_trait]
pub trait JobStore: Send + Sync {
    async fn load_jobs(&self) -> Vec<Job>;
    async fn update_last_run(&self, job_id: &str, dt: DateTime<Utc>);
}

#[derive(Clone)]
pub struct SqliteJobStore {
    pool: Arc<Pool<Sqlite>>,
}

impl SqliteJobStore {
    pub async fn new(db_url: &str) -> Self {
        // Ensure the directory exists

        if db_url.starts_with("sqlite://") {
            let path = db_url.trim_start_matches("sqlite://");

            let path_obj = Path::new(path);
            if let Some(parent) = path_obj.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    panic!("Failed to create DB folder {:?}: {}", parent, e);
                }
            }
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(db_url)
            .await
            .expect("Failed to connect to SQLite");

        // Auto migration
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                cron TEXT NOT NULL,
                task_type TEXT NOT NULL,
                payload TEXT,
                last_run TEXT
            )
            "#,
        )
            .execute(&pool)
            .await
            .unwrap();

        SqliteJobStore {
            pool: Arc::new(pool),
        }
    }
}

#[async_trait]
impl JobStore for SqliteJobStore {
    async fn load_jobs(&self) -> Vec<Job> {
        let rows = sqlx::query(r#"SELECT id, name, cron, task_type, payload, last_run FROM jobs"#)
            .fetch_all(&*self.pool)
            .await
            .unwrap();

        rows.into_iter()
            .map(|r| Job {
                id: r.try_get("id").unwrap(),
                name: r.try_get("name").unwrap(),
                cron: r.try_get("cron").unwrap(),
                task_type: r.try_get("task_type").unwrap(),
                payload: r.try_get("payload").unwrap_or_default(),
                last_run: r
                    .try_get::<Option<String>, _>("last_run")
                    .unwrap()
                    .map(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .unwrap()
                            .with_timezone(&Utc)
                    }),
            })
            .collect()
    }

    async fn update_last_run(&self, job_id: &str, dt: DateTime<Utc>) {
        sqlx::query(r#"UPDATE jobs SET last_run = ? WHERE id = ?"#)
            .bind(dt.to_rfc3339())
            .bind(job_id)
            .execute(&*self.pool)
            .await
            .unwrap();
    }
}
