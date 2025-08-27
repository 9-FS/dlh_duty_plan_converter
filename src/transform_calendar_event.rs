// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use icalendar::{Component, EventLike};
use crate::dateperhapstime_to_string::*;
use crate::is_archived::*;


/// # Summary
/// Transforms the briefing event. Additionally to the minimum actions changes summary to "Briefing", changes IATA location to ICAO location, and adds alarms at -1,5 h, -1 h, and -15 min.
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `db`: airport database connection pool
/// - `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - the transformed calendar event
pub fn transform_briefing(mut calendar_event: icalendar::Event, db: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event, archive_end_dt); // always do minimum before specific actions
    calendar_event.summary("Briefing");
    if let Some(row) = lookup_iata(calendar_event.get_location().unwrap_or_default().to_owned(), db) // if iata location found
    {
        if let Some(s) = row.airport_gps_code // if entry contains icao location
        {
            calendar_event.location(format!("{}: {}, {}", s, row.country_name, row.airport_name).as_str()); // change iata location to icao location
        }
    } // otherwise just keep original data
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::minutes(90))); // add alarm at -1,5 h
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::hours(-1))); // add alarm at -1 h
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::minutes(-15))); // add alarm at -15 min

    return calendar_event;
}


/// # Summary
/// Transforms the deadhead event. Additionally to the minimum actions changes summary format, changes IATA locations to departure ICAO location only, and adds an alarm at -1,5 h and -35 min.
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `flight_iata`: flight IATA code
/// - `departure_iata`: departure IATA code
/// - `destination_iata`: destination IATA code
/// - `db`: airport database connection pool
/// - `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - the transformed calendar event
pub fn transform_deadhead(mut calendar_event: icalendar::Event, flight_iata: String, departure_iata: String, destination_iata: String, db: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event, archive_end_dt); // always do minimum before specific actions
    calendar_event.summary(format!("DEADHEAD {flight_iata}: {} ✈ {}", try_iata_to_icao(departure_iata.to_owned(), db), try_iata_to_icao(destination_iata.to_owned(), db)).as_str()); // change summary format
    if let Some(row) = lookup_iata(departure_iata, db) // if iata location found
    {
        if let Some(s) = row.airport_gps_code // if entry contains icao location
        {
            calendar_event.location(format!("{}: {}, {}", s, row.country_name, row.airport_name).as_str()); // change iata location to icao location
        }
    } // otherwise just keep original data
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::minutes(90))); // add alarm at -1,5 h
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::minutes(-35))); // add alarm at -35 min

    return calendar_event;
}


/// # Summary
/// Transforms the flight event. Additionally to the minimum actions changes summary format, changes IATA locations to departure ICAO location only, and adds an alarm at -30 min.
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `flight_iata`: flight IATA code
/// - `departure_iata`: departure IATA code
/// - `destination_iata`: destination IATA code
/// - `db`: airport database connection pool
/// -- `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - the transformed calendar event
pub fn transform_flight(mut calendar_event: icalendar::Event, flight_iata: String, departure_iata: String, destination_iata: String, db: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event, archive_end_dt); // always do minimum before specific actions
    calendar_event.summary(format!("{flight_iata}: {} ✈ {}", try_iata_to_icao(departure_iata.to_owned(), db), try_iata_to_icao(destination_iata.to_owned(), db)).as_str()); // change summary format
    if let Some(row) = lookup_iata(departure_iata, db) // if iata location found
    {
        if let Some(s) = row.airport_gps_code // if entry contains icao location
        {
            calendar_event.location(format!("{}: {}, {}", s, row.country_name, row.airport_name).as_str()); // change iata location to icao location
        }
    } // otherwise just keep original data
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::minutes(-30))); // add alarm at -30 min

    return calendar_event;
}


