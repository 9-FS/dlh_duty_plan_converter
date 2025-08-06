// Copyright (c) 2025 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use icalendar::Component;
use icalendar::EventLike;
use crate::error::*;


/// # Summary
/// Loads the whole calendar from the database at `db` and returns the calendar as icalendar::Calendar.
///
/// # Arguments
/// - `db`: database connection
///
/// # Returns
/// - calendar or error
pub fn load_calendar(db: &rusqlite::Connection) -> Result<icalendar::Calendar, LoadCalendarError>
{
    const LOAD_CALENDAR_QUERY_STRING: &str = "SELECT * FROM Event ORDER BY start_dt ASC;"; // query to load calendar from database
    let mut calendar: icalendar::Calendar = icalendar::Calendar::new(); // calendar to be returned


    let mut db_stmt = db.prepare(LOAD_CALENDAR_QUERY_STRING)?; // prepare query
    let events = db_stmt.query_map((), |row|
    {
        let mut event = icalendar::Event::new();
        event.uid(row.get::<&str, std::string::String>("uid")?.as_str()); // set uid
        event.summary(row.get::<&str, std::string::String>("summary")?.as_str()); // set summary
        match row.get::<&str, chrono::DateTime<chrono::Utc>>("start_dt") // try to load start as datetime
        {
            Ok(o) => event.starts(o),
            Err(_) => event.starts(row.get::<&str, chrono::NaiveDate>("start_dt")?), // if not possible: try to load as date
        };
        match row.get::<&str, chrono::DateTime<chrono::Utc>>("end_dt") // try to load end as datetime
        {
            Ok(o) => event.ends(o),
            Err(_) => event.ends(row.get::<&str, chrono::NaiveDate>("end_dt")?), // if not possible: try to load as date
        };
        event.location(row.get::<&str, std::string::String>("location")?.as_str());
        event.description(row.get::<&str, std::string::String>("description")?.as_str());

        Ok(event)
    })?;
    for event in events // add events to calendar
    {
        calendar.push(event?);
    }

    return Ok(calendar);
}