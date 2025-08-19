// Copyright (c) 2025 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use icalendar::Component;
use icalendar::EventLike;
use rusqlite::OptionalExtension;
use crate::api_response::*;
use crate::dateperhapstime_to_string::*;
use crate::error::*;
use crate::is_archived::*;


/// # Summary
/// Downloads airport data from "ourairports.com/data/airports.csv", parses it, and updates the database table "Airport".
///
/// # Arguments
/// - `http_client`: http client
/// - `airport_data_url`: airport data source URL
/// - `db`: database connection pool
///
/// # Returns
/// - nothing or error
pub fn update_airports(http_client: &reqwest::blocking::Client, airport_data_url: &str, db: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>) -> Result<(), UpdateAirportsError>
{
    const AIRPORT_QUERY: &str = "INSERT OR REPLACE INTO Airport (id, ident, type, name, latitude_deg, longitude_deg, elevation_ft, continent, iso_country, iso_region, municipality, scheduled_service, gps_code, iata_code, local_code, home_link, wikipedia_link, keywords) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);"; // query string for Airport table
    let mut airports: Vec<AirportDownloadResponse> = std::vec::Vec::new(); // all airports
    let f: scaler::Formatter = scaler::Formatter::new().set_rounding(scaler::Rounding::Magnitude(0)).set_scaling(scaler::Scaling::None); // formatter for logging


    let r = http_client.get(airport_data_url).send()?; // download airport data
    log::debug!("{}", r.status());
    log::info!("Downloaded airport data from \"{airport_data_url}\".");

    for (i, row) in csv::Reader::from_reader(r.text()?.as_bytes()).deserialize::<AirportDownloadResponse>().enumerate() // parse csv
    {
        match row // parsed row successfully?
        {
            Ok(o) => airports.push(o.clone()),  // parsed successfully: add airport to list
            Err(e) => log::warn!("Parsing airport data from csv row {} failed with: {e}", i+1), // parsing failed: log warning
        }
    }
    log::debug!("Parsed {} airports.", f.format(airports.len() as f64));
    if airports.len() == 0 // no airports found
    {
        log::warn!("Downloaded data does not contain any airports. Skipping update.");
        return Ok(());
    }


    log::info!("Updating airport database...");
    let mut rows_affected = 0; // number of rows affected
    let mut db_con = db.get()?; // get connection
    let db_tx = db_con.transaction()?; // start transaction so automatic rollback on error
    {
        let mut db_stmt = db_tx.prepare(AIRPORT_QUERY)?; // prepare bulk insert
        for airport in airports
        {
            rows_affected += db_stmt.execute(rusqlite::params! // bind parameters, count rows affected
            [
                airport.id,
                airport.ident,
                format!("{:?}", airport.r#type),
                airport.name,
                airport.latitude_deg,
                airport.longitude_deg,
                airport.elevation_ft,
                format!("{:?}", airport.continent),
                airport.iso_country,
                airport.iso_region,
                airport.municipality,
                airport.scheduled_service,
                airport.gps_code,
                airport.iata_code,
                airport.local_code,
                airport.home_link,
                airport.wikipedia_link,
                airport.keywords
            ])?;
        }
    }
    db_tx.commit()?; // commit transaction
    log::info!("Updated airport database. Rows affected: {}", f.format(rows_affected as f64));

    return Ok(());
}


/// # Summary
/// Downloads country data from "ourairports.com/data/countries.csv", parses it, and updates the database table "Country".
///
/// # Arguments
/// - `http_client`: http client
/// - `country_data_url`: country data source URL
/// - `db`: database connection pool
///
/// # Returns
/// - nothing or error
pub fn update_countries(http_client: &reqwest::blocking::Client, country_data_url: &str, db: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>) -> Result<(), UpdateCountriesError>
{
    const COUNTRY_QUERY: &str = "INSERT OR REPLACE INTO Country (id, code, name, continent, wikipedia_link, keywords) VALUES (?, ?, ?, ?, ?, ?);"; // query string for Country table
    let mut countries: Vec<CountryDownloadResponse> = std::vec::Vec::new(); // all countries
    let f: scaler::Formatter = scaler::Formatter::new().set_rounding(scaler::Rounding::Magnitude(0)).set_scaling(scaler::Scaling::None); // formatter for logging


    let r = http_client.get(country_data_url).send()?; // download country data
    log::debug!("{}", r.status());
    log::info!("Downloaded country data from \"{country_data_url}\".");
    for (i, row) in csv::Reader::from_reader(r.text()?.as_bytes()).deserialize::<CountryDownloadResponse>().enumerate() // parse csv
    {
        match row // parsed row successfully?
        {
            Ok(o) => countries.push(o.clone()),  // parsed successfully: add country to list
            Err(e) => log::warn!("Parsing country data from csv row {} failed with: {e}", i+1), // parsing failed: log warning
        }
    }
    log::debug!("Parsed {} countries.", f.format(countries.len() as f64));
    if countries.len() == 0 // no countries found
    {
        log::warn!("Downloaded data does not contain any countries. Skipping update.");
        return Ok(());
    }

    log::info!("Updating country database...");
    let mut rows_affected: usize = 0; // number of rows affected
    let mut db_con = db.get()?; // get connection
    let db_tx = db_con.transaction()?; // start transaction so automatic rollback on error
    {
        let mut db_stmt = db_tx.prepare(COUNTRY_QUERY)?; // prepare bulk insert
        for country in countries
        {
            rows_affected += db_stmt.execute( // bind parameters, count rows affected
            (
                country.id,
                country.code,
                country.name,
                format!("{:?}", country.continent),
                country.wikipedia_link,
                country.keywords
            ))?;
        }
    }
    db_tx.commit()?; // commit transaction
    log::info!("Updated country database. Rows affected: {}", f.format(rows_affected as f64));

    return Ok(());
}


/// # Summary
/// Downloads calendar from myTime, parses it, and updates the database table "Event". Events that have ended at `archive_end_dt` or prior are considered archived and remain untouched. Events newer than that are considered active and are deleted from the database and then replaced by the downloaded data. Exception is if event database is still empty, then all downloaded events are inserted.
///
/// # Arguments
/// - `http_client`: http client
/// - `input_calendar_url`: calendar source URL
/// - `db`: database connection pool
/// - `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - nothing or error
pub fn update_events(http_client: &reqwest::blocking::Client, input_calendar_url: &str, db: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> Result<(), UpdateEventsError>
{
    const EVENT_QUERY: [&str; 3] = // query string for Event table
    [
        "SELECT * FROM Event;", // check if table is empty or not
        "DELETE FROM Event WHERE ? < end_dt;", // delete all active events, meaning events newer than end of archive
        "INSERT OR REPLACE INTO Event (uid, summary, start_dt, end_dt, location, description) VALUES (?, ?, ?, ?, ?, ?);" // insert new events
    ];
    let event_db_empty: bool; // check if event database is empty
    let f: scaler::Formatter = scaler::Formatter::new().set_rounding(scaler::Rounding::Magnitude(0)).set_scaling(scaler::Scaling::None); // formatter for logging
    let input_calendar: icalendar::Calendar; // input calendar


    let r = http_client.get(input_calendar_url).send()?; // download calendar ics
    log::debug!("{}", r.status());
    input_calendar = r.text()?.parse()?; // parse calendar ics
    log::info!("Downloaded and parsed calendar from \"{input_calendar_url}\"."); // log download
    log::debug!("{input_calendar}");


    log::info!("Updating event database...");
    let mut rows_affected;
    let mut db_con = db.get()?; // get connection
    let db_tx = db_con.transaction()?; // start transaction so automatic rollback on error
    {
        match db_tx.query_row(EVENT_QUERY[0], (), |_| Ok(())).optional()? // check if table is empty
        {
            Some(_) => // table is not empty
            {
                log::debug!("Event database is not empty. Only deleting and inserting active events.");
                event_db_empty = false;
            },
            None => // table is empty
            {
                log::debug!("Event database is empty. Inserting all events.");
                event_db_empty = true;
            },
        }

        if !event_db_empty // if table not empty: delete all active events before inserting new ones
        {
            rows_affected = db_tx.execute(EVENT_QUERY[1], (archive_end_dt.to_rfc3339(),))?; // delete all active events, meaning events newer than archive_end_dt, must convert to iso8601 because it does not contain space and default trait conversion contains space which is apparently not properly escaped in rusqlite
            log::debug!("Deleted all active events from event database. Rows affected: {}", f.format(rows_affected as f64));
        }


        rows_affected = 0; // reset rows affected
        let mut db_stmt = db_tx.prepare(EVENT_QUERY[2])?; // prepare bulk insert
        let mut events_to_insert: Vec<EventRow> = Vec::new(); // events to insert in database later, filtered and transformed
        for event in input_calendar.iter().filter_map(|component| component.as_event().or_else(|| {log::warn!("Component \"{:?}\" is not an event. Discarding component.", component); None})) // filter out all components that are not events
        {
            let end_str: String;
            let start_str: String;
            let uid_str: String;

            // filter out all events not to be inserted, make rest ready for insertion
            match event.get_uid() // UID
            {
                Some(s) => uid_str = s.to_owned(), // convert to string
                None => // if no uid: discard
                {
                    log::error!("Event \"{}\" has no UID. Discarding event.", event.get_summary().unwrap_or_default());
                    continue;
                },
            }
            match event.get_start() // DTSTART
            {
                Some(dt) =>
                {
                    match dateperhapstime_to_string(dt) // convert to string
                    {
                        Ok(dt) => start_str = dt,
                        Err(e) => // if invalid datetime: discard
                        {
                            log::error!("{e}");
                            continue;
                        }
                    }
                },
                None => // if no start date: discard
                {
                    log::error!("Event {} \"{}\"has no start datetime. Discarding event.", event.get_uid().unwrap_or_default(), event.get_summary().unwrap_or_default());
                    continue;
                },
            }
            match event.get_end() // DTEND
            {
                Some(dt) =>
                {
                    match dateperhapstime_to_string(dt) // convert to string
                    {
                        Ok(dt) => end_str = dt,
                        Err(e) => // if invalid datetime: discard
                        {
                            log::error!("{e}");
                            continue;
                        }
                    }

                },
                None => // if no end date: discard
                {
                    log::error!("Event {} \"{}\"has no end datetime. Discarding event.", event.get_uid().unwrap_or_default(), event.get_summary().unwrap_or_default());
                    continue;
                },
            }
            if !event_db_empty && is_archived(end_str.as_str(), archive_end_dt) // if table is not empty and event is archived: do not insert
                .expect(format!("Parsing \"{end_str}\" to datetime failed even though it should have been properly formatted in dateperhapstime_to_string.").as_str())
            {
                continue;
            }

            events_to_insert.push(EventRow
            {
                uid: uid_str,
                summary: event.get_summary().map(|s| s.to_owned()),
                start_str,
                end_str,
                location: event.get_location().map(|s| s.to_owned()),
                description: event.get_description().map(|s| s.to_owned()),
            });
        }

        for event_to_insert in events_to_insert
        {
            rows_affected += db_stmt.execute // bind parameters, count rows affected
            ((
                event_to_insert.uid,
                event_to_insert.summary,
                event_to_insert.start_str,
                event_to_insert.end_str,
                event_to_insert.location,
                event_to_insert.description
            ))?;
        }
    }
    db_tx.commit()?; // commit transaction
    log::info!("Updated event database. Rows affected: {}", f.format(rows_affected as f64));

    return Ok(());
}


#[derive(Debug, Clone, Eq, PartialEq,)]
pub struct EventRow
{
    pub uid: String,
    pub summary: Option<String>,
    pub start_str: String,
    pub end_str: String,
    pub location: Option<String>,
    pub description: Option<String>,
}