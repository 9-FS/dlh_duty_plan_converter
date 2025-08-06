// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use crate::config::*;
use crate::connect_to_db::*;
use crate::error::*;
use crate::update_calendar::*;
use crate::update_db::*;


pub fn main_inner(config: Config) -> Result<(), Error>
{
    const AIRPORT_DATA_URL: &str = "https://ourairports.com/data/airports.csv"; // airport data online
    const COUNTRY_DATA_URL: &str = "https://ourairports.com/data/countries.csv"; // country data online
    const DB_FILEPATH: &str = "./db/db.sqlite"; // database filepath
    const HTTP_TIMEOUT: u64 = 10; // connection timeout
    let http_client: reqwest::blocking::Client; // http client


    http_client = reqwest::blocking::Client::builder()  // create http client
        .danger_accept_invalid_certs(true) // accept invalid certificates from ourairports.com
        .timeout(Some(std::time::Duration::from_secs(HTTP_TIMEOUT)))
        .build()?;


    match connect_to_db(DB_FILEPATH) // connect to database
    {
        Ok(mut db) =>
        {
            if let Err(e) = update_airports(&http_client, AIRPORT_DATA_URL, &mut db) // download airport data, parse csv, update database
            {
                log::warn!("Updating airport database failed with: {e}\nContinuing with potentially outdated data.");
            }
            if let Err(e) = update_countries(&http_client, COUNTRY_DATA_URL, &mut db) // download country data, parse csv, update database
            {
                log::warn!("Updating country database failed with: {e}\nContinuing with potentially outdated data.");
            }
        }
        Err(e) => log::error!("{e}"),
    }


    loop
    {
        'iteration:
        {
            let mut db: rusqlite::Connection; // database containing all airport data
            let archive_end_dt: chrono::DateTime<chrono::Utc> = chrono::Utc::now() + config.ARCHIVE_END_RELATIVE; // when archive ends, read clock once to have clear reference point for archiving per iteration


            log::info!("--------------------------------------------------");

            match connect_to_db(DB_FILEPATH) // connect to database
            {
                Ok(o) => db = o,
                Err(e) =>
                {
                    log::error!("{e}");
                    break 'iteration; // abort iteration, go straight to sleeping
                }
            }


            if let Err(e) = update_calendar(&http_client, config.INPUT_CALENDAR_URL.as_str(), config.OUTPUT_CALENDAR_FILEPATH.as_str(), &mut db, &archive_end_dt) // update calendar iteration
            {
                log::error!("Updating calendar failed with: {e}"); // log error
            }
        } // free as much memory as possible

        std::thread::sleep(std::time::Duration::from_secs(config.SLEEP_INTERVAL)); // sleep between updates
    }
}