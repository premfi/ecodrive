use uom::si::{acceleration::meter_per_second_squared,
            available_energy::joule_per_kilogram,
            velocity::{meter_per_second, kilometer_per_hour},
            time::second,
            ratio::percent,
            length::meter};
use std::marker::PhantomData;
use float_cmp::approx_eq;

pub mod config;
use config::PrefFloat;
use config::uom_si_preffloat::{Mass, Area, Acceleration, MassDensity, Length, Velocity, Ratio, AvailableEnergy, Time};
use config::floats;

pub mod constants;
use constants::{GRAVITY_OF_EARTH, RHO_AIR, GLOBAL_V_MAX, GLOBAL_MOM_MAX};

mod vehicle;
pub use vehicle::{Vehicle, load_vehicles};

use uom::typenum::{N1, Z0};
pub type PerLength = uom::si::Quantity<uom::si::ISQ<N1, Z0, Z0, Z0, Z0, Z0, Z0>,
                                            uom::si::SI<PrefFloat>, PrefFloat>; // [1/m]

use ndarray::{Array3, Axis};
use ndarray_stats::QuantileExt;

use std::time;

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
    
    std::cmp::min(bin, num - 1)
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

    std::cmp::min(bin, num - 1)
}

/// Translates speed bin to corresponding Velocity.
pub fn v_bin_to_mps(bin: usize, min: Option<Velocity>, max: Velocity, num: usize) -> Velocity {
    let min = min.unwrap_or(Velocity::new::<meter_per_second>(0.0));
    let stepsize = (max - min) / (num - 1) as PrefFloat;

    stepsize * (bin as PrefFloat) + min
}

