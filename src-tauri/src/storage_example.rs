// Tauri SQL 插件使用示例
// 这是一个独立的示例文件，展示如何使用 tauri-plugin-sql 进行数据库操作
// 不影响现有代码，仅作为参考

use tauri_plugin_sql::{Migration, MigrationKind};
use uuid::{uuid, Uuid};

// 定义数据库表结构
// tracks 表用于存储音乐元数据
// CREATE TABLE tracks (
//     id TEXT PRIMARY KEY,           -- 基于URL生成的UUID
//     title TEXT NOT NULL,           -- 音乐标题
//     artist TEXT NOT NULL,          -- 艺术家
//     cover_url TEXT,                -- 封面URL
//     duration REAL,                 -- 时长(秒)
//     src TEXT NOT NULL,             -- 来源URL
//     created_at INTEGER NOT NULL,   -- 创建时间(时间戳)
//     updated_at INTEGER NOT NULL    -- 更新时间(时间戳)
// );

// 定义 URL 命名空间，用于生成确定性的 UUID
const URL_NAMESPACE: Uuid = uuid!("49be3fd4-a796-4392-9ce8-b7af0d3866f3");

/// 从 URL 生成确定性的 UUID
/// 相同的 URL 总是生成相同的 UUID，可用于去重
pub fn get_uuid_from_url(url: &str) -> Uuid {
    Uuid::new_v5(&URL_NAMESPACE, url.as_bytes())
}

/// 数据库迁移定义
/// Tauri SQL 插件会在应用启动时自动执行这些迁移

// ============================================================================
// 以下是如何在 Tauri 命令中使用 SQL 插件的示例代码
// ============================================================================

/*
// 在 lib.rs 中注册插件：

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            save_track,
            get_track,
            get_all_tracks,
            delete_track,
            search_tracks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
*/

/*
// 示例 1: 保存或更新音乐元数据

#[tauri::command]
async fn save_track(
    title: String,
    artist: String,
    cover_url: String,
    duration: f32,
    src: String,
    db: tauri_plugin_sql::DbInstances<'_>,
) -> Result<String, String> {
    let id = get_uuid_from_url(&src).to_string();
    let now = chrono::Utc::now().timestamp();

    // 使用 UPSERT (INSERT OR REPLACE) 语法
    // 如果记录已存在则更新，否则插入新记录
    let sql = r#"
        INSERT INTO tracks (id, title, artist, cover_url, duration, src, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            artist = excluded.artist,
            cover_url = excluded.cover_url,
            duration = excluded.duration,
            updated_at = excluded.updated_at
    "#;

    db.get("main")?.execute(
        sql,
        &[&id, &title, &artist, &cover_url, &duration, &src, &now, &now],
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(id)
}
*/

/*
// 示例 2: 根据 ID 查询单个音乐

#[tauri::command]
async fn get_track(
    id: String,
    db: tauri_plugin_sql::DbInstances<'_>,
) -> Result<Option<TrackView>, String> {
    let sql = "SELECT id, title, artist, cover_url, duration FROM tracks WHERE id = ?";

    let result = db.get("main")?.select(sql, &[&id]).await
        .map_err(|e| e.to_string())?;

    if result.is_empty() {
        Ok(None)
    } else {
        let row = &result[0];
        Ok(Some(TrackView {
            id: row.get("id").unwrap_or_default(),
            title: row.get("title").unwrap_or_default(),
            artist: row.get("artist").unwrap_or_default(),
            cover_url: row.get("cover_url").unwrap_or_default(),
            duration: row.get::<f32>("duration").unwrap_or(0.0),
        }))
    }
}
*/

/*
// 示例 3: 查询所有音乐

#[tauri::command]
async fn get_all_tracks(
    db: tauri_plugin_sql::DbInstances<'_>,
) -> Result<Vec<TrackView>, String> {
    let sql = "SELECT id, title, artist, cover_url, duration FROM tracks ORDER BY created_at DESC";

    let result = db.get("main")?.select(sql, &[]).await
        .map_err(|e| e.to_string())?;

    let tracks: Result<Vec<TrackView>, _> = result.iter().map(|row| {
        Ok(TrackView {
            id: row.get("id").unwrap_or_default(),
            title: row.get("title").unwrap_or_default(),
            artist: row.get("artist").unwrap_or_default(),
            cover_url: row.get("cover_url").unwrap_or_default(),
            duration: row.get::<f32>("duration").unwrap_or(0.0),
        })
    }).collect();

    tracks
}
*/