/// # Summary
/// Transforms the ground event. Additionally to the minimum actions changes summary format, changes IATA locations to ICAO location only, and adds alarms at -1 h and -15 min.
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `category`: category of the event
/// - `description`: description of the event
/// - `db`: airport database connection pool
/// - `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - the transformed calendar event
pub fn transform_ground(mut calendar_event: icalendar::Event, category: String, description: String, db: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event, archive_end_dt); // always do minimum before specific actions
    if category == "" {calendar_event.summary(description.as_str());} // if category is empty: change summary to description
    else {calendar_event.summary(format!("{category}: {description}").as_str());} // otherwise: change summary format only slightly
    if let Some(row) = lookup_iata(calendar_event.get_location().unwrap_or_default().to_owned(), db) // if iata location found
    {
        calendar_event.location(format!("{}, {}", row.country_name, row.airport_municipality).as_str()); // change iata location to country and city
    } // otherwise just keep original data
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::hours(-1))); // add alarm at -1 h
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::minutes(-15))); // add alarm at -15 min

    return calendar_event;
}


/// # Summary
/// Transforms the holiday event. Additionally to the minimum actions changes summary to "Holiday".
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - the transformed calendar event
pub fn transform_holiday(mut calendar_event: icalendar::Event, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event, archive_end_dt); // always do minimum before specific actions
    calendar_event.location(""); // holiday does not need a location
    calendar_event.summary("Holiday");

    return calendar_event;
}


/// # Summary
/// Transforms the layover event. Additionally to the minimum actions changes summary to "Layover" and changes IATA location to ICAO location.
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `db`: airport database connection pool
/// - `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - the transformed calendar event
pub fn transform_layover(mut calendar_event: icalendar::Event, db: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event, archive_end_dt); // always do minimum before specific actions
    calendar_event.summary("Layover");
    if let Some(row) = lookup_iata(calendar_event.get_location().unwrap_or_default().to_owned(), db) // if iata location found
    {
        calendar_event.location(format!("{}, {}", row.country_name, row.airport_municipality).as_str()); // change iata location to country and city
    } // otherwise just keep original data

    return calendar_event;
}


/// # Summary
/// Transforms the off event. Additionally to the minimum actions changes summary to "Off".
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - the transformed calendar event
pub fn transform_off(mut calendar_event: icalendar::Event, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event, archive_end_dt); // always do minimum before specific actions
    calendar_event.location(""); // off day does not need a location
    calendar_event.summary("Off");

    return calendar_event;
}


/// # Summary
/// Transforms the pickup event. Additionally to the minimum actions changes summary to "Pickup", changes IATA location to ICAO location, and adds alarms at -1 h, -15 min, and -1 min.
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `db`: airport database connection pool
/// - `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - the transformed calendar event
pub fn transform_pickup(mut calendar_event: icalendar::Event, db: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event, archive_end_dt); // always do minimum before specific actions
    calendar_event.summary("Pickup");
    if let Some(row) = lookup_iata(calendar_event.get_location().unwrap_or_default().to_owned(), db) // if iata location found
    {
        calendar_event.location(format!("{}, {}", row.country_name, row.airport_municipality).as_str()); // change iata location to country and city
    } // otherwise just keep original data
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::hours(-1))); // add alarm at -1 h
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::minutes(-15))); // add alarm at -15 min
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::minutes(-1))); // add alarm at -1 min

    return calendar_event;
}


/// # Summary
/// Transforms the reserve event. Additionally to the minimum actions changes summary format, changes IATA locations to ICAO location only, and adds alarms at -15 min.
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `description`: description of the event
/// - `db`: airport database connection pool
/// - `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - the transformed calendar event
pub fn transform_reserve(mut calendar_event: icalendar::Event, description: String, db: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event, archive_end_dt); // always do minimum before specific actions
    match description.as_str() // change summary format
    {
        "REP" => {calendar_event.summary("Reserve Pattern");},
        "RES" => {calendar_event.summary("Reserve Standby");},
        "SB" => {calendar_event.summary("Standby");},
        _ => {panic!("Reserve event's description has invalid value \"{description}\" even though `RESERVE_PATTERN` should prevent this.");},
    }

    if let Some(row) = lookup_iata(calendar_event.get_location().unwrap_or_default().to_owned(), db) // if iata location found
    {
        calendar_event.location(format!("{}, {}", row.country_name, row.airport_municipality).as_str()); // change iata location to country and city
    } // otherwise just keep original data
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::minutes(-15))); // add alarm at -15 min

    return calendar_event;
}


