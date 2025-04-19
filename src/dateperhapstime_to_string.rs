// Copyright (c) 2025 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use std::str::FromStr;
use crate::error::*;


/// # Summary
/// Converts a icalendar::DatePerhapsTime to a String.
///
/// # Arguments
/// - `dt`: date or perhaps datetime to convert
///
/// # Returns
/// - String or error
pub fn dateperhapstime_to_string(dt: icalendar::DatePerhapsTime) -> Result<String, DatePerhapsTimeToStringError>
{
    match dt
    {
        icalendar::DatePerhapsTime::Date(dt) => return Ok(format!("{}", dt.format("%Y-%m-%d"))), // only date
        icalendar::DatePerhapsTime::DateTime(dt) =>
        {
            match dt
            {
                icalendar::CalendarDateTime::Floating(dt) => return Ok(format!("{}", dt.format("%Y-%m-%dT%H:%M:%S"))), // assume utc
                icalendar::CalendarDateTime::Utc(dt) => return Ok(format!("{}", dt.format("%Y-%m-%dT%H:%M:%SZ"))),
                icalendar::CalendarDateTime::WithTimezone { date_time: dt, tzid } => // consider timezone
                {
                    let tz: chrono_tz::Tz = chrono_tz::Tz::from_str(&tzid)?; // parse timezone
                    let utc = dt.and_local_timezone(tz).single().ok_or(DatePerhapsTimeToStringError::LocalTimeMapping{ldt: dt, tz})?.with_timezone(&chrono::Utc); // create local time, then convert to utc
                    return Ok(format!("{}", utc.format("%Y-%m-%dT%H:%M:%SZ")));
                },
            }
        },
    }
}