/*
// 示例 4: 根据 ID 删除音乐

#[tauri::command]
async fn delete_track(
    id: String,
    db: tauri_plugin_sql::DbInstances<'_>,
) -> Result<(), String> {
    let sql = "DELETE FROM tracks WHERE id = ?";

    db.get("main")?.execute(sql, &[&id]).await
        .map_err(|e| e.to_string())?;

    Ok(())
}
*/

/*
// 示例 5: 搜索音乐 (根据标题或艺术家)

#[tauri::command]
async fn search_tracks(
    keyword: String,
    db: tauri_plugin_sql::DbInstances<'_>,
) -> Result<Vec<TrackView>, String> {
    let sql = r#"
        SELECT id, title, artist, cover_url, duration
        FROM tracks
        WHERE title LIKE ? OR artist LIKE ?
        ORDER BY created_at DESC
        LIMIT 100
    "#;

    let pattern = format!("%{}%", keyword);

    let result = db.get("main")?.select(sql, &[&pattern, &pattern]).await
        .map_err(|e| e.to_string())?;

    let tracks: Result<Vec<TrackView>, _> = result.iter().map(|row| {
        Ok(TrackView {
            id: row.get("id").unwrap_or_default(),
            title: row.get("title").unwrap_or_default(),
            artist: row.get("artist").unwrap_or_default(),
            cover_url: row.get("cover_url").unwrap_or_default(),
            duration: row.get::<f32>("duration").unwrap_or(0.0),
        })
    }).collect();

    tracks
}
*/

/*
// 示例 6: 批量插入音乐

#[tauri::command]
async fn save_tracks_batch(
    tracks: Vec<TrackView>,
    db: tauri_plugin_sql::DbInstances<'_>,
) -> Result<usize, String> {
    let now = chrono::Utc::now().timestamp();

    // 使用事务批量插入
    let db_instance = db.get("main")?;

    // 开启事务
    db_instance.execute("BEGIN TRANSACTION", &[]).await
        .map_err(|e| e.to_string())?;

    let mut count = 0;
    for track in tracks {
        let sql = r#"
            INSERT OR REPLACE INTO tracks (id, title, artist, cover_url, duration, src, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let id = get_uuid_from_url(&track.id).to_string(); // 假设 track.id 就是 src

        db_instance.execute(
            sql,
            &[&id, &track.title, &track.artist, &track.cover_url, &track.duration, &track.id, &now, &now],
        ).await
        .map_err(|e| {
            // 出错时回滚
            let _ = db_instance.execute("ROLLBACK", &[]).await;
            e.to_string()
        })?;

        count += 1;
    }

    // 提交事务
    db_instance.execute("COMMIT", &[]).await
        .map_err(|e| e.to_string())?;

    Ok(count)
}
*/

/*
// 示例 7: 清理旧数据 (删除超过 N 天未更新的记录)

#[tauri::command]
async fn cleanup_old_tracks(
    days: i64,
    db: tauri_plugin_sql::DbInstances<'_>,
) -> Result<usize, String> {
    let cutoff_time = chrono::Utc::now().timestamp() - (days * 86400);

    let sql = "DELETE FROM tracks WHERE updated_at < ?";

    db.get("main")?.execute(sql, &[&cutoff_time]).await
        .map_err(|e| e.to_string())?;

    // 返回删除的行数
    let count_sql = "SELECT changes() as count";
    let result = db.get("main")?.select(count_sql, &[]).await
        .map_err(|e| e.to_string())?;

    let count: i64 = result[0].get("count").unwrap_or(0);
    Ok(count as usize)
}
*/

/*
// 示例 8: 统计信息

#[tauri::command]
async fn get_statistics(
    db: tauri_plugin_sql::DbInstances<'_>,
) -> Result<TrackStatistics, String> {
    let count_sql = "SELECT COUNT(*) as count FROM tracks";
    let result = db.get("main")?.select(count_sql, &[]).await
        .map_err(|e| e.to_string())?;

    let total_tracks: i64 = result[0].get("count").unwrap_or(0);

    let duration_sql = "SELECT SUM(duration) as total_duration FROM tracks";
    let result = db.get("main")?.select(duration_sql, &[]).await
        .map_err(|e| e.to_string())?;

    let total_duration: f64 = result[0].get("total_duration").unwrap_or(0.0);

    Ok(TrackStatistics {
        total_tracks: total_tracks as usize,
        total_duration: total_duration as f32,
    })
}

#[derive(serde::Serialize)]
struct TrackStatistics {
    total_tracks: usize,
    total_duration: f32,
}
*/
// ============================================================================
// 测试示例
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_uuid_from_url() {
        // 测试相同的 URL 生成相同的 UUID
        let url1 = "https://example.com/music/123";
        let url2 = "https://example.com/music/123";

        assert_eq!(get_uuid_from_url(url1), get_uuid_from_url(url2));

        // 测试不同的 URL 生成不同的 UUID
        let url3 = "https://example.com/music/456";
        assert_ne!(get_uuid_from_url(url1), get_uuid_from_url(url3));
    }

    #[test]
    fn test_migrations_defined() {
        let migrations = get_migrations();
        assert_eq!(migrations.len(), 3);
        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[0].description, "create_tracks_table");
    }
}

