// Copyright (c) 2025 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use icalendar::Component;
use icalendar::EventLike;
use sqlx::Row;
use crate::error::*;


/// # Summary
/// Loads the whole calendar from the database at `db` and returns the calendar as icalendar::Calendar.
///
/// # Arguments
/// - `db`: database connection
///
/// # Returns
/// - calendar or error
pub async fn load_calendar(db: &sqlx::sqlite::SqlitePool) -> Result<icalendar::Calendar, LoadCalendarError>
{
    const LOAD_CALENDAR_QUERY_STRING: &str = "SELECT * FROM Event ORDER BY start_dt ASC"; // query to load calendar from database
    let mut calendar: icalendar::Calendar = icalendar::Calendar::new(); // calendar to be returned
    let rows: Vec<sqlx::sqlite::SqliteRow>; // db rows


    rows = sqlx::query(LOAD_CALENDAR_QUERY_STRING).fetch_all(db).await?; // load all events from database

    for row in rows // go through all events, create icalendar events and add them to the calendar
    {
        let mut event = icalendar::Event::new();
        event.uid(row.get::<&str, _>("uid")); // set uid
        event.summary(row.get::<&str, _>("summary"));
        match row.try_get::<chrono::DateTime<chrono::Utc>, _>("start_dt")
        {
            Ok(_) => event.starts(row.get::<chrono::DateTime<chrono::Utc>, _>("start_dt")), // try to load as datetime
            Err(_) => event.starts(row.get::<chrono::NaiveDate, _>("start_dt")), // if not possible: try to load as date
        };
        match row.try_get::<chrono::DateTime<chrono::Utc>, _>("end_dt")
        {
            Ok(_) => event.ends(row.get::<chrono::DateTime<chrono::Utc>, _>("end_dt")), // try to load as datetime
            Err(_) => event.ends(row.get::<chrono::NaiveDate, _>("end_dt")), // if not possible: try to load as date
        };
        event.location(row.get::<&str, _>("location"));
        event.description(row.get::<&str, _>("description"));

        calendar.push(event); // add event to calendar
    }

    return Ok(calendar);
}