// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.


#[derive(Debug, thiserror::Error)]
pub enum DatePerhapsTimeToStringError
{
    #[error("Mapping local time {} with timezone {tz} to UTC failed, likely because it falls into a fold or gap and thus cannot be resolved to UTC unambiguously.", ldt.format("%Y-%m-%dT%H:%M:%S"))]
    LocalTimeMapping{ldt: chrono::NaiveDateTime, tz: chrono_tz::Tz}, // local time mapping error

    #[error("Parsing timezone failed with: {0}")]
    TimezoneParsing(#[from] chrono_tz::ParseError), // local time parse error
}

#[derive(Debug, thiserror::Error)]
pub enum Error
{
    #[error("Creating http client failed with: {0}")]
    Reqwest(#[from] reqwest::Error), // reqwest error
}


#[derive(Debug, thiserror::Error)]
pub enum LoadCalendarError
{
    #[error("Loading calendar from database failed with: {0}")]
    Sqlx(#[from] sqlx::Error), // sqlx error
}


#[derive(Debug, thiserror::Error)]
pub enum UpdateAirportsError
{
    #[error("Downloading airport data from \"{}\" failed with: {}", .0.url().map_or_else(|| "<unknown>", |o| o.as_str()), .0)]
    Reqwest(#[from] reqwest::Error), // reqwest error

    #[error("Updating airports in database failed with: {0}")]
    Sqlx(#[from] sqlx::Error), // sqlx error
}


#[derive(Debug, thiserror::Error)]
pub enum UpdateCalendarError
{
    #[error("{0}")]
    LoadCalendar(#[from] LoadCalendarError), // load calendar error

    #[error("Saving output calendar failed with: {0}")]
    StdIo(#[from] std::io::Error), // std io error

    #[error("{0}")]
    UpdateEvents(#[from] UpdateEventsError), // update events error
}


#[derive(Debug, thiserror::Error)]
pub enum UpdateCountriesError
{
    #[error("Downloading country data from \"{}\" failed with: {}", .0.url().map_or_else(|| "<unknown>", |o| o.as_str()), .0)]
    Reqwest(#[from] reqwest::Error), // reqwest error

    #[error("Updating countries in database failed with: {0}")]
    Sqlx(#[from] sqlx::Error), // sqlx error
}


#[derive(Debug, thiserror::Error)]
pub enum UpdateEventsError
{
    #[error("Parsing input calendar failed with: {0}")]
    Parse(String), // icalendar parse error

    #[error("Downloading input calendar from \"{}\" failed with: {}", .0.url().map_or_else(|| "<unknown>", |o| o.as_str()), .0)]
    Reqwest(#[from] reqwest::Error), // reqwest error

    #[error("Updating events in database failed with: {0}")]
    Sqlx(#[from] sqlx::Error), // sqlx error
}

impl From<String> for UpdateEventsError
{
    fn from(s: String) -> Self
    {
        Self::Parse(s) // #[from] does not like String, that's why this is needed
    }
}