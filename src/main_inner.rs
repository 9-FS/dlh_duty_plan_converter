// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use crate::config::*;
use crate::connect_to_db::*;
use crate::error::*;
use crate::update_calendar::*;
use crate::update_data::*;


pub async fn main_inner(config: Config) -> Result<(), Error>
{
    const AIRPORT_DATA_URL: &str = "https://ourairports.com/data/airports.csv"; // airport data online
    const COUNTRY_DATA_URL: &str = "https://ourairports.com/data/countries.csv"; // country data online
    const DB_FILEPATH: &str = "./db/db.sqlite"; // database filepath


    if let Err(e) = update_airport_data(AIRPORT_DATA_URL, DB_FILEPATH).await // download airport data, parse csv, update database
    {
        log::warn!("Updating airport database failed with: {e}\nContinuing with potentially outdated data.");
    }
    if let Err(e) = update_country_data(COUNTRY_DATA_URL, DB_FILEPATH).await // download country data, parse csv, update database
    {
        log::warn!("Updating country database failed with: {e}\nContinuing with potentially outdated data.");
    }


    loop
    {
        'iteration:
        {
            let db: sqlx::sqlite::SqlitePool; // database containing all airport data


            log::info!("--------------------------------------------------");

            match connect_to_db(DB_FILEPATH).await // connect to database
            {
                Ok(o) => db = o,
                Err(e) =>
                {
                    log::error!("{e}");
                    break 'iteration; // abort iteration, go straight to sleeping
                }
            }


            if let Err(e) = update_calendar(config.INPUT_CALENDAR_URL.as_str(), config.OUTPUT_CALENDAR_FILEPATH.as_str(), &db).await // update calendar iteration
            {
                log::error!("Updating calendar failed with: {e}"); // log error
            }

            db.close().await; // close database connection
            log::info!("Disconnected from database at \"{}\".", DB_FILEPATH);
        } // free as much memory as possible

        std::thread::sleep(std::time::Duration::from_secs(config.SLEEP_INTERVAL)); // sleep between updates
    }
}