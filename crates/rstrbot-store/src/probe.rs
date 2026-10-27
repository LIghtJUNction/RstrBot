use crate::*;

/// A connectivity inspection. It never creates tables or applies migrations.
#[derive(Debug, Serialize)]
pub struct Probe {
    pub backend: &'static str,
    pub initialized: bool,
    pub has_user_tables: bool,
    pub rstrbot_schema: bool,
}
impl Store {
    pub async fn probe(url: &str) -> Result<Probe> {
        sqlx::any::install_default_drivers();
        let sqlite = url.starts_with("sqlite:");
        if !sqlite && !url.starts_with("postgres:") && !url.starts_with("postgresql:") {
            return Err(Error::Invalid("use SQLite or PostgreSQL".into()));
        }
        // Opening a new SQLite file is a mutation too. A connection test uses
        // read-only mode; file creation happens only during explicit selection.
        let probe_url = if sqlite && !url.contains(":memory:") {
            let (path, query) = url.split_once('?').unwrap_or((url, ""));
            let mut parts: Vec<&str> = query
                .split('&')
                .filter(|p| !p.is_empty() && !p.starts_with("mode="))
                .collect();
            parts.push("mode=ro");
            format!("{path}?{}", parts.join("&"))
        } else {
            url.to_owned()
        };
        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&probe_url)
            .await?;
        let names = if sqlite {
            sqlx::query_scalar::<_, String>(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            )
            .fetch_all(&pool)
            .await?
        } else {
            // information_schema.sql_identifier is backed by PostgreSQL's
            // NAME type, which the cross-database Any driver cannot decode.
            // Cast metadata explicitly; an empty schema otherwise hides this
            // incompatibility until the first restart of a populated database.
            sqlx::query_scalar::<_,String>("SELECT CAST(table_name AS TEXT) FROM information_schema.tables WHERE table_schema=current_schema() AND table_type='BASE TABLE'").fetch_all(&pool).await?
        };
        let schema = [
            "users",
            "workspaces",
            "members",
            "instance_state",
            "resources",
        ]
        .iter()
        .all(|name| names.iter().any(|n| n == name));
        let version = if schema {
            sqlx::query_scalar::<_, String>(
                "SELECT value FROM instance_state WHERE key='schema_version'",
            )
            .fetch_optional(&pool)
            .await?
        } else {
            None
        };
        // Table names alone are not proof that this installation understands
        // the schema. Refuse unknown future versions without migrating them.
        let schema = schema && version.as_deref() == Some("1");
        let initialized = if schema {
            sqlx::query_scalar::<_, String>(
                "SELECT value FROM instance_state WHERE key='initialized'",
            )
            .fetch_optional(&pool)
            .await?
            .is_some()
        } else {
            false
        };
        pool.close().await;
        Ok(Probe {
            backend: if sqlite { "sqlite" } else { "postgresql" },
            initialized,
            has_user_tables: !names.is_empty(),
            rstrbot_schema: schema,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn probe_does_not_create_a_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("absent.db");
        assert!(
            Store::probe(&format!("sqlite://{}?mode=rwc", path.display()))
                .await
                .is_err()
        );
        assert!(!path.exists());
    }
    #[tokio::test]
    async fn probe_does_not_migrate_existing_foreign_database() {
        let temp = tempfile::tempdir().unwrap();
        let url = format!(
            "sqlite://{}?mode=rwc",
            temp.path().join("existing.db").display()
        );
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect(&url)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE valuable_data (value TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        let result = Store::probe(&url).await.unwrap();
        assert!(result.has_user_tables);
        assert!(!result.rstrbot_schema);
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE type='table'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 1);
    }
}
