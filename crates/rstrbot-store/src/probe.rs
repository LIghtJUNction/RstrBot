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
    /// Encode a filesystem path without treating a Windows drive as a URL host.
    /// SQLx's Any driver first parses a URL, then SQLite percent-decodes its
    /// filename once. Encode the full path so separators, `%`, `?` and `#`
    /// survive both steps unchanged; relative paths remain relative.
    pub fn sqlite_file_url(path: &std::path::Path) -> Result<String> {
        let path = path
            .to_str()
            .ok_or_else(|| Error::Invalid("SQLite database path must be valid UTF-8".into()))?;
        let mut options: sqlx::any::AnyConnectOptions = "sqlite:".parse()?;
        options
            .database_url
            .query_pairs_mut()
            .append_pair("path", path);
        let encoded = options
            .database_url
            .query()
            .and_then(|query| query.strip_prefix("path="))
            .ok_or_else(|| Error::Invalid("could not encode SQLite database path".into()))?;
        // Form encoding represents spaces as '+', but SQLite's filename decoder
        // only percent-decodes. A literal '+' is already encoded as `%2B`.
        Ok(format!("sqlite:{}?mode=rwc", encoded.replace('+', "%20")))
    }

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
        let schema = schema && matches!(version.as_deref(), Some("1" | "2"));
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
    use sqlx::ConnectOptions;

    #[test]
    fn sqlite_file_paths_survive_any_driver_url_roundtrip() {
        for path in [
            r"C:\Users\Runner Test\AppData\Local\Temp\existing.db",
            r"C:/Users/Runner Test\existing.db",
            r"\\server\shared folder\existing #+%25.db",
            r"\\?\C:\Users\runner\existing #+%2F.db",
            "/tmp/数据库 space? # + %2F %25.db",
            "/tmp/parent/.././file.db",
            "../relative space #+%2F.db",
        ] {
            let path = std::path::Path::new(path);
            let url = Store::sqlite_file_url(path).unwrap();
            let any: sqlx::any::AnyConnectOptions = url.parse().unwrap();
            assert!(any.database_url.host_str().is_none());
            assert!(any.database_url.fragment().is_none());
            assert_eq!(any.database_url.query(), Some("mode=rwc"));
            let sqlite = sqlx::sqlite::SqliteConnectOptions::from_url(&any.database_url).unwrap();
            assert_eq!(sqlite.get_filename(), path);
        }
    }

    #[cfg(unix)]
    #[test]
    fn sqlite_file_url_rejects_non_utf8_without_lossy_conversion() {
        use std::os::unix::ffi::OsStrExt;
        let path = std::path::Path::new(std::ffi::OsStr::from_bytes(b"/tmp/invalid-\xff.db"));
        assert!(matches!(
            Store::sqlite_file_url(path),
            Err(Error::Invalid(_))
        ));
    }

    #[tokio::test]
    async fn probe_does_not_create_a_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("absent.db");
        assert!(
            Store::probe(&Store::sqlite_file_url(&path).unwrap())
                .await
                .is_err()
        );
        assert!(!path.exists());
    }
    #[tokio::test]
    async fn probe_does_not_migrate_existing_foreign_database() {
        let temp = tempfile::tempdir().unwrap();
        let url = Store::sqlite_file_url(&temp.path().join("existing.db")).unwrap();
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
        pool.close().await;
    }

    #[tokio::test]
    async fn encoded_file_path_opens_the_exact_file_and_probe_keeps_it_read_only() {
        let temp = tempfile::tempdir().unwrap();
        let name = if cfg!(windows) {
            "数据库 space # + %25.db"
        } else {
            "数据库 space? # + %25.db"
        };
        let path = temp.path().join(name);
        let url = Store::sqlite_file_url(&path).unwrap();
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
        sqlx::query("INSERT INTO valuable_data (value) VALUES ('keep me')")
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;
        let before = std::fs::read(&path).unwrap();
        let result = Store::probe(&url).await.unwrap();
        assert!(result.has_user_tables);
        assert!(!result.rstrbot_schema);
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
}
