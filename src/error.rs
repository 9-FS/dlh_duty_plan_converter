// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.


#[derive(Debug, thiserror::Error)]
pub enum Error
{
}


#[derive(Debug, thiserror::Error)]
pub enum UpdateAirportDataError
{
    #[error("Downloading airport data from \"{}\" failed with: {}", .0.url().map_or_else(|| "<unknown>", |o| o.as_str()), .0)]
    Reqwest(#[from] reqwest::Error), // reqwest error

    #[error("Connecting to database failed with: {0}")]
    Sqlx(#[from] sqlx::Error), // sqlx error
}


#[derive(Debug, thiserror::Error)]
pub enum UpdateCalendarError
{
    #[error("Downloading input calendar from \"{}\" failed with: {}", .0.url().map_or_else(|| "<unknown>", |o| o.as_str()), .0)]
    Reqwest(#[from] reqwest::Error), // reqwest error

    #[error("Saving output calendar failed with: {0}")]
    StdIo(#[from] std::io::Error), // std io error

    #[error("Parsing input calendar failed with: {0}")]
    Parse(String), // icalendar parse error
}

impl From<String> for UpdateCalendarError
{
    fn from(s: String) -> Self
    {
        Self::Parse(s) // #[from] does not like String, that's why this is needed
    }
}


#[derive(Debug, thiserror::Error)]
pub enum UpdateCountryDataError
{
    #[error("Downloading country data from \"{}\" failed with: {}", .0.url().map_or_else(|| "<unknown>", |o| o.as_str()), .0)]
    Reqwest(#[from] reqwest::Error), // reqwest error

    #[error("Connecting to database failed with: {0}")]
    Sqlx(#[from] sqlx::Error), // sqlx error
}