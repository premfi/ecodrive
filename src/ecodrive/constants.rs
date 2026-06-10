use crate::ecodrive::config::uom_si_preffloat::{Acceleration, MassDensity};
use std::marker::PhantomData;

pub const GRAVITY_OF_EARTH: Acceleration = Acceleration {dimension:PhantomData, units: PhantomData, value: 9.81}; // gravitational acceleration [m/s^2]
pub const RHO_AIR: MassDensity = MassDensity {dimension: PhantomData, units: PhantomData, value: 1.2}; // air density [kg/m^3]