pub fn dp_optim(route: &Route, vehicle: &Vehicle, max_time: Time, t_res: usize, v_res: usize) -> i16 {
    let start_time_dp = time::Instant::now();
    println!("DP called!");
    // TODO: check that no max_speed is larger than GLOBAL_V_MAX, or at least check that it will be clamped automatically by discretize_v
    // maybe don't throw an error in this case, but print that it will be clamped to GLOBAL_V_MAX and do that

    // ==== PRELIMINARIES AND DEFINITIONS =================

    // set moment bounds, including rho_rot
    let (min_moment, max_moment) = (-GLOBAL_MOM_MAX * vehicle.rho_rot, GLOBAL_MOM_MAX * vehicle.rho_rot);

    let route_resistances: Vec<Acceleration> = route.slopes.iter().map(|&slope| route_res(slope, vehicle.roll_res_coeff)).collect();
    
    let mut max_speeds_discretized: Vec<usize> = route.max_speeds.iter().map(|&max_speed| discretize_v(max_speed, None, GLOBAL_V_MAX, v_res)).collect();
    
    // ensure that vehicle (nearly) stops at the end
    let max_allowed_end_speed = Velocity::new::<meter_per_second>(2.0);
    let max_allowed_end_speed_discretized = discretize_v(max_allowed_end_speed, None, GLOBAL_V_MAX, v_res);
    max_speeds_discretized.push(max_allowed_end_speed_discretized);
    
    let num_sections = route.lengths.len();
    let parent_uninit = usize::MAX;

    // create matrices to store best paths and their energies
    let mut mat_e_used  = Array3::<AvailableEnergy>::zeros((num_sections + 1, v_res, t_res)); // contains energy of best path to this state found so far
    let mut mat_parents = Array3::<          usize>::zeros((num_sections + 1, v_res, t_res)); // each element is the flattened index of the parent state [t, v]

    // so far, no paths exist yet. So the minimal energy is infinite and all parents uninitialized
    mat_e_used.fill(AvailableEnergy::new::<joule_per_kilogram>(PrefFloat::INFINITY));
    mat_parents.fill(parent_uninit);

    // initialize step 0 with [0, 0] as only populated state and 0 used energy
    mat_e_used[[0, 0, 0]] = AvailableEnergy::new::<joule_per_kilogram>(0.0);
    mat_parents[[0, 0, 0]] = 0;

    // ==== ACTUAL OPTIMIZATION ============================

    // go through route section by section
    for step in 0..num_sections {
        let s = route.lengths[step];
        let route_res = route_resistances[step];
        let max_speed_discretized = std::cmp::min(max_speeds_discretized[step], max_speeds_discretized[step+1]);

        // go through all populated states at the current step
        for state_v in 0..v_res {
            let ekin_curr = v_to_ekin(v_bin_to_mps(state_v, None, GLOBAL_V_MAX, v_res));

            for state_t in 0..t_res {

                // skip unpopulated states
                if mat_parents[[step, state_v, state_t]] == parent_uninit {
                    continue;
                }

                // branch out from populated states
                for v_next_discretized in 0..=max_speed_discretized {
                    
                    let ekin_next = v_to_ekin(v_bin_to_mps(v_next_discretized, None, GLOBAL_V_MAX, v_res));

                    // discard path if vehicle would stand still across the whole section
                    if approx_eq!(PrefFloat, ekin_curr.value, 0.0, ulps=2) && approx_eq!(PrefFloat, ekin_next.value, 0.0, ulps=2) {
                        continue;
                    }

                    // calculate necessary A to reach that next ekin
                    let a_param = retrieve_a_param(s, ekin_curr, ekin_next, vehicle.get_c_param());
                    let mom = a_param + route_res; // mom includes rho_rot

                    // skip if necessary moment is not allowed
                    if mom < min_moment {
                        continue; // try again with next-larger velocity
                    }
                    if mom > max_moment {
                        break; // larger velocities will also exceed max_moment
                    }

                    // add time for the current section to time used so far
                    let time_used_next = delta_t(s, a_param, vehicle.get_c_param(), ekin_curr) + time_bin_to_seconds(state_t, None, max_time, t_res);

                    // discard path if forbidden
                    if time_used_next > max_time {
                        continue; // try again with next-larger velocity
                    }

                    // round up time_used_next into bins
                    let time_used_next_discretized = discretize_time(time_used_next, None, max_time, t_res);

                    // add energy used on current section to energy used so far
                    let energy_used_next = mat_e_used[[step, state_v, state_t]] + energy_used(s, mom, vehicle.rec_eff) / vehicle.rho_rot;

                    // if current path to the reached state is optimal, replace parent of reached state by current path
                    if energy_used_next < mat_e_used[[step+1, v_next_discretized, time_used_next_discretized]] {
                        // set current path as new optimal parent
                        mat_parents[[step+1, v_next_discretized, time_used_next_discretized]] = state_v * t_res + state_t; // calculate index of parent
                        // set current used energy as new optimal used energy
                        mat_e_used[[step+1, v_next_discretized, time_used_next_discretized]] = energy_used_next;
                    }
                }
            }
        }
        println!("{}% finished", (step + 1) * 100 / num_sections);
    }

    // ==== RETRIEVAL OF BEST PATH =============================

    // TODO: implement return of error if no path was found

    let (_, v_opt, t_opt) = mat_e_used.select(Axis(0), &[mat_e_used.shape()[0] - 1]).argmin().unwrap(); // TODO: Error handling instead of unwrap!
    let minimal_energy = mat_e_used[[mat_e_used.shape()[0] - 1, v_opt, t_opt]]; // mat_e_used.select(Axis(0), &[mat_e_used.shape()[0] - 1]).min().unwrap();
    println!("\nv_opt={:?}, t_opt={:?}", v_opt, t_opt);
    println!("minimal_energy= {:?}", minimal_energy);

    let elapsed_time = start_time_dp.elapsed();
    println!("Running dp_optim() took {} ms", elapsed_time.as_millis());
    println!("OUTPUT = {:?}", num_sections);
    0
}