// ============================================================================
// CRUD 操作测试用例
// ============================================================================

// 注意：要在测试中使用这些测试，需要在 Cargo.toml 中添加测试依赖：
// [dev-dependencies]
// rusqlite = "0.32"

/*
#[cfg(test)]
mod crud_tests {
    use super::*;
    use rusqlite::{Connection, params};
    use std::time::SystemTime;

    // 测试用的 Track 结构
    #[derive(Debug, Clone)]
    struct TestTrack {
        id: String,
        title: String,
        artist: String,
        cover_url: String,
        duration: f32,
        src: String,
    }

    impl TestTrack {
        fn new(title: &str, artist: &str, src: &str) -> Self {
            TestTrack {
                id: get_uuid_from_url(src).to_string(),
                title: title.to_string(),
                artist: artist.to_string(),
                cover_url: format!("https://example.com/cover/{}.jpg", title),
                duration: 180.0,
                src: src.to_string(),
            }
        }
    }

    // 辅助函数：创建测试数据库
    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        // 创建表
        conn.execute(
            "CREATE TABLE tracks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                cover_url TEXT,
                duration REAL,
                src TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        ).unwrap();

        conn
    }

    // 辅助函数：插入测试数据
    fn insert_track(conn: &Connection, track: &TestTrack) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO tracks (id, title, artist, cover_url, duration, src, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &track.id,
                &track.title,
                &track.artist,
                &track.cover_url,
                &track.duration,
                &track.src,
                now,
                now,
            ],
        ).unwrap();
    }

    // 辅助函数：查询所有记录
    fn get_all_tracks(conn: &Connection) -> Vec<TestTrack> {
        let mut stmt = conn.prepare("SELECT id, title, artist, cover_url, duration, src FROM tracks ORDER BY created_at").unwrap();
        let tracks = stmt.query_map([], |row| {
            Ok(TestTrack {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                cover_url: row.get(3)?,
                duration: row.get(4)?,
                src: row.get(5)?,
            })
        }).unwrap();

        tracks.filter_map(|t| t.ok()).collect()
    }

    // 测试 1: CREATE - 插入新记录
    #[test]
    fn test_create_track() {
        let conn = create_test_db();
        let track = TestTrack::new("Test Song", "Test Artist", "https://example.com/test.mp3");

        insert_track(&conn, &track);

        let tracks = get_all_tracks(&conn);
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "Test Song");
        assert_eq!(tracks[0].artist, "Test Artist");
    }

    // 测试 2: READ - 根据 ID 查询单条记录
    #[test]
    fn test_read_track_by_id() {
        let conn = create_test_db();
        let track = TestTrack::new("Test Song", "Test Artist", "https://example.com/test.mp3");

        insert_track(&conn, &track);

        let result: Option<TestTrack> = conn.query_row(
            "SELECT id, title, artist, cover_url, duration, src FROM tracks WHERE id = ?1",
            params![&track.id],
            |row| {
                Ok(TestTrack {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    artist: row.get(2)?,
                    cover_url: row.get(3)?,
                    duration: row.get(4)?,
                    src: row.get(5)?,
                })
            },
        ).ok();

        assert!(result.is_some());
        let found_track = result.unwrap();
        assert_eq!(found_track.title, "Test Song");
        assert_eq!(found_track.artist, "Test Artist");
    }

    // 测试 3: READ - 查询所有记录
    #[test]
    fn test_read_all_tracks() {
        let conn = create_test_db();

        insert_track(&conn, &TestTrack::new("Song 1", "Artist 1", "https://example.com/1.mp3"));
        insert_track(&conn, &TestTrack::new("Song 2", "Artist 2", "https://example.com/2.mp3"));
        insert_track(&conn, &TestTrack::new("Song 3", "Artist 3", "https://example.com/3.mp3"));

        let tracks = get_all_tracks(&conn);
        assert_eq!(tracks.len(), 3);
        assert_eq!(tracks[0].title, "Song 1");
        assert_eq!(tracks[1].title, "Song 2");
        assert_eq!(tracks[2].title, "Song 3");
    }

    // 测试 4: UPDATE - 更新现有记录
    #[test]
    fn test_update_track() {
        let conn = create_test_db();
        let mut track = TestTrack::new("Test Song", "Test Artist", "https://example.com/test.mp3");

        insert_track(&conn, &track);

        // 更新记录
        track.title = "Updated Song".to_string();
        track.artist = "Updated Artist".to_string();

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "UPDATE tracks SET title = ?1, artist = ?2, updated_at = ?3 WHERE id = ?4",
            params![&track.title, &track.artist, now, &track.id],
        ).unwrap();

        // 验证更新
        let result: Option<TestTrack> = conn.query_row(
            "SELECT id, title, artist, cover_url, duration, src FROM tracks WHERE id = ?1",
            params![&track.id],
            |row| {
                Ok(TestTrack {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    artist: row.get(2)?,
                    cover_url: row.get(3)?,
                    duration: row.get(4)?,
                    src: row.get(5)?,
                })
            },
        ).ok();

        assert!(result.is_some());
        let found_track = result.unwrap();
        assert_eq!(found_track.title, "Updated Song");
        assert_eq!(found_track.artist, "Updated Artist");
    }

    // 测试 5: UPSERT - 插入或更新
    #[test]
    fn test_upsert_track() {
        let conn = create_test_db();
        let track = TestTrack::new("Test Song", "Test Artist", "https://example.com/test.mp3");

        // 第一次插入
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO tracks (id, title, artist, cover_url, duration, src, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &track.id,
                &track.title,
                &track.artist,
                &track.cover_url,
                &track.duration,
                &track.src,
                now,
                now,
            ],
        ).unwrap();

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);

        // 使用 UPSERT 更新
        let mut updated_track = track.clone();
        updated_track.title = "Upserted Song".to_string();

        conn.execute(
            "INSERT INTO tracks (id, title, artist, cover_url, duration, src, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                artist = excluded.artist,
                cover_url = excluded.cover_url,
                duration = excluded.duration,
                updated_at = excluded.updated_at",
            params![
                &updated_track.id,
                &updated_track.title,
                &updated_track.artist,
                &updated_track.cover_url,
                &updated_track.duration,
                &updated_track.src,
                now,
                now,
            ],
        ).unwrap();

        // 验证仍然只有一条记录，但内容已更新
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);

        let result: Option<TestTrack> = conn.query_row(
            "SELECT id, title, artist, cover_url, duration, src FROM tracks WHERE id = ?1",
            params![&track.id],
            |row| {
                Ok(TestTrack {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    artist: row.get(2)?,
                    cover_url: row.get(3)?,
                    duration: row.get(4)?,
                    src: row.get(5)?,
                })
            },
        ).ok();

        assert!(result.is_some());
        let found_track = result.unwrap();
        assert_eq!(found_track.title, "Upserted Song");
    }

    // 测试 6: DELETE - 删除记录
    #[test]
    fn test_delete_track() {
        let conn = create_test_db();
        let track = TestTrack::new("Test Song", "Test Artist", "https://example.com/test.mp3");

        insert_track(&conn, &track);

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);

        // 删除记录
        conn.execute("DELETE FROM tracks WHERE id = ?1", params![&track.id]).unwrap();

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);
    }

    // 测试 7: SEARCH - 搜索功能
    #[test]
    fn test_search_tracks_by_title() {
        let conn = create_test_db();

        insert_track(&conn, &TestTrack::new("Amazing Song", "Artist A", "https://example.com/1.mp3"));
        insert_track(&conn, &TestTrack::new("Beautiful Music", "Artist B", "https://example.com/2.mp3"));
        insert_track(&conn, &TestTrack::new("Crazy Beat", "Artist C", "https://example.com/3.mp3"));
        insert_track(&conn, &TestTrack::new("Dance Party", "Artist D", "https://example.com/4.mp3"));

        // 搜索包含 "Song" 的记录
        let pattern = "%Song%";
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, cover_url, duration, src FROM tracks WHERE title LIKE ?1"
        ).unwrap();

        let tracks: Vec<TestTrack> = stmt.query_map(params![pattern], |row| {
            Ok(TestTrack {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                cover_url: row.get(3)?,
                duration: row.get(4)?,
                src: row.get(5)?,
            })
        }).unwrap().filter_map(|t| t.ok()).collect();

        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "Amazing Song");
    }

    // 测试 8: SEARCH - 按艺术家搜索
    #[test]
    fn test_search_tracks_by_artist() {
        let conn = create_test_db();

        insert_track(&conn, &TestTrack::new("Song 1", "Taylor Swift", "https://example.com/1.mp3"));
        insert_track(&conn, &TestTrack::new("Song 2", "Ed Sheeran", "https://example.com/2.mp3"));
        insert_track(&conn, &TestTrack::new("Song 3", "Taylor Swift", "https://example.com/3.mp3"));
        insert_track(&conn, &TestTrack::new("Song 4", "Adele", "https://example.com/4.mp3"));

        // 搜索 "Taylor Swift" 的记录
        let pattern = "%Taylor Swift%";
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, cover_url, duration, src FROM tracks WHERE artist LIKE ?1"
        ).unwrap();

        let tracks: Vec<TestTrack> = stmt.query_map(params![pattern], |row| {
            Ok(TestTrack {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                cover_url: row.get(3)?,
                duration: row.get(4)?,
                src: row.get(5)?,
            })
        }).unwrap().filter_map(|t| t.ok()).collect();

        assert_eq!(tracks.len(), 2);
    }

    // 测试 9: BATCH - 批量插入
    #[test]
    fn test_batch_insert() {
        let conn = create_test_db();

        let tracks: Vec<TestTrack> = vec![
            TestTrack::new("Song 1", "Artist 1", "https://example.com/1.mp3"),
            TestTrack::new("Song 2", "Artist 2", "https://example.com/2.mp3"),
            TestTrack::new("Song 3", "Artist 3", "https://example.com/3.mp3"),
            TestTrack::new("Song 4", "Artist 4", "https://example.com/4.mp3"),
            TestTrack::new("Song 5", "Artist 5", "https://example.com/5.mp3"),
        ];

        // 使用事务批量插入
        let tx = conn.transaction().unwrap();

        for track in &tracks {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            tx.execute(
                "INSERT INTO tracks (id, title, artist, cover_url, duration, src, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    &track.id,
                    &track.title,
                    &track.artist,
                    &track.cover_url,
                    &track.duration,
                    &track.src,
                    now,
                    now,
                ],
            ).unwrap();
        }

        tx.commit().unwrap();

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 5);
    }

    // 测试 10: TRANSACTION - 事务回滚测试
    #[test]
    fn test_transaction_rollback() {
        let conn = create_test_db();

        // 先插入一条正常记录
        insert_track(&conn, &TestTrack::new("Song 1", "Artist 1", "https://example.com/1.mp3"));

        // 开始事务，插入一条会失败的数据（故意使用相同的 ID）
        let tx = conn.transaction().unwrap();

        let track = TestTrack::new("Song 1", "Artist 1", "https://example.com/1.mp3"); // 相同的 ID

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // 这条插入会失败（因为主键冲突）
        let result = tx.execute(
            "INSERT INTO tracks (id, title, artist, cover_url, duration, src, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &track.id,
                &track.title,
                &track.artist,
                &track.cover_url,
                &track.duration,
                &track.src,
                now,
                now,
            ],
        );

        // 如果失败，回滚事务
        if result.is_err() {
            tx.rollback().unwrap();
        } else {
            tx.commit().unwrap();
        }

        // 验证只有一条记录
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    // 测试 11: LIMIT - 分页查询
    #[test]
    fn test_pagination() {
        let conn = create_test_db();

        for i in 1..=10 {
            insert_track(&conn, &TestTrack::new(
                &format!("Song {}", i),
                &format!("Artist {}", i),
                &format!("https://example.com/{}.mp3", i),
            ));
        }

        // 查询第一页（每页 3 条）
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, cover_url, duration, src FROM tracks ORDER BY created_at LIMIT 3 OFFSET 0"
        ).unwrap();

        let page1: Vec<TestTrack> = stmt.query_map([], |row| {
            Ok(TestTrack {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                cover_url: row.get(3)?,
                duration: row.get(4)?,
                src: row.get(5)?,
            })
        }).unwrap().filter_map(|t| t.ok()).collect();

        assert_eq!(page1.len(), 3);
        assert_eq!(page1[0].title, "Song 1");

        // 查询第二页
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, cover_url, duration, src FROM tracks ORDER BY created_at LIMIT 3 OFFSET 3"
        ).unwrap();

        let page2: Vec<TestTrack> = stmt.query_map([], |row| {
            Ok(TestTrack {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                cover_url: row.get(3)?,
                duration: row.get(4)?,
                src: row.get(5)?,
            })
        }).unwrap().filter_map(|t| t.ok()).collect();

        assert_eq!(page2.len(), 3);
        assert_eq!(page2[0].title, "Song 4");
    }

    // 测试 12: AGGREGATION - 聚合查询
    #[test]
    fn test_aggregation() {
        let conn = create_test_db();

        insert_track(&conn, &TestTrack::new("Song 1", "Artist 1", "https://example.com/1.mp3"));
        insert_track(&conn, &TestTrack::new("Song 2", "Artist 2", "https://example.com/2.mp3"));
        insert_track(&conn, &TestTrack::new("Song 3", "Artist 1", "https://example.com/3.mp3"));

        // 统计总数
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 3);

        // 统计总时长
        let total_duration: f64 = conn.query_row("SELECT SUM(duration) FROM tracks", [], |row| row.get(0)).unwrap();
        assert_eq!(total_duration, 540.0); // 3 * 180

        // 统计平均时长
        let avg_duration: f64 = conn.query_row("SELECT AVG(duration) FROM tracks", [], |row| row.get(0)).unwrap();
        assert_eq!(avg_duration, 180.0);

        // 统计不同艺术家的数量
        let artist_count: i64 = conn.query_row("SELECT COUNT(DISTINCT artist) FROM tracks", [], |row| row.get(0)).unwrap();
        assert_eq!(artist_count, 2);
    }

    // 测试 13: NULL 值处理
    #[test]
    fn test_null_values() {
        let conn = create_test_db();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // 插入带有 NULL 值的记录
        conn.execute(
            "INSERT INTO tracks (id, title, artist, cover_url, duration, src, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                get_uuid_from_url("https://example.com/test.mp3").to_string(),
                "Test Song",
                "Test Artist",
                Option::<String>::None,  // NULL cover_url
                180.0,
                "https://example.com/test.mp3",
                now,
                now,
            ],
        ).unwrap();

        // 查询 NULL 值
        let mut stmt = conn.prepare(
            "SELECT id, title, artist, cover_url, duration, src FROM tracks WHERE cover_url IS NULL"
        ).unwrap();

        let tracks: Vec<TestTrack> = stmt.query_map([], |row| {
            Ok(TestTrack {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                cover_url: row.get::<Option<String>>(3).unwrap_or_default(),
                duration: row.get(4)?,
                src: row.get(5)?,
            })
        }).unwrap().filter_map(|t| t.ok()).collect();

        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].cover_url, "");
    }

    // 测试 14: 唯一性约束测试
    #[test]
    fn test_unique_constraint() {
        let conn = create_test_db();
        let track = TestTrack::new("Test Song", "Test Artist", "https://example.com/test.mp3");

        insert_track(&conn, &track);

        // 尝试插入相同 ID 的记录（应该失败）
        let result = conn.execute(
            "INSERT INTO tracks (id, title, artist, cover_url, duration, src, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &track.id,
                "Another Song",
                "Another Artist",
                "https://example.com/another.jpg",
                200.0,
                "https://example.com/another.mp3",
                1234567890,
                1234567890,
            ],
        );

        assert!(result.is_err());

        // 验证仍然只有一条记录
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    // 测试 15: 时间戳更新
    #[test]
    fn test_timestamp_update() {
        let conn = create_test_db();
        let track = TestTrack::new("Test Song", "Test Artist", "https://example.com/test.mp3");

        insert_track(&conn, &track);

        // 获取初始的 updated_at
        let initial_updated: i64 = conn.query_row(
            "SELECT updated_at FROM tracks WHERE id = ?1",
            params![&track.id],
            |row| row.get(0),
        ).unwrap();

        // 等待一小段时间
        std::thread::sleep(std::time::Duration::from_millis(10));

        // 更新记录
        let new_updated = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "UPDATE tracks SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params!["Updated Song", new_updated, &track.id],
        ).unwrap();

        // 验证 updated_at 已更新
        let final_updated: i64 = conn.query_row(
            "SELECT updated_at FROM tracks WHERE id = ?1",
            params![&track.id],
            |row| row.get(0),
        ).unwrap();

        assert!(final_updated > initial_updated);
    }
}
*/
