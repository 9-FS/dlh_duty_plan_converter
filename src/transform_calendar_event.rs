// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use icalendar::{Component, EventLike};


/// # Summary
/// Transforms the briefing event. Additionally to the minimum actions changes summary to "Briefing", changes IATA location to ICAO location, and adds alarms at -1,5 h and -5 min.
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `db`: airport database
///
/// # Returns
/// - the transformed calendar event
pub async fn transform_briefing(mut calendar_event: icalendar::Event, db: &sqlx::sqlite::SqlitePool) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event); // always do minimum before specific actions
    calendar_event.summary("Briefing");
    if let Some(row) = lookup_iata(calendar_event.get_location().unwrap_or_default().to_owned(), db).await // if iata location found
    {
        if let Some(s) = row.airport_gps_code // if entry contains icao location
        {
            calendar_event.location(format!("{}: {}, {}", s, row.country_name, row.airport_name).as_str()); // change iata location to icao location
        }
    } // otherwise just keep original data
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::hours(-1))); // add alarm at -1 h
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::minutes(-5))); // add alarm at -5 min

    return calendar_event;
}


/// # Summary
/// Transforms the deadhead event. Additionally to the minimum actions changes summary format, changes IATA locations to departure ICAO location only, and adds an alarm at -1,5 h and -35 min.
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `db`: airport database
///
/// # Returns
/// - the transformed calendar event
pub async fn transform_deadhead(mut calendar_event: icalendar::Event, flight_iata: String, departure_iata: String, destination_iata: String, db: &sqlx::sqlite::SqlitePool) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event); // always do minimum before specific actions
    calendar_event.summary(format!("DEADHEAD {flight_iata}: {} ✈ {}", try_iata_to_icao(departure_iata.to_owned(), db).await, try_iata_to_icao(destination_iata.to_owned(), db).await).as_str()); // change summary format
    if let Some(row) = lookup_iata(departure_iata, db).await // if iata location found
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
/// - `db`: airport database
///
/// # Returns
/// - the transformed calendar event
pub async fn transform_flight(mut calendar_event: icalendar::Event, flight_iata: String, departure_iata: String, destination_iata: String, db: &sqlx::sqlite::SqlitePool) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event); // always do minimum before specific actions
    calendar_event.summary(format!("{flight_iata}: {} ✈ {}", try_iata_to_icao(departure_iata.to_owned(), db).await, try_iata_to_icao(destination_iata.to_owned(), db).await).as_str()); // change summary format
    if let Some(row) = lookup_iata(departure_iata, db).await // if iata location found
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
/// Transforms the ground event. Additionally to the minimum actions changes summary format, changes IATA locations to ICAO location only, and adds alarms at -1 h and -5 min.
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `db`: airport database
///
/// # Returns
/// - the transformed calendar event
pub async fn transform_ground(mut calendar_event: icalendar::Event, category: String, description: String, db: &sqlx::sqlite::SqlitePool) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event); // always do minimum before specific actions
    if category == "" {calendar_event.summary(description.as_str());} // if category is empty: change summary to description
    else {calendar_event.summary(format!("{category}: {description}").as_str());} // otherwise: change summary format only slightly
    if let Some(row) = lookup_iata(calendar_event.get_location().unwrap_or_default().to_owned(), db).await // if iata location found
    {
        calendar_event.location(format!("{}, {}", row.country_name, row.airport_municipality).as_str()); // change iata location to country and city
    } // otherwise just keep original data
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::hours(-1))); // add alarm at -1 h
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::minutes(-5))); // add alarm at -5 min

    return calendar_event;
}


