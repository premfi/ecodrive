use uom::si::{acceleration::meter_per_second_squared,
            available_energy::joule_per_kilogram,
            velocity::meter_per_second,
            time::second};
use std::marker::PhantomData;
use float_cmp::approx_eq;

pub mod config;
use config::PrefFloat;
use config::uom_si_preffloat::{Mass, Area, Acceleration, MassDensity, Length, Velocity, Ratio, AvailableEnergy, Time};
use config::floats;

pub mod constants;
use constants::{GRAVITY_OF_EARTH, RHO_AIR};

mod vehicle;
pub use vehicle::Vehicle;

use uom::typenum::{N1, Z0};
pub type PerLength = uom::si::Quantity<uom::si::ISQ<N1, Z0, Z0, Z0, Z0, Z0, Z0>,
                                            uom::si::SI<PrefFloat>, PrefFloat>; // [1/m]

use ndarray::Array1;


pub struct Route {
    pub lengths: Vec<Length>,
    pub slopes: Vec<Ratio>,
    pub max_speeds: Vec<Velocity>, 

}

// fn e_kin(s, )

/// Returns specific used energy when applying moment `mom` on length `s`.
pub fn energy_used(s: Length, mom: Acceleration /* [N/kg] */, rec_eff: PrefFloat) -> AvailableEnergy /* [J/kg] */ {
    if mom >= Acceleration::new::<meter_per_second_squared>(0.0) {
        s * mom
    } else {
        s * mom * rec_eff
    }
}

/// Returns the specific route resistance, consisting of slope and rolling resistance.
pub fn route_res(slope: Ratio, roll_res_coeff: PrefFloat) -> Acceleration /* [N/kg] */ {
    GRAVITY_OF_EARTH * (slope.get::<uom::si::ratio::ratio>() + roll_res_coeff)
}

/// Calculates A parameter necessary to reach `ekin_s` after length `s` when starting with `ekin_0`.
pub fn retrieve_a_param(s: Length, ekin_0: AvailableEnergy, ekin_s: AvailableEnergy, c_param: PerLength) -> Acceleration /* [N/kg] */ {
    let a_param = c_param * (ekin_s - ekin_0 * floats::consts::E.powf((-c_param * s).into())) / (1.0 - floats::consts::E.powf((-c_param * s).into()));
    a_param
}

/// Calculates the time needed for a section.
pub fn delta_t(s: Length, a_param: Acceleration, c_param: PerLength, ekin_0: AvailableEnergy) -> Time {
   
    // A = 0
    if approx_eq!(PrefFloat, a_param.value, 0.0, ulps=2) {

        // A = 0, ekin_0 > 0
        if ekin_0 > AvailableEnergy::new::<joule_per_kilogram>(0.0) {
            return PrefFloat::sqrt(2.0) / (c_param * ekin_0.sqrt()) * (PrefFloat::from(c_param * s / 2.0).exp() - 1.0);
        }
        
        // A = 0, no ekin_0 -> impossible to reach `s`
        else {
            println!("case 1.1");
            return Time::new::<second>(floats::INFINITY);
        }
    }
    
    // positive A
    else if a_param > Acceleration::new::<meter_per_second_squared>(0.0) {
        // constant speed
        if approx_eq!(PrefFloat, ((c_param / a_param) * ekin_0).into(), 1.0, ulps=2) {
            return s / (2.0 * ekin_0).sqrt();
        } 
        // higher or lower end velocity, but always > 0
        else {
            // approximation for numerical stability
            if PrefFloat::from(s * c_param) > 12.0 {
                let max_stable_s = 12.0 / c_param;
                let y_axis_offset = delta_t(max_stable_s, a_param, c_param, ekin_0);
                let m = c_param / (2.0 * a_param * c_param).sqrt();
                return m * (s - max_stable_s) + y_axis_offset;
            } 
            // actual formula
            else {
                let x: PrefFloat = (1.0 + (PrefFloat::from((c_param / a_param) * ekin_0) - 1.0) * PrefFloat::from((-c_param * s).exp())).sqrt();
                let y: PrefFloat = ((c_param / a_param) * ekin_0).sqrt().into();
                return PrefFloat::sqrt(2.0) * ((x - y) / (1.0 - x * y)).atanh() / (a_param * c_param).sqrt();
            }
        }
    }

    // negative A
    else {
        // end speed will be exactly zero
        if approx_eq!(PrefFloat, (-c_param * ekin_0 / (PrefFloat::from((c_param * s).exp()) - 1.0)).value, a_param.value, ulps=2) {
            // println!("case 3.1");
            let x: PrefFloat = 0.0;
            let y: PrefFloat = (-(c_param / a_param) * ekin_0).sqrt().into();
            return (-2.0 / (a_param * c_param)).sqrt() * ((y - x) / (1.0 + x * y)).atan();
        }
        // end speed larger than zero
        else if (-c_param * ekin_0 / (PrefFloat::from(c_param * s).exp() - 1.0)) < a_param {
            // println!("case 3.2");
            let x: PrefFloat = (-1.0 - (PrefFloat::from((c_param / a_param) * ekin_0) - 1.0) * floats::consts::E.powf((-c_param * s).into())).sqrt();
            let y: PrefFloat = (-(c_param / a_param) * ekin_0).sqrt().into();
            return (2.0 / (-a_param * c_param)).sqrt() * ((y - x) / (1.0 + x * y)).atan();
        } 
        // A too small, end will not be reached
        else {
            // println!("case 3.3");
            return Time::new::<second>(floats::INFINITY);
        }
    }
        
}

