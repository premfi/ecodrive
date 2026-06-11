use crate::ecodrive::config::uom_si_preffloat::{Acceleration, MassDensity, Velocity};
use std::marker::PhantomData;

pub const GRAVITY_OF_EARTH: Acceleration = Acceleration {dimension:PhantomData, units: PhantomData, value: 9.81}; // gravitational acceleration [m/s^2]
pub const RHO_AIR: MassDensity = MassDensity {dimension: PhantomData, units: PhantomData, value: 1.2}; // air density [kg/m^3]
pub const GLOBAL_V_MAX: Velocity = Velocity {dimension: PhantomData, units: PhantomData, value: 50.}; // global maximum velocity [m/s]
pub const GLOBAL_MOM_MAX: Acceleration = Acceleration {dimension:PhantomData, units: PhantomData, value: 3.0}; // global maximum applied momentum [m/s^2], negative is used as minimum