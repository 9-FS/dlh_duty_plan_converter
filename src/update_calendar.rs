// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use icalendar::Component;
use crate::error::*;
use crate::event_type::*;
use crate::load_calendar::*;
use crate::transform_calendar_event::*;
use crate::update_db::*;


/// # Summary
/// Downloads calendar from myTime, parses it, and updates the database table "Event". After that, loads the whole calendar from the database, transforms it, and saves it to a file.
///
/// # Arguments
/// - `http_client`: http client
/// - `input_calendar_url`: calendar source URL
/// - `output_calendar_filepath`: calendar output file path
/// - `db`: database connection
/// - `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - nothing or error
pub fn update_calendar(http_client: &reqwest::blocking::Client, input_calendar_url: &str, output_calendar_filepath: &str, mut db: &mut rusqlite::Connection, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> Result<(), UpdateCalendarError>
{
    const ALERT_TRIGGER_PATTERN: &str = r"PT(?P<t_trigger>[0-9]+)S"; // alert trigger pattern in calendar ical, purposely disregard potential minus sign in front of "PT" to keep it unchanged
    let input_calendar: icalendar::Calendar; // input calendar
    let mut output_calendar: icalendar::Calendar = icalendar::Calendar::new(); // transformed output calendar


    update_events(http_client, input_calendar_url, db, archive_end_dt)?;
    input_calendar = load_calendar(db)?; // load whole calendar from database


    output_calendar.name("DLH Duty Plan"); // set calendar name
    for calendar_component in input_calendar.components // go through all calendar components and change them as needed
    {
        match calendar_component
        {
            icalendar::CalendarComponent::Event(calendar_event) => // transform event
            {
                match EventType::determine_event_type(calendar_event.get_summary().unwrap_or_default().to_owned()) // determine event type, transform accordingly
                {
                    EventType::Briefing => {output_calendar.push(transform_briefing(calendar_event, &mut db, archive_end_dt));},
                    EventType::Deadhead {flight_iata, departure_iata, destination_iata} => {output_calendar.push(transform_deadhead(calendar_event, flight_iata, departure_iata, destination_iata, &mut db, archive_end_dt));},
                    EventType::Flight {flight_iata, departure_iata, destination_iata} => {output_calendar.push(transform_flight(calendar_event, flight_iata, departure_iata, destination_iata, &mut db, archive_end_dt));},
                    EventType::Ground {category, description} => {output_calendar.push(transform_ground(calendar_event, category, description, &mut db, archive_end_dt));},
                    EventType::Holiday => {output_calendar.push(transform_holiday(calendar_event, archive_end_dt));},
                    EventType::Layover => {output_calendar.push(transform_layover(calendar_event, &mut db, archive_end_dt));},
                    EventType::Off => {output_calendar.push(transform_off(calendar_event, archive_end_dt));},
                    EventType::Pickup => {output_calendar.push(transform_pickup(calendar_event, &mut db, archive_end_dt));},
                    EventType::Sickness => {output_calendar.push(transform_sickness(calendar_event, archive_end_dt));},
                    EventType::Unknown => {output_calendar.push(transform_unknown(calendar_event, archive_end_dt));},
                }
            },
            _ => {output_calendar.push(calendar_component);}, // if not event: forward unchanged
        }
    }
    let output_calendar: String = regex::Regex::new(ALERT_TRIGGER_PATTERN).expect("Compiling alert trigger regex failed.").replace_all(&output_calendar.to_string(), |captures: &regex::Captures|
    {
        let t_trigger: i32 = captures["t_trigger"].parse().expect("Parsing alert trigger to i32 failed even though regex should have made sure it can't."); // parse alert trigger
        if t_trigger.rem_euclid(3600) == 0 {format!("PT{}H", t_trigger / 3600)} // if alert trigger is a multiple of an hour: convert to hours
        else if t_trigger.rem_euclid(60) == 0 {format!("PT{}M", t_trigger / 60)} // if alert trigger is a multiple of a minute: convert to minutes
        else {captures["t_trigger"].to_owned()} // return unchanged
    }).to_string(); // Calendar -> String, convert alert triggers in seconds to hours or minutes for google calendar compatibility
    log::info!("Transformed calendar.");
    log::debug!("{output_calendar}");


    if let Some(parent) = std::path::Path::new(output_calendar_filepath).parent()
    {
        std::fs::create_dir_all(parent)?; // create parent directories if necessary
    }
    std::fs::write(output_calendar_filepath, output_calendar)?; // save output calendar
    log::info!("Saved transformed calendar to \"{output_calendar_filepath}\".");

    return Ok(());
}