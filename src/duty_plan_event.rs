// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.


/// # Summary
/// Types of events the duty plan can have. Each event defines a transformation to be made for the output calendar, "Rest" means nothing is changed. Contains the regex patterns to match the events.
#[derive(Clone, Debug)]
pub enum DutyPlanEvent
{
    Briefing, // briefing before a rotation
    Deadhead {flight_iata: String, departure_iata: String, destination_iata: String}, // deadhead from A to B
    Flight {flight_iata: String, departure_iata: String, destination_iata: String}, // flight from A to B
    Ground {category: String, description: String}, // ground event like simulator, classroom
    Holiday, // holiday
    Layover, // layover somewhere else
    Off, // free day
    Pickup, // hotel pickup
    Sickness, // sickness
    Unknown, // unknown events with no specially defined behaviour, only do minimum
}

impl DutyPlanEvent
{
    /// # Summary
    /// Determine the event type of a calendar event based on its summary.
    ///
    /// # Arguments
    /// - `calendar_event_summary`: the summary of the calendar event to determine the event type of
    ///
    /// # Returns
    /// - the determined event type or `DutyPlanEvent::Default` if the event type could not be determined
    pub fn determine_event(calendar_event_summary: String) -> Self
    {
        const BRIEFING_PATTERN: &str = r"^(\d{2}:\d{2} LT Briefing [A-Z]{3})$";
        const DEADHEAD_PATTERN: &str = r"^(DH (?P<flight_iata>[\dA-Z][A-Z] \d{1,4}): (?P<departure_iata>[A-Z]{3})-(?P<destination_iata>[A-Z]{3}))$";
        const FLIGHT_PATTERN: &str = r"^((?P<flight_iata>[\dA-Z][A-Z] \d{1,4}): (?P<departure_iata>[A-Z]{3})-(?P<destination_iata>[A-Z]{3}))$";
        const GROUND_PATTERN: &str = r"^((?P<category>GeneralEvent|Mandatory Training|Simulator) \((?P<description>.+)\))$";
        const HOLIDAY_PATTERN: &str = r"^(Absence \(U\))$";
        const LAYOVER_PATTERN: &str = r"^(LAYOVER)$";
        const OFF_PATTERN: &str = r"^(Off Day \(ORTSTAG\)|Off Day \(OFF\))$";
        const PICKUP_PATTERN: &str = r"^(\d{2}:\d{2} LT Pickup [A-Z]{3})$";
        const SICKNESS_PATTERN: &str = r"^(Sickness \(K(O)?\))$";


        if regex::Regex::new(BRIEFING_PATTERN).expect("Compiling briefing regex failed.").is_match(calendar_event_summary.as_str())
        {
            return Self::Briefing;
        }
        else if let Some(captures) = regex::Regex::new(DEADHEAD_PATTERN).expect("Compiling deadhead regex failed.").captures(calendar_event_summary.as_str())
        {
            return Self::Deadhead {flight_iata: captures["flight_iata"].replace(" ", ""), departure_iata: captures["departure_iata"].to_owned(), destination_iata: captures["destination_iata"].to_owned()}; // remove spaces from flight number
        }
        else if let Some(captures) = regex::Regex::new(FLIGHT_PATTERN).expect("Compiling flight regex failed.").captures(calendar_event_summary.as_str())
        {
            return Self::Flight {flight_iata: captures["flight_iata"].replace(" ", ""), departure_iata: captures["departure_iata"].to_owned(), destination_iata: captures["destination_iata"].to_owned()}; // remove spaces from flight number
        }
        else if let Some(captures) = regex::Regex::new(GROUND_PATTERN).expect("Compiling ground regex failed.").captures(calendar_event_summary.as_str())
        {
            let category_mapping: std::collections::HashMap<&str, &str> = std::collections::HashMap::from
            ([
                ("GeneralEvent", ""), // unnecessary, no information value with this
                ("Mandatory Training", "Training")
            ]); // map categories to shorter and prettier versions, if not in here forward category unchanged
            return Self::Ground {category: category_mapping.get(&captures["category"]).unwrap_or(&&captures["category"]).to_string(), description: captures["description"].to_owned()};
        }
        else if regex::Regex::new(HOLIDAY_PATTERN).expect("Compiling holiday regex failed.").is_match(calendar_event_summary.as_str())
        {
            return Self::Holiday;
        }
        else if regex::Regex::new(LAYOVER_PATTERN).expect("Compiling layover regex failed.").is_match(calendar_event_summary.as_str())
        {
            return Self::Layover;
        }
        else if regex::Regex::new(OFF_PATTERN).expect("Compiling off regex failed.").is_match(calendar_event_summary.as_str())
        {
            return Self::Off;
        }
        else if regex::Regex::new(PICKUP_PATTERN).expect("Compiling pickup regex failed.").is_match(calendar_event_summary.as_str())
        {
            return Self::Pickup;
        }
        else if regex::Regex::new(SICKNESS_PATTERN).expect("Compiling sickness regex failed.").is_match(calendar_event_summary.as_str())
        {
            return Self::Sickness;
        }
        else // if nothing matches: only do minimum
        {
            log::warn!("Could not determine duty plan event type of summary: \"{calendar_event_summary}\"");
            return Self::Unknown;
        }
    }
}