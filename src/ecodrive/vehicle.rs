use crate::ecodrive::config::PrefFloat;
use crate::ecodrive::config::uom_si_preffloat::{Mass, Area, Energy};

use uom::si::{mass::kilogram, area::square_meter, energy::kilowatt_hour};

use crate::ecodrive::constants::{RHO_AIR};
use crate::ecodrive::PerLength;

/// Struct containing all relevant data of a single vehicle.
/// 
/// C parameter is calculated from mass, frontal_area and c_w.

use serde::{Deserialize, Deserializer};

fn deserialize_float_to_kg<'de, D>(d: D) -> Result<Mass, D::Error> where D: Deserializer<'de> {
    let kilograms = PrefFloat::deserialize(d)?;
    Ok(Mass::new::<kilogram>(kilograms))
}

fn deserialize_float_to_sqm<'de, D>(d: D) -> Result<Area, D::Error> where D: Deserializer<'de> {
    let square_meters = PrefFloat::deserialize(d)?;
    Ok(Area::new::<square_meter>(square_meters))
}

fn deserialize_float_to_kWh<'de, D>(d: D) -> Result<Energy, D::Error> where D: Deserializer<'de> {
    let kilowatt_hours = PrefFloat::deserialize(d)?;
    Ok(Energy::new::<kilowatt_hour>(kilowatt_hours))
}

pub fn load_vehicles(path: &str) -> Result<Vec<Vehicle>, csv::Error> {
    let mut vehicles: Vec<Vehicle> = vec![];
    let mut reader = csv::ReaderBuilder::new().trim(csv::Trim::All).from_path(path)?;

    for record in reader.deserialize() {
        let mut vehicle: Vehicle = record?;
        // vehicle.update_c_param();
        vehicles.push(vehicle);
    }

    Ok(vehicles)
}

#[derive(Clone, Debug, Deserialize)]
pub struct Vehicle {
    pub roll_res_coeff: PrefFloat,  // rolling resistance coefficient
    pub rho_rot: PrefFloat,         // factor for equivalent mass of rotating parts
    pub rec_eff: PrefFloat,         // regenerative braking efficiency
    #[serde(deserialize_with="deserialize_float_to_kWh", alias="bat_cap [kWh]")]
    pub bat_cap: Energy,            // battery capacity [kWh]
    #[serde(deserialize_with="deserialize_float_to_kg", alias="mass [kg]")]
    pub mass: Mass,                 // vehicle mass [kg]
    #[serde(deserialize_with="deserialize_float_to_sqm", alias="frontal_area [m^2]")]
    pub frontal_area: Area,         // frontal area [m^2]
    pub c_w: PrefFloat,             // drag coefficient
}

impl Vehicle {
    /// Creates a new vehicle.
    pub fn new(roll_res_coeff: PrefFloat,   // rolling resistance coefficient
                rec_eff: PrefFloat,     // regenerative braking efficiency
                rho_rot: PrefFloat,     // factor for equivalent mass of rotating parts
                bat_cap: Energy,        // battery capacity [kWh]
                mass: Mass,             // vehicle mass [kg]
                frontal_area: Area,     // frontal area [m^2]
                c_w: PrefFloat          // drag coefficient
                ) -> Vehicle {

        // fill the public fields
        let vhl = Vehicle {roll_res_coeff,
                    rec_eff,
                    rho_rot,
                    bat_cap,
                    mass,
                    frontal_area,
                    c_w};

        vhl
    }

    /// Calculates C parameter (mass-normalized air resistance prefactor) [1/m]
    pub fn get_c_param(&self) -> PerLength /* [1/m] */ {
        // self.c_param.expect("c_param not set! Should have been calculated automatically.")
        RHO_AIR * self.c_w * self.frontal_area / self.mass
    }
}