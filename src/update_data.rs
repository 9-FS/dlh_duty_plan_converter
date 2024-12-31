// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use futures::StreamExt;
use crate::api_response::*;
use crate::connect_to_db::*;
use crate::error::*;


/// # Summary
/// Downloads airport data from "ourairports.com/data/airports.csv", parses it, and updates the database table "Airport".
///
/// # Arguments
/// - `airport_data_url`: airport data source URL
/// - `db_url`: database URL
///
/// # Returns
/// - nothing or error
pub async fn update_airport_data(airport_data_url: &str, db_url: &str) -> Result<(), UpdateAirportDataError>
{
    const AIRPORT_QUERY_STRING: &str = "INSERT OR REPLACE INTO Airport (id, ident, type, name, latitude_deg, longitude_deg, elevation_ft, continent, iso_country, iso_region, municipality, scheduled_service, gps_code, iata_code, local_code, home_link, wikipedia_link, keywords) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);"; // query string for Airport table
    let mut airports: Vec<AirportDownloadResponse> = std::vec::Vec::new(); // all airports
    let db: sqlx::sqlite::SqlitePool; // database containing all airport data


    let r = reqwest::get(airport_data_url).await?.text().await?; // download airport data
    log::info!("Downloaded airport data from \"{airport_data_url}\".");
    for (i, row) in csv::Reader::from_reader(r.as_bytes()).deserialize::<AirportDownloadResponse>().enumerate() // parse csv
    {
        match row // parsed row successfully?
        {
            Ok(o) => airports.push(o.clone()),  // parsed successfully: add airport to list
            Err(e) => log::warn!("Parsing airport data from csv row {} failed with: {e}", i+1), // parsing failed: log warning
        }
    }
    log::debug!("Parsed {} airports.", airports.len());


    db = connect_to_db(db_url).await?; // connect to database
    let db_ref: &sqlx::Pool<sqlx::Sqlite> = &db; // reference to database for async closure
    futures::stream::iter(airports).for_each_concurrent(2, |airport| async move
    {
        let mut query: sqlx::query::Query<'_, _, _>; // query to insert all airports into database

        query = sqlx::query(AIRPORT_QUERY_STRING); // create query
        query = query
            .bind(airport.id)
            .bind(airport.ident)
            .bind(format!("{:?}", airport.r#type))
            .bind(airport.name)
            .bind(airport.latitude_deg)
            .bind(airport.longitude_deg)
            .bind(airport.elevation_ft)
            .bind(format!("{:?}", airport.continent))
            .bind(airport.iso_country)
            .bind(airport.iso_region)
            .bind(airport.municipality)
            .bind(airport.scheduled_service)
            .bind(airport.gps_code)
            .bind(airport.iata_code)
            .bind(airport.local_code)
            .bind(airport.home_link)
            .bind(airport.wikipedia_link)
            .bind(airport.keywords);
        query.execute(db_ref).await.expect("TODO"); // execute query
    }).await;
    db.close().await; // close database connection
    log::info!("Updated airport database.");

    return Ok(());
}


/// # Summary
/// Downloads country data from "ourairports.com/data/countries.csv", parses it, and updates the database table "Country".
///
/// # Arguments
/// - `country_data_url`: country data source URL
/// - `db_url`: database URL
///
/// # Returns
/// - nothing or error
pub async fn update_country_data(country_data_url: &str, db_url: &str) -> Result<(), UpdateCountryDataError>
{
    const COUNTRY_QUERY_STRING: &str = "INSERT OR REPLACE INTO Country (id, code, name, continent, wikipedia_link, keywords) VALUES (?, ?, ?, ?, ?, ?);"; // query string for Country table
    let mut countries: Vec<CountryDownloadResponse> = std::vec::Vec::new(); // all countries
    let db: sqlx::sqlite::SqlitePool; // database containing all country data


    let r = reqwest::get(country_data_url).await?.text().await?; // download country data
    log::info!("Downloaded country data from \"{country_data_url}\".");
    for (i, row) in csv::Reader::from_reader(r.as_bytes()).deserialize::<CountryDownloadResponse>().enumerate() // parse csv
    {
        match row // parsed row successfully?
        {
            Ok(o) => countries.push(o.clone()),  // parsed successfully: add country to list
            Err(e) => log::warn!("Parsing country data from csv row {} failed with: {e}", i+1), // parsing failed: log warning
        }
    }
    log::debug!("Parsed {} countries.", countries.len());


    db = connect_to_db(db_url).await?; // connect to database
    let db_ref: &sqlx::Pool<sqlx::Sqlite> = &db; // reference to database for async closure
    futures::stream::iter(countries).for_each_concurrent(2, |country| async move
    {
        let mut query: sqlx::query::Query<'_, _, _>; // query to insert all countries into database

        query = sqlx::query(COUNTRY_QUERY_STRING); // create query
        query = query
            .bind(country.id)
            .bind(country.code)
            .bind(country.name)
            .bind(format!("{:?}", country.continent))
            .bind(country.wikipedia_link)
            .bind(country.keywords);
        query.execute(db_ref).await.expect("TODO"); // execute query
    }).await;
    db.close().await; // close database connection
    log::info!("Updated country database.");

    return Ok(());
}