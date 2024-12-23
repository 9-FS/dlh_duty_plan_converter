// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use sqlx::ConnectOptions;
use sqlx::migrate::MigrateDatabase;


/// # Summary
/// Creates a new database or connects to an existing one at `database_url`, runs the instructions in `migrations_path`, and returns a connection pool.
///
/// # Arguments
/// - `database_url`: url to database file, might not be local but is recommended to be so
/// - `migrations_path`: path to directory containing migration files
///
/// # Returns
/// - connection pool to database or error
pub async fn connect_to_db(db_url: &str, db_migrations_path: &str) -> Result<sqlx::SqlitePool, sqlx::Error>
{
    let db: sqlx::SqlitePool; // database connection pool


    if !sqlx::Sqlite::database_exists(db_url).await? // if database does not exist
    {
        match std::path::Path::new(db_url).parent()
        {
            Some(parent) =>
            {
                if let Err(e) = tokio::fs::create_dir_all(parent).await // create all parent directories
                {
                    log::warn!("Creating parent directories for new database at \"{db_url}\" failed with {e}.\nThis could be expected behaviour, usually if this is a remote pointing URL and not a local filepath. In that case create the parent directories manually.");
                }
            }
            None => log::warn!("Creating parent directories for new database at \"{db_url}\", because the directory part could not be parsed.\nThis could be expected behaviour, usually if this is a remote pointing URL and not a local filepath. In that case create the parent directories manually."),
        }
        sqlx::Sqlite::create_database(db_url).await?; // create new database
        log::info!("Created new database at \"{db_url}\".");
    }

    db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1) // only 1 connection to database at the same time, otherwise concurrent writers fail
        .max_lifetime(None) // keep connection open indefinitely otherwise database locks up after lifetime, closing and reconnecting manually
        .connect(db_url).await?; // connect to database
    db.set_connect_options(sqlx::sqlite::SqliteConnectOptions::new()
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal) // use write-ahead journal for better performance
        .locking_mode(sqlx::sqlite::SqliteLockingMode::Exclusive) // do not release file lock until all transactions are complete
        .log_slow_statements(log::LevelFilter::Warn, std::time::Duration::from_secs(5)) // log slow statements only after 5 s
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)); // ensure data is written to disk after each transaction for consistent state
    log::info!("Connected to database at \"{db_url}\".");

    if std::path::Path::new(db_migrations_path).exists() // if migrations path exists
    {
        sqlx::migrate::Migrator::new(std::path::Path::new(db_migrations_path)).await?.run(&db).await?; // run migrations to create and update tables
        log::debug!("Executed migrations at \"{db_migrations_path}\".");
    }

    return Ok(db);
}