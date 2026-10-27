use crate::*;
use futures_util::TryStreamExt;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// A consistent database snapshot. This deliberately does not claim to include
/// external plugin state, media files or the separately protected encryption key.
/// It contains password hashes and must be handled as sensitive backup material.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    pub format: String,
    pub schema_version: i64,
    pub created_at: i64,
    pub source_backend: String,
    pub includes_external_state: bool,
    pub tables: BTreeMap<String, Vec<Value>>,
    pub sha256: String,
}

impl Store {
    /// Export one database snapshot, excluding session bearer-token hashes.
    pub async fn export_backup(&self, actor: &User) -> Result<Backup> {
        self.require_instance_admin(&actor.id).await?;
        self.export_backup_local().await
    }

    /// Trusted local maintenance operation. Do not route this method to HTTP.
    pub async fn export_backup_local(&self) -> Result<Backup> {
        let mut tx = self.pool.begin().await?;
        if self.backend == "postgresql" {
            sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
                .execute(&mut *tx)
                .await?;
        }
        let mut tables = BTreeMap::new();
        let mut total = 0usize;
        for table in TABLES {
            let sql = format!(
                "SELECT {} FROM {} ORDER BY {}",
                table
                    .columns
                    .iter()
                    .map(|c| c.name)
                    .collect::<Vec<_>>()
                    .join(", "),
                table.name,
                table.columns[0].name
            );
            let mut rows = sqlx::query(&sql).fetch(&mut *tx);
            let mut encoded = Vec::new();
            while let Some(row) = rows.try_next().await? {
                if encoded.len() >= 250_000 {
                    return Err(Error::Invalid(
                        "snapshot exceeds the bounded export size; use a database-native backup"
                            .into(),
                    ));
                }
                let mut item = serde_json::Map::new();
                for column in table.columns {
                    let value = match column.kind {
                        Kind::Text => Value::String(row.try_get(column.name)?),
                        Kind::Int => Value::from(row.try_get::<i64, _>(column.name)?),
                        Kind::NullableInt => row
                            .try_get::<Option<i64>, _>(column.name)?
                            .map(Value::from)
                            .unwrap_or(Value::Null),
                    };
                    item.insert(column.name.to_owned(), value);
                }
                let item = Value::Object(item);
                total += serde_json::to_vec(&item)?.len();
                if total > 256 * 1024 * 1024 {
                    return Err(Error::Invalid(
                        "snapshot exceeds 256 MiB; use a database-native backup".into(),
                    ));
                }
                encoded.push(item);
            }
            tables.insert(table.name.to_owned(), encoded);
        }
        tx.commit().await?;
        let sha256 = checksum(&tables)?;
        Ok(Backup {
            format: "rstrbot-db-v1".into(),
            schema_version: 1,
            created_at: now(),
            source_backend: self.backend.into(),
            includes_external_state: false,
            tables,
            sha256,
        })
    }

