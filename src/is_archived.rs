// Copyright (c) 2025 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.


/// # Summary
/// Checks if the provided date or datetime is old enough to be archived.
///
/// # Arguments
/// - `dt_str`: the date or datetime to check
/// - `archive_end_dt`: datetime when to archive ends, latest datetime to be considered for archiving
///
/// # Returns
/// - `true` if the event should be considered archived, `false` otherwise
pub fn is_archived(dt_str: &str, archive_end_dt: &chrono::DateTime<chrono::Utc>) -> Result<bool, chrono::ParseError>
{
    let dt: chrono::DateTime<chrono::Utc>;


    match chrono::DateTime::parse_from_rfc3339(dt_str) // try to parse proper datetime
    {
        Ok(o) => dt = o.with_timezone(&chrono::Utc), // convert to utc
        Err(_) =>
        {
            match chrono::NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%dT%H:%M:%S") // try to parse as naive datetime
            {
                Ok(o) => dt = o.and_utc(), // assume utc
                Err(_) =>
                {
                    dt = chrono::NaiveDate::parse_from_str(dt_str, "%Y-%m-%d")? // try to parse as naive date
                        .and_hms_opt(0, 0, 0).expect("Appending default time 00:00:00 to date failed even though it is hard coded and should always be valid.") // append default time midnight
                        .and_utc(); // assume utc
                }
            }
        }
    }

    return Ok(dt <= *archive_end_dt); // if event ended in archive datetime or older: event should be archived
}