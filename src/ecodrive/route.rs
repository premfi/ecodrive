use crate::ecodrive::config::PrefFloat;
use crate::ecodrive::config::uom_si_preffloat::{Length, Ratio, Velocity};

use uom::si::{length::meter, ratio::percent, velocity::kilometer_per_hour};

pub struct Route {
    pub lengths: Vec<Length>,
    pub slopes: Vec<Ratio>,
    pub min_speeds: Vec<Velocity>,
    pub max_speeds: Vec<Velocity>, 

}


use serde::Deserialize;
#[derive(Debug, Deserialize)]
struct RouteSection {
    #[serde(alias = "length [m]")]
    pub length_m: PrefFloat,

    #[serde(alias = "slope [%]")]
    pub slope_pct: PrefFloat,

    #[serde(alias = "min_speed [km/h]")]
    pub min_speed_kph: Option<PrefFloat>,

    #[serde(alias = "max_speed [km/h]")]
    pub max_speed_kph: PrefFloat,
}

pub fn load_route(path: &str) -> Result<Route, csv::Error> {
    println!("Loading route from {}", path);

    // initialize empty route
    let mut route = Route {
                        lengths: vec![],
                        slopes: vec![],
                        min_speeds: vec![],
                        max_speeds: vec![]};

    // parse csv file and add entries line by line
    let mut reader = csv::ReaderBuilder::new().trim(csv::Trim::All).from_path(path)?;
    for record in reader.deserialize() {
        let section: RouteSection = record?;
        
        println!(
            "s= {}, slope= {}, v_max= {}",
            section.length_m,
            section.slope_pct,
            section.max_speed_kph,
        );

        // convert and add entries to route
        route.lengths.push(Length::new::<meter>(section.length_m));
        route.slopes.push(Ratio::new::<percent>(section.slope_pct));
        route.min_speeds.push(Velocity::new::<kilometer_per_hour>(section.min_speed_kph.unwrap_or_default()));
        route.max_speeds.push(Velocity::new::<kilometer_per_hour>(section.max_speed_kph));
    }

    Ok(route)
}