    /// Restore atomically into an uninitialized destination. Source remains
    /// untouched. The restored database starts in recovery/quarantine mode.
    pub async fn import_backup(&self, backup: &Backup) -> Result<()> {
        if backup.format != "rstrbot-db-v1"
            || backup.schema_version != 1
            || checksum(&backup.tables)? != backup.sha256
        {
            return Err(Error::Invalid("backup version or checksum mismatch".into()));
        }
        if serde_json::to_vec(&backup.tables)?.len() > 256 * 1024 * 1024 {
            return Err(Error::Invalid("backup exceeds 256 MiB".into()));
        }
        if backup.tables.len() != TABLES.len()
            || TABLES.iter().any(|t| !backup.tables.contains_key(t.name))
        {
            return Err(Error::Invalid(
                "backup table manifest is incomplete or unknown".into(),
            ));
        }
        let mut tx = self.pool.begin().await?;
        let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&mut *tx)
            .await?;
        if users != 0 {
            return Err(Error::Invalid(
                "restore target must be a new, empty database".into(),
            ));
        }
        sqlx::query("DELETE FROM counters")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM instance_state")
            .execute(&mut *tx)
            .await?;
        for table in TABLES {
            let rows = backup
                .tables
                .get(table.name)
                .ok_or_else(|| Error::Invalid("missing table".into()))?;
            if rows.len() > 250_000 {
                return Err(Error::Invalid("backup table exceeds row limit".into()));
            }
            let columns = table
                .columns
                .iter()
                .map(|c| c.name)
                .collect::<Vec<_>>()
                .join(", ");
            let placeholders = (1..=table.columns.len())
                .map(|n| format!("${n}"))
                .collect::<Vec<_>>()
                .join(", ");
            let sql = format!(
                "INSERT INTO {} ({columns}) VALUES ({placeholders})",
                table.name
            );
            for row in rows {
                let object = row
                    .as_object()
                    .filter(|o| o.len() == table.columns.len())
                    .ok_or_else(|| Error::Invalid("invalid backup row".into()))?;
                let mut query = sqlx::query(&sql);
                for column in table.columns {
                    let value = object
                        .get(column.name)
                        .ok_or_else(|| Error::Invalid("missing backup field".into()))?;
                    query =
                        match column.kind {
                            Kind::Text => query.bind(value.as_str().ok_or_else(|| {
                                Error::Invalid("expected backup text field".into())
                            })?),
                            Kind::Int => query.bind(value.as_i64().ok_or_else(|| {
                                Error::Invalid("expected backup integer field".into())
                            })?),
                            Kind::NullableInt => query.bind(if value.is_null() {
                                None
                            } else {
                                Some(value.as_i64().ok_or_else(|| {
                                    Error::Invalid("invalid nullable integer".into())
                                })?)
                            }),
                        };
                }
                query.execute(&mut *tx).await?;
            }
        }
        let invalid_owners: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workspaces w WHERE NOT EXISTS (SELECT 1 FROM members m WHERE m.workspace_id = w.id AND m.role = 'owner')")
            .fetch_one(&mut *tx).await?;
        let invalid_budget: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM budgets b WHERE b.reserved_microusd != COALESCE((SELECT SUM(r.amount_microusd) FROM reservations r WHERE r.workspace_id = b.workspace_id AND r.state IN ('reserved','unknown')), 0)")
            .fetch_one(&mut *tx).await?;
        if invalid_owners != 0 || invalid_budget != 0 {
            return Err(Error::Invalid(
                "backup ownership or reservation ledger is inconsistent".into(),
            ));
        }
        sqlx::query("INSERT INTO instance_state (key, value) VALUES ('recovery_mode', 'true') ON CONFLICT (key) DO UPDATE SET value = excluded.value")
            .execute(&mut *tx).await?;
        // Existing reservations are not released or changed into new operations.
        // Authentication sessions are intentionally not part of the snapshot.
        tx.commit().await?;
        Ok(())
    }
}

fn checksum(tables: &BTreeMap<String, Vec<Value>>) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(tables)?)))
}

#[derive(Clone, Copy)]
enum Kind {
    Text,
    Int,
    NullableInt,
}
struct Column {
    name: &'static str,
    kind: Kind,
}
struct Table {
    name: &'static str,
    columns: &'static [Column],
}
const fn text(name: &'static str) -> Column {
    Column {
        name,
        kind: Kind::Text,
    }
}
const fn int(name: &'static str) -> Column {
    Column {
        name,
        kind: Kind::Int,
    }
}
const fn nullable_int(name: &'static str) -> Column {
    Column {
        name,
        kind: Kind::NullableInt,
    }
}
const TABLES: &[Table] = &[
    Table {
        name: "users",
        columns: &[
            text("id"),
            text("username"),
            text("password_hash"),
            int("instance_admin"),
            int("disabled"),
            int("created_at"),
        ],
    },
    Table {
        name: "workspaces",
        columns: &[text("id"), text("name"), int("revision"), int("created_at")],
    },
    Table {
        name: "members",
        columns: &[text("workspace_id"), text("user_id"), text("role")],
    },
    Table {
        name: "resources",
        columns: &[
            text("id"),
            text("workspace_id"),
            text("owner_id"),
            text("kind"),
            text("visibility"),
            int("version"),
            text("data"),
            int("created_at"),
            int("updated_at"),
        ],
    },
    Table {
        name: "budgets",
        columns: &[
            text("workspace_id"),
            int("limit_microusd"),
            int("spent_microusd"),
            int("reserved_microusd"),
        ],
    },
    Table {
        name: "reservations",
        columns: &[
            text("id"),
            text("workspace_id"),
            text("user_id"),
            text("operation_id"),
            int("amount_microusd"),
            nullable_int("actual_microusd"),
            text("state"),
            int("created_at"),
            int("updated_at"),
        ],
    },
    Table {
        name: "operations",
        columns: &[
            text("workspace_id"),
            text("user_id"),
            text("operation_key"),
            text("fingerprint"),
            text("state"),
            text("result"),
            int("created_at"),
        ],
    },
    Table {
        name: "events",
        columns: &[
            int("id"),
            text("workspace_id"),
            text("owner_id"),
            text("visibility"),
            text("resource_id"),
            text("kind"),
            text("data"),
            int("created_at"),
        ],
    },
    Table {
        name: "counters",
        columns: &[text("name"), int("value")],
    },
    Table {
        name: "instance_state",
        columns: &[text("key"), text("value")],
    },
];
