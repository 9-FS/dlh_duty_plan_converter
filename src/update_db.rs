// Copyright (c) 2025 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use icalendar::Component;
use icalendar::EventLike;
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
/// - `db`: database connection
///
/// # Returns
/// - nothing or error
pub async fn update_airports(http_client: &reqwest::Client, airport_data_url: &str, db: &sqlx::sqlite::SqlitePool) -> Result<(), UpdateAirportsError>
{
    const AIRPORT_QUERY_STRING: &str = "INSERT OR REPLACE INTO Airport (id, ident, type, name, latitude_deg, longitude_deg, elevation_ft, continent, iso_country, iso_region, municipality, scheduled_service, gps_code, iata_code, local_code, home_link, wikipedia_link, keywords) "; // query string for Airport table
    const BINDS_PER_QUERY: usize = 32766; // maximum number of binds per query in sqlite, for chunking
    let mut airports: Vec<AirportDownloadResponse> = std::vec::Vec::new(); // all airports
    let mut db_tx: sqlx::Transaction<'_, sqlx::Sqlite>; // database transaction so automatic rollback on error
    let f: scaler::Formatter = scaler::Formatter::new().set_rounding(scaler::Rounding::Magnitude(0)).set_scaling(scaler::Scaling::None); // formatter for logging


    let r = http_client.get(airport_data_url).send().await?; // download airport data
    log::debug!("{}", r.status());
    log::info!("Downloaded airport data from \"{airport_data_url}\".");

    for (i, row) in csv::Reader::from_reader(r.text().await?.as_bytes()).deserialize::<AirportDownloadResponse>().enumerate() // parse csv
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
    let mut rows_affected: u64 = 0; // number of rows affected
    db_tx = db.begin().await?; // start transaction
    for chunk in airports.chunks(BINDS_PER_QUERY / 18) // chunk airports into maximum number of binds per query
    {
        let mut query = sqlx::query_builder::QueryBuilder::new(AIRPORT_QUERY_STRING);
        query.push_values(chunk, |mut builder, airport|
        {
            builder
                .push_bind(&airport.id)
                .push_bind(&airport.ident)
                .push_bind(format!("{:?}", airport.r#type))
                .push_bind(&airport.name)
                .push_bind(&airport.latitude_deg)
                .push_bind(&airport.longitude_deg)
                .push_bind(&airport.elevation_ft)
                .push_bind(format!("{:?}", airport.continent))
                .push_bind(&airport.iso_country)
                .push_bind(&airport.iso_region)
                .push_bind(&airport.municipality)
                .push_bind(&airport.scheduled_service)
                .push_bind(&airport.gps_code)
                .push_bind(&airport.iata_code)
                .push_bind(&airport.local_code)
                .push_bind(&airport.home_link)
                .push_bind(&airport.wikipedia_link)
                .push_bind(&airport.keywords);
        });
        rows_affected += query
            .build()
            .persistent(false)
            .execute(&mut *db_tx).await? // execute query
            .rows_affected(); // get number of rows affected
    }
    db_tx.commit().await?; // commit transaction
    log::info!("Updated airport database. Rows affected: {}", f.format(rows_affected as f64));

    return Ok(());
}


/// # Summary
/// Downloads country data from "ourairports.com/data/countries.csv", parses it, and updates the database table "Country".
///
/// # Arguments
/// - `http_client`: http client
/// - `country_data_url`: country data source URL
/// - `db`: database connection
///
/// # Returns
/// - nothing or error
pub async fn update_countries(http_client: &reqwest::Client, country_data_url: &str, db: &sqlx::sqlite::SqlitePool) -> Result<(), UpdateCountriesError>
{
    const COUNTRY_QUERY_STRING: &str = "INSERT OR REPLACE INTO Country (id, code, name, continent, wikipedia_link, keywords) "; // query string for Country table
    let mut countries: Vec<CountryDownloadResponse> = std::vec::Vec::new(); // all countries
    let mut db_tx: sqlx::Transaction<'_, sqlx::Sqlite>; // database transaction so automatic rollback on error
    let f: scaler::Formatter = scaler::Formatter::new().set_rounding(scaler::Rounding::Magnitude(0)).set_scaling(scaler::Scaling::None); // formatter for logging


    let r = http_client.get(country_data_url).send().await?; // download country data
    log::debug!("{}", r.status());
    log::info!("Downloaded country data from \"{country_data_url}\".");
    for (i, row) in csv::Reader::from_reader(r.text().await?.as_bytes()).deserialize::<CountryDownloadResponse>().enumerate() // parse csv
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
    db_tx = db.begin().await?; // start transaction
    let mut query = sqlx::query_builder::QueryBuilder::new(COUNTRY_QUERY_STRING);
    query.push_values(countries, |mut builder, country|
    {
        builder
            .push_bind(country.id)
            .push_bind(country.code)
            .push_bind(country.name)
            .push_bind(format!("{:?}", country.continent))
            .push_bind(country.wikipedia_link)
            .push_bind(country.keywords);
    });
    let rows_affected: u64 = query
        .build()
        .persistent(false)
        .execute(&mut *db_tx).await? // execute query
        .rows_affected(); // get number of rows affected
    db_tx.commit().await?; // commit transaction
    log::info!("Updated country database. Rows affected: {}", f.format(rows_affected as f64));

    return Ok(());
}


/// # Summary
/// Downloads calendar from myTime, parses it, and updates the database table "Event". Events that have ended 1 week ago (604,8 ks) or prior are considered archived and remain untouched. Events newer than that are considered active and are deleted from the database and then replaced by the downloaded data. Exception is if event database is still empty, then all downloaded events are inserted.
///
/// # Arguments
/// - `http_client`: http client
/// - `input_calendar_url`: calendar source URL
/// - `db`: database connection
/// - `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - nothing or error
pub async fn update_events(http_client: &reqwest::Client, input_calendar_url: &str, db: &sqlx::sqlite::SqlitePool, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> Result<(), UpdateEventsError>
{
    const BINDS_PER_QUERY: usize = 32766; // maximum number of binds per query in sqlite, for chunking
    const EVENT_QUERY_STRING: [&str; 3] = // query string for Event table
    [
        "SELECT * FROM Event;", // check if table is empty or not
        "DELETE FROM Event WHERE ? < end_dt;", // delete all active events, meaning events newer than 1 week ago
        "INSERT OR REPLACE INTO Event (uid, summary, start_dt, end_dt, location, description) " // insert new events
    ];
    let mut db_tx: sqlx::Transaction<'_, sqlx::Sqlite>; // database transaction so automatic rollback on error
    let event_db_empty: bool; // check if event database is empty
    let f: scaler::Formatter = scaler::Formatter::new().set_rounding(scaler::Rounding::Magnitude(0)).set_scaling(scaler::Scaling::None); // formatter for logging
    let input_calendar: icalendar::Calendar; // input calendar


    let r = http_client.get(input_calendar_url).send().await?; // download calendar ics
    log::debug!("{}", r.status());
    input_calendar = r.text().await?.parse()?; // parse calendar ics
    log::info!("Downloaded and parsed calendar from \"{input_calendar_url}\"."); // log download
    log::debug!("{input_calendar}");


    log::info!("Updating event database...");
    let mut rows_affected: u64 = 0; // number of rows affected
    db_tx = db.begin().await?; // start transaction

    let mut query = sqlx::query_builder::QueryBuilder::new(EVENT_QUERY_STRING[0]); // check if table is empty
    match query
        .build()
        .persistent(false)
        .fetch_optional(&mut *db_tx).await? // execute query
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
        query = sqlx::query_builder::QueryBuilder::new(EVENT_QUERY_STRING[1]);
        rows_affected += query
            .build()
            .bind(archive_end_dt) // bind archive end datetime to query
            .persistent(false)
            .execute(&mut *db_tx).await? // execute query
            .rows_affected(); // get number of rows affected
        log::debug!("Deleted all active events from event database. Rows affected: {}", f.format(rows_affected as f64));
    }

    rows_affected = 0; // reset rows affected
    for chunk in input_calendar.chunks(BINDS_PER_QUERY / 6) // chunk events into maximum number of binds per query
    {
        let mut events_to_insert: Vec<EventRow> = Vec::new(); // events to insert in database later, filtered and transformed

        for event in chunk.iter().filter_map(|component| component.as_event().or_else(|| {log::warn!("Component \"{:?}\" is not an event. Discarding component.", component); None})) // filter out all components that are not events
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
            if !event_db_empty && is_archived(end_str.as_str(),  &(chrono::Utc::now() - chrono::Duration::weeks(1))) // if table is not empty and event is archived: do not insert
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

        let mut query = sqlx::query_builder::QueryBuilder::new(EVENT_QUERY_STRING[2]);
        query.push_values(events_to_insert, |mut builder, event_to_insert|
        {
            builder
                .push_bind(event_to_insert.uid) // UID -> Event.uid
                .push_bind(event_to_insert.summary) // SUMMARY -> Event.summary
                .push_bind(event_to_insert.start_str) // DTSTART -> Event.start_dt
                .push_bind(event_to_insert.end_str) // DTEND -> Event.end_dt
                .push_bind(event_to_insert.location) // LOCATION -> Event.location
                .push_bind(event_to_insert.description); // DESCRIPTION -> Event.description
        });
        rows_affected += query
            .build()
            .persistent(false)
            .execute(&mut *db_tx).await? // execute query
            .rows_affected(); // get number of rows affected
    }
    db_tx.commit().await?; // commit transaction
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