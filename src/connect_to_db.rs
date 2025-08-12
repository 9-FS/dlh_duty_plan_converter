// Copyright (c) 2025 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use crate::error::*;


/// # Summary
/// Creates a new database or connects to an existing one at `db_url`, runs the instructions in `migrations_path`, and returns a connection pool.
///
/// # Arguments
/// - `db_url`: url to database file, might not be local but is recommended to be so
///
/// # Returns
/// - database connection pool or error
pub fn connect_to_db(db_url: &str) -> Result<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>, ConnectToDbError>
{
    static MIGRATIONS_DIR: include_dir::Dir = include_dir::include_dir!("./db_migrations/");
    let migrations: rusqlite_migration::Migrations<'static> = rusqlite_migration::Migrations::from_directory(&MIGRATIONS_DIR).unwrap();
    let db: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>; // database connection


    if !std::fs::exists(db_url).unwrap_or(false) // if database does not exist
    {
        match std::path::Path::new(db_url).parent()
        {
            Some(parent) =>
            {
                if let Err(e) = std::fs::create_dir_all(parent) // create all parent directories
                {
                    log::warn!("Creating parent directories for new database at \"{db_url}\" failed with {e}.\nThis could be expected behaviour, usually if this is a remote pointing URL and not a local filepath. In that case create the parent directories manually.");
                }
            }
            None => log::warn!("Creating parent directories for new database at \"{db_url}\", because the directory part could not be parsed.\nThis could be expected behaviour, usually if this is a remote pointing URL and not a local filepath. In that case create the parent directories manually."),
        }
        db = r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::file(db_url))?; // create new database and connect to it
        log::info!("Created new database at \"{db_url}\".");
    }
    else
    {
        db = r2d2::Pool::new(r2d2_sqlite::SqliteConnectionManager::file(db_url))?; // connect to existing database
        log::info!("Connected to database at \"{db_url}\".");
    }


    let mut db_con = db.get()?;
    migrations.to_latest(&mut db_con)?; // run migrations to create and update tables
    log::debug!("Executed migrations at \"./db_migrations/\".");

    return Ok(db);
}