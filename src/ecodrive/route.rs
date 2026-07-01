use crate::ecodrive::config::PrefFloat;
use crate::ecodrive::config::uom_si_preffloat::{Length, Ratio, Velocity};

use uom::si::{length::meter, ratio::percent, velocity::kilometer_per_hour};

pub struct Route {
    pub lengths: Vec<Length>,
    pub slopes: Vec<Ratio>,
    pub min_speeds: Vec<Velocity>,
    pub max_speeds: Vec<Velocity>, 
    pub roll_res_factors: Vec<PrefFloat>,
}

fn roll_res_factor_default() -> PrefFloat {1.0}

use serde::Deserialize;
#[derive(Debug, Deserialize)]
struct RouteSection {
    #[serde(alias = "length [m]")]
    pub length_m: PrefFloat,

    #[serde(alias = "slope [pct]")]
    pub slope_pct: PrefFloat,

    #[serde(alias = "min_speed [km/h]", default)]
    pub min_speed_kph: PrefFloat,

    #[serde(alias = "max_speed [km/h]")]
    pub max_speed_kph: PrefFloat,

    #[serde(default = "roll_res_factor_default")]
    pub roll_res_factor: PrefFloat,
}

pub fn load_route(path: &str) -> Result<Route, csv::Error> {
    println!("Loading route from {}", path);

    // initialize empty route
    let mut route = Route {
                        lengths: vec![],
                        slopes: vec![],
                        min_speeds: vec![],
                        max_speeds: vec![],
                        roll_res_factors: vec![]};

    // parse csv file and add entries line by line
    let mut reader = csv::ReaderBuilder::new().trim(csv::Trim::All).from_path(path)?;
    for record in reader.deserialize() {
        let section: RouteSection = record?;

        // convert and add entries to route
        route.lengths.push(Length::new::<meter>(section.length_m));
        route.slopes.push(Ratio::new::<percent>(section.slope_pct));
        route.min_speeds.push(Velocity::new::<kilometer_per_hour>(section.min_speed_kph));
        route.max_speeds.push(Velocity::new::<kilometer_per_hour>(section.max_speed_kph));
        route.roll_res_factors.push(section.roll_res_factor);
    }

    Ok(route)
}