/// # Summary
/// Transforms a sick day. Additionally to the minimum actions changes summary to "Sickness".
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - the transformed calendar event
pub fn transform_sickness(mut calendar_event: icalendar::Event, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event, archive_end_dt); // always do minimum before specific actions
    calendar_event.location(""); // sickness does not need a location
    calendar_event.summary("Sickness");

    return calendar_event;
}


/// # Summary
/// Transforms an unknown event. Only does the minimum: removes the unnecessary description and checks if the event is archived.
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - the transformed calendar event
pub fn transform_unknown(mut calendar_event: icalendar::Event, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> icalendar::Event
{
    calendar_event.description(""); // remove unnecessary description from mytime

    match dateperhapstime_to_string(calendar_event.get_end().expect(format!("Calendar event {} \"{}\" has no end datetime even though it is mandatory upon saving in the database.", calendar_event.get_uid().unwrap_or_default(), calendar_event.get_summary().unwrap_or_default()).as_str()))
    {
        Ok(o) =>
        {
            if is_archived(o.as_str(),  archive_end_dt) // if table is not empty and event is archived: do not insert
                .expect(format!("Parsing \"{o}\" to datetime failed even though it should have been properly formatted in dateperhapstime_to_string.").as_str())
            {
                calendar_event.description("archived event 🔒"); // if event is archived: state in description
            } // set end date to string
        },
        Err(e) =>
        {
            log::error!("{e}");
            return calendar_event; // if parsing failed: return unchanged calendar event
        }
    }

    return calendar_event;
}


/// # Summary
/// Takes an IATA location and tries to get the ICAO location, country, and airport name. If not exactly 1 entry could be found, returns None.
///
/// # Arguments
/// - `iata`: IATA location
///
/// # Returns
/// - ICAO location
/// - country name
/// - airport name
fn lookup_iata(iata: String, db: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>) -> Option<IataLookupRow>
{
    const LOOKUP_IATA_QUERY: &str = "SELECT Airport.gps_code AS airport_gps_code, Airport.municipality AS airport_municipality, Country.name AS country_name, Airport.name AS airport_name FROM Airport JOIN Country ON Airport.iso_country = Country.code WHERE Airport.iata_code = ?;"; // query string for iata lookup


    let db_con = db.get().ok()?; // get connection
    return db_con.query_one(LOOKUP_IATA_QUERY, (iata,), |row| { Ok(IataLookupRow
    {
        airport_name: row.get("airport_name")?,
        airport_gps_code: row.get("airport_gps_code")?,
        airport_municipality: row.get("airport_municipality")?,
        country_name: row.get("country_name")?
    })}).ok(); // execute query, if failed return None as if no icao location found
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct IataLookupRow
{
    pub airport_name: String, // Airport.name
    pub airport_gps_code: Option<String>, // Airport.gps_code, icao location
    pub airport_municipality: String, // Airport.municipality, city
    pub country_name: String, // Country.name
}


/// # Summary
/// Takes an IATA location and tries to get the ICAO location. If no entry could be found, returns input value unchanged.
///
/// # Arguments
/// - `iata`: IATA location
///
/// # Returns
/// - ICAO location or unchanged input value
fn try_iata_to_icao(iata: String, db: &r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>) -> String
{
    const IATA_TO_ICAO_QUERY: &str = "SELECT gps_code FROM Airport WHERE iata_code = ?;"; // query string for iata to icao lookup


    let db_con = match db.get() // get connection or fallback to return value unchanged
    {
        Ok(o) => o,
        Err(_) => {return iata;},
    };
    return db_con.query_one(IATA_TO_ICAO_QUERY, (&iata,), |row| { row.get("gps_code")}).unwrap_or(iata); // if no icao location found: forward unchanged value
}