// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.


/// # Summary
/// Airport download response from "ourairports.com/data/airports.csv".
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AirportDownloadResponse
{
    pub id: u32,
    pub ident: String,
    pub r#type: AirportType,
    pub name: String,
    pub latitude_deg: f32,
    pub longitude_deg: f32,
    pub elevation_ft: Option<i32>,
    pub continent: Continent,
    pub iso_country: String,
    pub iso_region: String,
    pub municipality: Option<String>,
    #[serde(deserialize_with = "yes_no_to_bool")]
    pub scheduled_service: bool,
    pub gps_code: Option<String>,
    pub iata_code: Option<String>,
    pub local_code: Option<String>,
    pub home_link: Option<String>,
    pub wikipedia_link: Option<String>,
    pub keywords: Option<String>,
}


#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AirportType
{
    balloonport,
    closed,
    heliport,
    large_airport,
    medium_airport,
    seaplane_base,
    small_airport,
}


#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Continent
{
    AF,
    AN,
    AS,
    EU,
    NA,
    OC,
    SA,
}


/// # Summary
/// Country download response from "ourairports.com/data/countries.csv".
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CountryDownloadResponse
{
    pub id: u32,
    pub code: String,
    pub name: String,
    pub continent: Continent,
    pub wikipedia_link: Option<String>,
    pub keywords: Option<String>,
}


/// # Summary
/// Parses "yes" and "no" to true and false.
///
/// # Arguments
/// - `deserializer`: serde deserializer
///
/// # Returns
/// - bool or serde::Error
fn yes_no_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let scheduled_service: bool;
    let scheduled_service_response: String = serde::Deserialize::deserialize(deserializer)?;

    match scheduled_service_response.as_str()
    {
        "yes" => scheduled_service = true,
        "no" => scheduled_service = false,
        _ => return Err(serde::de::Error::custom(format!("Value \"{scheduled_service_response}\" is neither \"yes\" nor \"no\".")))?,
    };

    return Ok(scheduled_service);
}