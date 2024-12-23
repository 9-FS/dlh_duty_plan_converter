// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use icalendar::Component;
use crate::duty_plan_event::*;
use crate::error::*;
use crate::transform_calendar_event::*;


pub async fn update_calendar(input_calendar_url: &str, output_calendar_filepath: &str, db: &sqlx::sqlite::SqlitePool) -> Result<(), UpdateCalendarError>
{
    let input_calendar: icalendar::Calendar; // input calendar
    let mut output_calendar: icalendar::Calendar = icalendar::Calendar::new(); // transformed output calendar


    input_calendar = reqwest::get(input_calendar_url).await?.text().await?.parse()?; // download calendar ics
    log::info!("Downloaded and parsed calendar from \"{input_calendar_url}\"."); // log download
    log::debug!("{input_calendar}");


    output_calendar.name("DLH Duty Plan"); // set calendar name
    for calendar_component in input_calendar.components // go through all calendar components and change them as needed
    {
        match calendar_component
        {
            icalendar::CalendarComponent::Event(calendar_event) => // transform event
            {
                match DutyPlanEvent::determine_event(calendar_event.get_summary().unwrap_or_default().to_owned()) // determine event type, transform accordingly
                {
                    DutyPlanEvent::Briefing => {output_calendar.push(transform_briefing(calendar_event, &db).await);},
                    DutyPlanEvent::Deadhead {flight_iata, departure_iata, destination_iata} => {output_calendar.push(transform_deadhead(calendar_event, flight_iata, departure_iata, destination_iata, &db).await);},
                    DutyPlanEvent::Flight {flight_iata, departure_iata, destination_iata} => {output_calendar.push(transform_flight(calendar_event, flight_iata, departure_iata, destination_iata, &db).await);},
                    DutyPlanEvent::Ground {category, description} => {output_calendar.push(transform_ground(calendar_event, category, description, &db).await);},
                    DutyPlanEvent::Layover => {output_calendar.push(transform_layover(calendar_event, &db).await);},
                    DutyPlanEvent::Off => {output_calendar.push(transform_off(calendar_event));},
                    DutyPlanEvent::Unknown => {output_calendar.push(transform_unknown(calendar_event));},
                }
            },
            _ => {output_calendar.push(calendar_component);}, // if not event: forward unchanged
        }
    }
    log::info!("Transformed calendar.");
    log::debug!("{output_calendar}");


    if let Some(parent) = std::path::Path::new(output_calendar_filepath).parent()
    {
        std::fs::create_dir_all(parent)?; // create parent directories if necessary
    }
    std::fs::write(output_calendar_filepath, output_calendar.to_string())?; // save output calendar
    log::info!("Saved transformed calendar to \"{output_calendar_filepath}\".");

    return Ok(());
}