/// # Summary
/// Transforms the layover event. Additionally to the minimum actions changes summary to "Layover" and changes IATA location to ICAO location.
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `db`: airport database
///
/// # Returns
/// - the transformed calendar event
pub async fn transform_layover(mut calendar_event: icalendar::Event, db: &sqlx::sqlite::SqlitePool) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event); // always do minimum before specific actions
    calendar_event.summary("Layover");
    if let Some(row) = lookup_iata(calendar_event.get_location().unwrap_or_default().to_owned(), db).await // if iata location found
    {
        calendar_event.location(format!("{}, {}", row.country_name, row.airport_municipality).as_str()); // change iata location to country and city
    } // otherwise just keep original data
    // calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), (chrono::Duration::minutes(-90), icalendar::Related::End))); // add alarm at -1.5 h before layover end, at layover end briefing begins
    // calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), (chrono::Duration::minutes(-5), icalendar::Related::End))); // add alarm at -5 min before layover end, at layover end briefing begins
    // icalender::Related::End is ignored by Google Calendar, so alarms are commented out

    return calendar_event;
}


/// # Summary
/// Transforms the off event. Additionally to the minimum actions changes summary to "Off".
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
///
/// # Returns
/// - the transformed calendar event
pub fn transform_off(mut calendar_event: icalendar::Event) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event); // always do minimum before specific actions
    calendar_event.location(""); // off day does not need a location
    calendar_event.summary("Off");

    return calendar_event;
}


/// # Summary
/// Transforms the pickup event. Additionally to the minimum actions changes summary to "Pickup", changes IATA location to ICAO location, and adds alarms at -1 h and -15 min.
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
/// - `db`: airport database
///
/// # Returns
/// - the transformed calendar event
pub async fn transform_pickup(mut calendar_event: icalendar::Event, db: &sqlx::sqlite::SqlitePool) -> icalendar::Event
{
    calendar_event = transform_unknown(calendar_event); // always do minimum before specific actions
    calendar_event.summary("Pickup");
    if let Some(row) = lookup_iata(calendar_event.get_location().unwrap_or_default().to_owned(), db).await // if iata location found
    {
        calendar_event.location(format!("{}, {}", row.country_name, row.airport_municipality).as_str()); // change iata location to country and city
    } // otherwise just keep original data
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::hours(-1))); // add alarm at -1 h
    calendar_event.alarm(icalendar::Alarm::display(calendar_event.get_summary().unwrap_or_default(), chrono::Duration::minutes(-15))); // add alarm at -15 min

    return calendar_event;
}


/// # Summary
/// Transforms an unknown event. Only does the minimum: removes the unnecessary description.
///
/// # Arguments
/// - `calendar_event`: the calendar event to transform
///
/// # Returns
/// - the transformed calendar event
pub fn transform_unknown(mut calendar_event: icalendar::Event) -> icalendar::Event
{
    calendar_event.description(""); // remove unnecessary description from mytime

    return calendar_event;
}


/// # Summary
/// Takes an IATA location and tries to get the ICAO location, country, and airport name. If no entry could be found, returns None.
///
/// # Arguments
/// - `iata`: IATA location
///
/// # Returns
/// - ICAO location
/// - country name
/// - airport name
async fn lookup_iata(iata: String, db: &sqlx::sqlite::SqlitePool) -> Option<IataLookupRow>
{
    return sqlx::query_as("SELECT Airport.gps_code AS airport_gps_code, Airport.municipality AS airport_municipality, Country.name AS country_name, Airport.name AS airport_name FROM Airport JOIN Country ON Airport.iso_country = Country.code WHERE Airport.iata_code = ?") // get icao location
        .bind(&iata) // from iata location
        .fetch_optional(db).await.unwrap_or_default(); // execute query, if failed return None as if no icao location found
}

#[derive(Clone, Debug, Eq, PartialEq, sqlx::FromRow)]
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
async fn try_iata_to_icao(iata: String, db: &sqlx::sqlite::SqlitePool) -> String
{
    return sqlx::query_scalar("SELECT gps_code FROM Airport WHERE iata_code = ?") // get icao location
        .bind(&iata) // from iata location
        .fetch_optional(db).await.unwrap_or_default() // execute query, if failed return None as if no icao location found
        .unwrap_or(iata); // if no icao location found: forward unchanged value
}