use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;

use super::models::*;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(data_dir: PathBuf) -> Result<Self, rusqlite::Error> {
        std::fs::create_dir_all(&data_dir).ok();
        let db_path = data_dir.join("skytrace.db");
        let conn = Connection::open(db_path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS sky_app (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                app_id     TEXT NOT NULL,
                app_uk     TEXT NOT NULL,
                token      TEXT NOT NULL,
                name       TEXT NOT NULL DEFAULT '',
                env        TEXT NOT NULL DEFAULT 'prod',
                created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
            );

            CREATE TABLE IF NOT EXISTS supplier (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL,
                code        TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                service_ids TEXT NOT NULL DEFAULT '[]',
                created_at  TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                deleted_at  TEXT
            );

            CREATE TABLE IF NOT EXISTS trace_flow (
                id             INTEGER PRIMARY KEY AUTOINCREMENT,
                name           TEXT NOT NULL,
                description    TEXT NOT NULL DEFAULT '',
                supplier_id    INTEGER,
                tags           TEXT NOT NULL DEFAULT '[]',
                is_favorite    INTEGER NOT NULL DEFAULT 0,
                sort_order     INTEGER NOT NULL DEFAULT 0,
                dynamic_params TEXT NOT NULL DEFAULT '[]',
                nodes          TEXT NOT NULL DEFAULT '[]',
                created_at     TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at     TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                deleted_at     TEXT
            );

            CREATE TABLE IF NOT EXISTS checklist_group (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                items       TEXT NOT NULL DEFAULT '[]',
                created_at  TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                deleted_at  TEXT
            );

            CREATE TABLE IF NOT EXISTS recovery_group (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                steps       TEXT NOT NULL DEFAULT '[]',
                created_at  TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now','localtime')),
                deleted_at  TEXT
            );

            CREATE TABLE IF NOT EXISTS execution_history (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                flow_id      INTEGER NOT NULL,
                input_params TEXT NOT NULL DEFAULT '{}',
                results      TEXT NOT NULL DEFAULT '{}',
                status       TEXT NOT NULL DEFAULT 'success',
                duration_ms  INTEGER NOT NULL DEFAULT 0,
                created_at   TEXT NOT NULL DEFAULT (datetime('now','localtime'))
            );
            ",
        )?;
        Self::migrate_add_deleted_at(&conn)?;
        Ok(())
    }

    fn migrate_add_deleted_at(conn: &Connection) -> Result<(), rusqlite::Error> {
        let has_col = |table: &str, col: &str| -> bool {
            conn.prepare(&format!("SELECT {} FROM {} LIMIT 0", col, table)).is_ok()
        };
        if !has_col("trace_flow", "deleted_at") {
            conn.execute_batch("ALTER TABLE trace_flow ADD COLUMN deleted_at TEXT;")?;
        }
        if !has_col("supplier", "deleted_at") {
            conn.execute_batch("ALTER TABLE supplier ADD COLUMN deleted_at TEXT;")?;
        }
        Ok(())
    }

    // ── SkyApp ──

    pub fn get_sky_apps(&self) -> Result<Vec<SkyApp>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, app_id, app_uk, token, name, env, created_at FROM sky_app ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(SkyApp {
                id: row.get(0)?,
                app_id: row.get(1)?,
                app_uk: row.get(2)?,
                token: row.get(3)?,
                name: row.get(4)?,
                env: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn save_sky_app(&self, input: SkyAppInput) -> Result<SkyApp, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sky_app (app_id, app_uk, token, name, env) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![input.app_id, input.app_uk, input.token, input.name, input.env],
        )?;
        let id = conn.last_insert_rowid();
        Ok(SkyApp {
            id,
            app_id: input.app_id,
            app_uk: input.app_uk,
            token: input.token,
            name: input.name,
            env: input.env,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    }

    pub fn delete_sky_app(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sky_app WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ── Supplier ──

    pub fn get_suppliers(&self) -> Result<Vec<Supplier>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, code, description, service_ids, created_at FROM supplier WHERE deleted_at IS NULL ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            let sids_str: String = row.get(4)?;
            let service_ids: Vec<i64> = serde_json::from_str(&sids_str).unwrap_or_default();
            Ok(Supplier {
                id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                description: row.get(3)?,
                service_ids,
                created_at: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn save_supplier(&self, input: SupplierInput) -> Result<Supplier, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let sids_json = serde_json::to_string(&input.service_ids).unwrap_or_default();
        conn.execute(
            "INSERT INTO supplier (name, code, description, service_ids) VALUES (?1, ?2, ?3, ?4)",
            params![input.name, input.code, input.description, sids_json],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Supplier {
            id,
            name: input.name,
            code: input.code,
            description: input.description,
            service_ids: input.service_ids,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    }

    pub fn delete_supplier(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute("UPDATE supplier SET deleted_at = ?1 WHERE id = ?2", params![now, id])?;
        Ok(())
    }

    pub fn get_deleted_suppliers(&self) -> Result<Vec<Supplier>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, code, description, service_ids, created_at FROM supplier WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let sids_str: String = row.get(4)?;
            let service_ids: Vec<i64> = serde_json::from_str(&sids_str).unwrap_or_default();
            Ok(Supplier {
                id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                description: row.get(3)?,
                service_ids,
                created_at: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn restore_supplier(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE supplier SET deleted_at = NULL WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn permanently_delete_supplier(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM supplier WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ── TraceFlow ──

    pub fn get_flows(&self, supplier_id: Option<i64>) -> Result<Vec<TraceFlow>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let (sql, filter_val) = match supplier_id {
            Some(sid) => (
                "SELECT id, name, description, supplier_id, tags, is_favorite, sort_order, dynamic_params, nodes, created_at, updated_at
                 FROM trace_flow WHERE supplier_id = ?1 AND deleted_at IS NULL ORDER BY sort_order DESC, id DESC",
                Some(sid),
            ),
            None => (
                "SELECT id, name, description, supplier_id, tags, is_favorite, sort_order, dynamic_params, nodes, created_at, updated_at
                 FROM trace_flow WHERE deleted_at IS NULL ORDER BY sort_order DESC, id DESC",
                None,
            ),
        };

        let mut stmt = conn.prepare(sql)?;
        let rows = if let Some(sid) = filter_val {
            stmt.query_map(params![sid], Self::map_flow)?
        } else {
            stmt.query_map([], Self::map_flow)?
        };
        rows.collect()
    }

    pub fn get_flow(&self, id: i64) -> Result<TraceFlow, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, description, supplier_id, tags, is_favorite, sort_order, dynamic_params, nodes, created_at, updated_at
             FROM trace_flow WHERE id = ?1",
            params![id],
            Self::map_flow,
        )
    }

    pub fn save_flow(&self, input: TraceFlowInput) -> Result<TraceFlow, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let tags_json = serde_json::to_string(&input.tags).unwrap_or_default();
        let params_json = serde_json::to_string(&input.dynamic_params).unwrap_or_default();
        let nodes_json = serde_json::to_string(&input.nodes).unwrap_or_default();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        if let Some(id) = input.id {
            conn.execute(
                "UPDATE trace_flow SET name=?1, description=?2, supplier_id=?3, tags=?4, dynamic_params=?5, nodes=?6, updated_at=?7 WHERE id=?8",
                params![input.name, input.description, input.supplier_id, tags_json, params_json, nodes_json, now, id],
            )?;
            drop(conn);
            return self.get_flow(id);
        }

        conn.execute(
            "INSERT INTO trace_flow (name, description, supplier_id, tags, dynamic_params, nodes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![input.name, input.description, input.supplier_id, tags_json, params_json, nodes_json, now],
        )?;
        let id = conn.last_insert_rowid();
        drop(conn);
        self.get_flow(id)
    }

    pub fn delete_flow(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute("UPDATE trace_flow SET deleted_at = ?1 WHERE id = ?2", params![now, id])?;
        Ok(())
    }

    pub fn get_deleted_flows(&self) -> Result<Vec<TraceFlow>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, supplier_id, tags, is_favorite, sort_order, dynamic_params, nodes, created_at, updated_at
             FROM trace_flow WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC",
        )?;
        let rows = stmt.query_map([], Self::map_flow)?;
        rows.collect()
    }

    pub fn restore_flow(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE trace_flow SET deleted_at = NULL WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn permanently_delete_flow(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM trace_flow WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn empty_trash(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "DELETE FROM trace_flow WHERE deleted_at IS NOT NULL;
             DELETE FROM supplier WHERE deleted_at IS NOT NULL;
             DELETE FROM recovery_group WHERE deleted_at IS NOT NULL;",
        )?;
        Ok(())
    }

    pub fn duplicate_flow(&self, id: i64) -> Result<TraceFlow, rusqlite::Error> {
        let original = self.get_flow(id)?;
        let input = TraceFlowInput {
            id: None,
            name: format!("{} (副本)", original.name),
            description: original.description,
            supplier_id: original.supplier_id,
            tags: original.tags,
            dynamic_params: original.dynamic_params,
            nodes: original.nodes,
        };
        self.save_flow(input)
    }

    pub fn toggle_flow_favorite(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE trace_flow SET is_favorite = CASE WHEN is_favorite = 1 THEN 0 ELSE 1 END WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    // ── ChecklistGroup ──

    pub fn get_checklist_groups(&self) -> Result<Vec<ChecklistGroup>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, items, created_at, updated_at
             FROM checklist_group WHERE deleted_at IS NULL ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            let items_str: String = row.get(3)?;
            Ok(ChecklistGroup {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                items: serde_json::from_str(&items_str).unwrap_or_default(),
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_checklist_group(&self, id: i64) -> Result<ChecklistGroup, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, description, items, created_at, updated_at
             FROM checklist_group WHERE id = ?1",
            params![id],
            |row| {
                let items_str: String = row.get(3)?;
                Ok(ChecklistGroup {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    items: serde_json::from_str(&items_str).unwrap_or_default(),
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
    }

    pub fn save_checklist_group(&self, input: ChecklistGroupInput) -> Result<ChecklistGroup, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let items_json = serde_json::to_string(&input.items).unwrap_or_default();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        if let Some(id) = input.id {
            conn.execute(
                "UPDATE checklist_group SET name=?1, description=?2, items=?3, updated_at=?4 WHERE id=?5",
                params![input.name, input.description, items_json, now, id],
            )?;
            drop(conn);
            return self.get_checklist_group(id);
        }

        conn.execute(
            "INSERT INTO checklist_group (name, description, items, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![input.name, input.description, items_json, now],
        )?;
        let id = conn.last_insert_rowid();
        drop(conn);
        self.get_checklist_group(id)
    }

    pub fn delete_checklist_group(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute("UPDATE checklist_group SET deleted_at = ?1 WHERE id = ?2", params![now, id])?;
        Ok(())
    }

    // ── RecoveryGroup ──

    pub fn get_recovery_groups(&self) -> Result<Vec<RecoveryGroup>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, steps, created_at, updated_at
             FROM recovery_group WHERE deleted_at IS NULL ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            let steps_str: String = row.get(3)?;
            Ok(RecoveryGroup {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                steps: serde_json::from_str(&steps_str).unwrap_or_default(),
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_recovery_group(&self, id: i64) -> Result<RecoveryGroup, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, description, steps, created_at, updated_at
             FROM recovery_group WHERE id = ?1",
            params![id],
            |row| {
                let steps_str: String = row.get(3)?;
                Ok(RecoveryGroup {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    steps: serde_json::from_str(&steps_str).unwrap_or_default(),
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
    }

    pub fn save_recovery_group(&self, input: RecoveryGroupInput) -> Result<RecoveryGroup, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let steps_json = serde_json::to_string(&input.steps).unwrap_or_default();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        if let Some(id) = input.id {
            conn.execute(
                "UPDATE recovery_group SET name=?1, description=?2, steps=?3, updated_at=?4 WHERE id=?5",
                params![input.name, input.description, steps_json, now, id],
            )?;
            drop(conn);
            return self.get_recovery_group(id);
        }

        conn.execute(
            "INSERT INTO recovery_group (name, description, steps, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![input.name, input.description, steps_json, now],
        )?;
        let id = conn.last_insert_rowid();
        drop(conn);
        self.get_recovery_group(id)
    }

    pub fn delete_recovery_group(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute("UPDATE recovery_group SET deleted_at = ?1 WHERE id = ?2", params![now, id])?;
        Ok(())
    }

    pub fn get_deleted_recovery_groups(&self) -> Result<Vec<RecoveryGroup>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, steps, created_at, updated_at
             FROM recovery_group WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let steps_str: String = row.get(3)?;
            Ok(RecoveryGroup {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                steps: serde_json::from_str(&steps_str).unwrap_or_default(),
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn restore_recovery_group(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE recovery_group SET deleted_at = NULL WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn permanently_delete_recovery_group(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM recovery_group WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn map_flow(row: &rusqlite::Row) -> Result<TraceFlow, rusqlite::Error> {
        let tags_str: String = row.get(4)?;
        let params_str: String = row.get(7)?;
        let nodes_str: String = row.get(8)?;

        Ok(TraceFlow {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            supplier_id: row.get(3)?,
            tags: serde_json::from_str(&tags_str).unwrap_or_default(),
            is_favorite: row.get::<_, i64>(5)? != 0,
            sort_order: row.get(6)?,
            dynamic_params: serde_json::from_str(&params_str).unwrap_or_default(),
            nodes: serde_json::from_str(&nodes_str).unwrap_or_default(),
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    }
}