/// Converts v into corresponding specific kinetic energy.
pub fn v_to_ekin(v: Velocity) -> AvailableEnergy {
    v * v / 2.0
}

// fn ekin_to_v(ekin) // not necessary in direct DP version

// *discretize and undiscretice t and v (4 functions in total)*

/// Assings `t` to its corresponding bin from [0, `num-1`]. Bins are linearly spaced between `min` and `max`.
/// Positive values out of range are clamped into the edge bins. Panics for negative `t` inputs.
pub fn discretize_time(t: Time, min: Option<Time>, max: Time, num: usize) -> usize {

    assert!(t >= Time::new::<second>(0.0), "`t` must be non-negative! t={:?}", t);

    let min = min.unwrap_or(Time::new::<second>(0.0));
    assert!(min <= max, "`min` must not be larger than `max`! min={:?}, max={:?}", min, max);

    if t > max {
        return num - 1;
    }

    if t < min {
        return 0;
    }

    let stepsize = (max - min) / (num - 1) as PrefFloat;
    let bin = PrefFloat::from((t - min) / stepsize).ceil() as usize;
    
    bin
}

/// Translates time bin to corresponding Time value.
pub fn time_bin_to_seconds(bin: usize, min: Option<Time>, max: Time, num: usize) -> Time {
    let min = min.unwrap_or(Time::new::<second>(0.0));
    let stepsize = (max - min) / (num - 1) as PrefFloat;

    stepsize * (bin as PrefFloat) + min
}

/// Assings `v` to its corresponding bin from [0, `num-1`]. Bins are linearly spaced between `min` and `max`.
/// Positive values out of range are clamped into the edge bins. Panics for negative `v` inputs.
pub fn discretize_v(v: Velocity, min: Option<Velocity>, max: Velocity, num: usize) -> usize {

    assert!(v >= Velocity::new::<meter_per_second>(0.0), "`v`must be non-negative! v={:?}", v);
    
    let min = min.unwrap_or(Velocity::new::<meter_per_second>(0.0));
    assert!(min <= max, "`min` must not be larger than `max`! min={:?}, max={:?}", min, max);

    if v > max {
        return num - 1;
    }

    if v < min {
        return 0;
    }

    let stepsize = (max - min) / (num - 1) as PrefFloat;
    let bin = PrefFloat::from((v - min) / stepsize).floor() as usize;

    bin
}

/// Translates speed bin to corresponding Velocity.
pub fn v_bin_to_mps(bin: usize, min: Option<Velocity>, max: Velocity, num: usize) -> Velocity {
    let min = min.unwrap_or(Velocity::new::<meter_per_second>(0.0));
    let stepsize = (max - min) / (num - 1) as PrefFloat;

    stepsize * (bin as PrefFloat) + min
}

pub fn dp_optim(route: &Route, vehicle: &Vehicle, max_time: Time, t_res: usize, v_res: usize) -> i16 {
    println!("DP called!");

    0
}