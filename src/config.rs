// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.


/// # Summary
/// Collection of settings making up the configuration of the application.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[allow(non_snake_case)]
pub struct Config
{
    pub DEBUG: Option<bool>, // debug mode?
    pub INPUT_CALENDAR_URL: String, // original calendar url to read from
    pub OUTPUT_CALENDAR_FILEPATH: String, // file path to write calendar to
    pub SLEEP_INTERVAL: u64, // sleep interval between calendar updates
}

impl Default for Config
{
    fn default() -> Self
    {
        Self
        {
            DEBUG: None, // no entry in default config, defaults to false
            INPUT_CALENDAR_URL: "".to_owned(), // default calendar url
            OUTPUT_CALENDAR_FILEPATH: "./calendar/duty_plan.ics".to_owned(), // default calendar file path
            SLEEP_INTERVAL: 500, // default sleep interval
        }
    }
}