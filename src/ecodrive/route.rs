use crate::ecodrive::config::PrefFloat;
use crate::ecodrive::config::uom_si_preffloat::{Length, Ratio, Velocity};

use uom::si::{length::meter, ratio::percent, velocity::kilometer_per_hour};

#[derive(Debug)]
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


impl Route {
    /// Partitions each section into `num` equal parts.
    pub fn partition(&self, num: usize) -> Route {
        let new_len = self.lengths.len() * num;

        let mut route = Route {
                        lengths: Vec::with_capacity(new_len),
                        slopes: Vec::with_capacity(new_len),
                        min_speeds: Vec::with_capacity(new_len),
                        max_speeds: Vec::with_capacity(new_len),
                        roll_res_factors: Vec::with_capacity(new_len)};

        for i in 0..self.lengths.len() {
            let split_length = self.lengths[i] / num as PrefFloat;
            route.lengths.extend(vec![split_length; num]);
            route.slopes.extend(vec![self.slopes[i]; num]);
            route.min_speeds.extend(vec![self.min_speeds[i]; num]);
            route.max_speeds.extend(vec![self.max_speeds[i]; num]);
            route.roll_res_factors.extend(vec![self.roll_res_factors[i]; num]);
        }

        route
    }
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