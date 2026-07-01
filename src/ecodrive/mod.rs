use uom::si::{acceleration::meter_per_second_squared,
            available_energy::joule_per_kilogram,
            energy::kilowatt_hour,
            velocity::{meter_per_second, kilometer_per_hour},
            time::second};
use uom::fmt::DisplayStyle::Abbreviation;
use float_cmp::approx_eq;

use std::fmt::{Debug, Display};

pub mod config;
pub use config::PrefFloat;
use config::uom_si_preffloat::{Acceleration, Length, Velocity, Ratio, AvailableEnergy, Energy, Time};
use config::floats;

pub mod constants;
use constants::{GRAVITY_OF_EARTH, GLOBAL_V_MAX, GLOBAL_MOM_MAX};

mod vehicle;
pub use vehicle::{Vehicle, load_vehicles};

mod route;
pub use route::{Route, load_route};

mod schedule;
pub use schedule::DrivingSchedule;

use uom::typenum::{N1, Z0};
pub type PerLength = uom::si::Quantity<uom::si::ISQ<N1, Z0, Z0, Z0, Z0, Z0, Z0>,
                                            uom::si::SI<PrefFloat>, PrefFloat>; // [1/m]

use ndarray::{Array3, Axis};
use ndarray_stats::QuantileExt;

#[derive(Debug)]
pub enum DPError {
    ImpossibleTask,
    NoPathFound,
}

impl std::error::Error for DPError {}

impl Display for DPError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            DPError::ImpossibleTask => write!(f, "task impossible to solve with given parameters"),
            DPError::NoPathFound => write!(f, "no valid path found"),
        }
    }
}

#[derive(Debug)]
pub enum ValueError<T> {
    NegativeValue(T),
}

impl<T: Debug + Display + Copy> std::error::Error for ValueError<T> {}

impl<T: Debug + Copy> Display for ValueError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ValueError::NegativeValue(val) => write!(f, "negative value not allowed: {:?}", val),
        }
    }
}

fn e_kin(s: Length, a_param: Acceleration, c_param: PerLength, ekin_0: AvailableEnergy) -> AvailableEnergy {
    (a_param / c_param) + (ekin_0 - (a_param / c_param)) * (-c_param * s).exp()
}

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
    let a_param = c_param * (ekin_s - ekin_0 * PrefFloat::from((-c_param * s).exp())) / (1.0 - PrefFloat::from((-c_param * s).exp()));
    a_param
}

/// Calculates the time needed for a section.
pub fn delta_t(s: Length, a_param: Acceleration, c_param: PerLength, ekin_0: AvailableEnergy) -> Time {
   
    // A = 0
    if approx_eq!(PrefFloat, a_param.value, 0.0, epsilon=1e-9, ulps=100) {

        // A = 0, ekin_0 > 0
        if ekin_0 > AvailableEnergy::new::<joule_per_kilogram>(0.0) {
            return PrefFloat::sqrt(2.0) / (c_param * ekin_0.sqrt()) * (PrefFloat::from(c_param * s / 2.0).exp() - 1.0);
        }
        
        // A = 0, no ekin_0 -> impossible to reach `s`
        else {
            // println!("case 1.1");
            return Time::new::<second>(floats::INFINITY);
        }
    }
    
    // positive A
    else if a_param > Acceleration::new::<meter_per_second_squared>(0.0) {
        // constant speed
        if approx_eq!(PrefFloat, ((c_param / a_param) * ekin_0).into(), 1.0, ulps=100) {
            // println!("case 2.1");
            return s / (2.0 * ekin_0).sqrt();
        } 
        // higher or lower end velocity, but always > 0
        else {
            // approximation for numerical stability
            if PrefFloat::from(s * c_param) > 12.0 {
                // println!("approximation used");
                let max_stable_s = 12.0 / c_param;
                let y_axis_offset = delta_t(max_stable_s, a_param, c_param, ekin_0);
                let m = c_param / (2.0 * a_param * c_param).sqrt();
                return m * (s - max_stable_s) + y_axis_offset;
            } 
            // actual formula
            else {
                // println!("case 2.2");
                let x: PrefFloat = (1.0 + (PrefFloat::from((c_param / a_param) * ekin_0) - 1.0) * PrefFloat::from((-c_param * s).exp())).sqrt();
                let y: PrefFloat = ((c_param / a_param) * ekin_0).sqrt().into();
                return PrefFloat::sqrt(2.0) * ((x - y) / (1.0 - x * y)).atanh() / (a_param * c_param).sqrt();
            }
        }
    }

    // negative A
    else {
        // end speed will be exactly zero
        if approx_eq!(PrefFloat, (-c_param * ekin_0 / (PrefFloat::from((c_param * s).exp()) - 1.0)).value, a_param.value, epsilon=1e-5, ulps=100) {
            // println!("case 3.1");
            let x: PrefFloat = 0.0;
            let y: PrefFloat = (-(c_param / a_param) * ekin_0).sqrt().into();
            return (-2.0 / (a_param * c_param)).sqrt() * ((y - x) / (1.0 + x * y)).atan();
        }
        // end speed larger than zero
        else if (-c_param * ekin_0 / (PrefFloat::from(c_param * s).exp() - 1.0)) < a_param {
            // println!("case 3.2");
            let x: PrefFloat = (-1.0 - (PrefFloat::from((c_param / a_param) * ekin_0) - 1.0) * PrefFloat::from((-c_param * s).exp())).sqrt();
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

pub fn ekin_to_v(ekin: AvailableEnergy) -> Result<Velocity, ValueError<AvailableEnergy>> {
    if approx_eq!(PrefFloat, ekin.value, 0.0, ulps=100) {
        return Ok(Velocity::new::<meter_per_second>(0.0));
    }
    if ekin < AvailableEnergy::new::<joule_per_kilogram>(0.0) {
        return Err(ValueError::NegativeValue(ekin));
    }
    Ok((2.0 * ekin).sqrt())
}

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

/// Assings `e` to its corresponding bin from [0, `num-1`]. Bins are linearly spaced between `min` and `max`.
/// Positive values out of range are clamped into the edge bins. Panics for negative `e` inputs.
fn discretize_energy(e: AvailableEnergy, min: Option<AvailableEnergy>, max: AvailableEnergy, num: usize) -> usize {
    assert!(e >= AvailableEnergy::new::<joule_per_kilogram>(0.0), "`e` must be non-negative! e={:?}", e);

    let min = min.unwrap_or(AvailableEnergy::new::<joule_per_kilogram>(0.0));
    assert!(min <= max, "`min` must not be larger than `max`! min={:?}, max={:?}", min, max);

    if e > max {
        return num - 1;
    }

    if e < min {
        return 0;
    }

    let stepsize = (max - min) / (num - 1) as PrefFloat;
    let bin = PrefFloat::from((e - min) / stepsize).floor() as usize;
    
    std::cmp::min(bin, num - 1)
}

/// Translates energy bin to corresponding AvailableEnergy value.
fn e_bin_to_J_p_kg(bin: usize, min: Option<AvailableEnergy>, max: AvailableEnergy, num: usize) -> AvailableEnergy {
    let min = min.unwrap_or(AvailableEnergy::new::<joule_per_kilogram>(0.0));
    let stepsize = (max - min) / (num - 1) as PrefFloat;

    stepsize * (bin as PrefFloat) + min
}

/// Assigns `v` to its corresponding bin from [0, `num-1`]. Bins are linearly spaced between `min` and `max`.
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

/// Optimizes energy use for a vehicle on a route, given a time budget.
/// 
/// parameters
/// ----------
/// route: route on which the optimization takes place
/// vehicle: vehicle traversing the route
/// max_time: maximum time budget. Result will either lie within this budget or abort
/// t_res: resolution of the time discretization
/// v_res: resolution of the velocity discretization
/// v_0 (optional): initial velocity, defaults to 0.0mps if None
/// v_end (optional): range for the end velocity, no constraints imposed if None
/// e_headroom (optional): maximum energy that can be stored in the battery on top of the initial charge, no constraint imposed if None
/// 
/// returns
/// -------
/// on success: (optimal energy, optimal schedule)
/// possible errors: ImpossibleTask or NoPathFound
pub fn optim_energy(route: &Route, vehicle: &Vehicle, max_time: Time, t_res: usize, v_res: usize, v_0: Option<Velocity>, v_end: Option<(Velocity, Velocity)>, e_headroom: Option<Energy>) -> Result<(Energy, DrivingSchedule), DPError> {
    let start_time_dp = std::time::Instant::now();
    println!("optim_energy: starting optimization...");

    // ==== INPUT CHECKS =================================

    // return ImpossibleTask if its impossible to traverse the route in the given time
    let min_theor_time = route.lengths.iter().zip(route.max_speeds.iter()).fold(Time::new::<second>(0.0), |acc , (&s, &v_max)| acc + (s / v_max));
    if min_theor_time > max_time {
        return Err(DPError::ImpossibleTask);
    }

    // ==== PRELIMINARIES AND DEFINITIONS =================

    // set moment bounds, including rho_rot
    let (min_moment, max_moment) = (-GLOBAL_MOM_MAX * vehicle.rho_rot, GLOBAL_MOM_MAX * vehicle.rho_rot);

    let route_resistances: Vec<Acceleration> = route.slopes.iter().zip(route.roll_res_factors.iter())
                                                .map(|(&slope, &roll_res_fac)| route_res(slope, vehicle.roll_res_coeff * roll_res_fac))
                                                .collect();
    
    // set default value for e_headroom, if None was given
    let min_allowed_energy: AvailableEnergy = match e_headroom {
        Some(e) => -e.abs() / vehicle.mass,
        None => AvailableEnergy::new::<joule_per_kilogram>(-floats::INFINITY),
    };  // min_allowed_energy now holds the minimal allowed value for the used energy

    // set default values for min/max allowed end speeds, if None were given
    let (min_allowed_end_speed, max_allowed_end_speed) = v_end.unwrap_or((Velocity::new::<meter_per_second>(0.0), GLOBAL_V_MAX));

    let mut min_speeds_discretized: Vec<usize> = route.min_speeds.iter().map(|&min_speed| discretize_v(min_speed, None, GLOBAL_V_MAX, v_res)).collect();
    let min_allowed_end_speed_discretized = discretize_v(min_allowed_end_speed, None, GLOBAL_V_MAX, v_res);
    min_speeds_discretized.push(min_allowed_end_speed_discretized);

    let mut max_speeds_discretized: Vec<usize> = route.max_speeds.iter().map(|&max_speed| discretize_v(max_speed, None, GLOBAL_V_MAX, v_res)).collect();
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

    let v_0_idx = discretize_v(v_0.unwrap_or(Velocity::new::<meter_per_second>(0.0)), None, GLOBAL_V_MAX, v_res);

    // initialize step 0 with [v0_idx, 0] as only populated state and 0 used energy
    mat_e_used[[0, v_0_idx, 0]] = AvailableEnergy::new::<joule_per_kilogram>(0.0);
    mat_parents[[0, v_0_idx, 0]] = 0;

    // ==== ACTUAL OPTIMIZATION ============================

    // go through route section by section
    for step in 0..num_sections {
        let s = route.lengths[step];
        let route_res = route_resistances[step];
        let c_param = vehicle.get_c_param();
        let min_speed_discretized = std::cmp::max(min_speeds_discretized[step], min_speeds_discretized[step+1]);
        let max_speed_discretized = std::cmp::min(max_speeds_discretized[step], max_speeds_discretized[step+1]);

        // go through all populated states at the current step
        for state_v in 0..v_res {
            let ekin_curr = v_to_ekin(v_bin_to_mps(state_v, None, GLOBAL_V_MAX, v_res));

            // let min_reachable_v_next = discretize_v(ekin_to_v(
            //                             e_kin(s, min_moment - route_res, c_param, ekin_curr)).unwrap_or(Velocity::new::<meter_per_second>(0.0)),
            //                             None, GLOBAL_V_MAX, v_res);
            let max_reachable_v_next = discretize_v(ekin_to_v(
                                        e_kin(s, max_moment - route_res, c_param, ekin_curr)).unwrap_or(GLOBAL_V_MAX),
                                        None, GLOBAL_V_MAX, v_res);

            // start either from lowest reachable or lowest allowed velocity
            // let min_v_next = std::cmp::max(min_reachable_v_next, min_speed_discretized);
            // start either from highest reachable or highest allowed velocity
            let max_v_next = std::cmp::min(max_reachable_v_next, max_speed_discretized);

            for state_t in 0..t_res {

                // skip unpopulated states
                if mat_parents[[step, state_v, state_t]] == parent_uninit {
                    continue;
                }

                // branch out from populated states
                // for v_next_discretized in min_v_next..=max_speed_discretized {
                for v_next_discretized in (min_speed_discretized..=max_v_next).rev() {
                    
                    let ekin_next = v_to_ekin(v_bin_to_mps(v_next_discretized, None, GLOBAL_V_MAX, v_res));

                    // discard path if vehicle would stand still across the whole section
                    if approx_eq!(PrefFloat, ekin_curr.value, 0.0, ulps=100) && approx_eq!(PrefFloat, ekin_next.value, 0.0, ulps=100) {
                        continue;
                    }

                    // calculate necessary A to reach that next ekin
                    let a_param = retrieve_a_param(s, ekin_curr, ekin_next, c_param);
                    let mom = a_param + route_res; // mom includes rho_rot

                    // skip if necessary moment is not allowed
                    if mom < min_moment {
                        // continue; // try again with next-larger velocity
                        break; // lower velocities will also lead to too low moment
                    }
                    if mom > max_moment {
                        // break; // larger velocities will also exceed max_moment
                        continue; // try again with next-lower velocity
                    }

                    // add time for the current section to time used so far
                    let time_used_next = delta_t(s, a_param, c_param, ekin_curr) + time_bin_to_seconds(state_t, None, max_time, t_res);

                    // discard path if forbidden
                    if time_used_next > max_time {
                        // continue; // try again with next-larger velocity
                        break; // next-lower velocity will also exceed max_time
                    }

                    // round up time_used_next into bins
                    let time_used_next_discretized = discretize_time(time_used_next, None, max_time, t_res);

                    // add energy used on current section to energy used so far
                    let energy_used_next = mat_e_used[[step, state_v, state_t]] + energy_used(s, mom, vehicle.rec_eff) / vehicle.rho_rot;
                    
                    // battery cannot store more additional energy than specified by e_headroom
                    let energy_used_next = energy_used_next.max(min_allowed_energy);

                    // if current path to the reached state is optimal, replace parent of reached state by current path
                    if energy_used_next < mat_e_used[[step+1, v_next_discretized, time_used_next_discretized]] {
                        // set current path as new optimal parent
                        mat_parents[[step+1, v_next_discretized, time_used_next_discretized]] = state_v * t_res + state_t; // calculate flattened index of parent
                        // set current used energy as new optimal used energy
                        mat_e_used[[step+1, v_next_discretized, time_used_next_discretized]] = energy_used_next;
                    }
                }
            }
        }
        println!("{}% finished", (step + 1) * 100 / num_sections);
    }

    // ==== RETRIEVAL OF BEST END STATE ==========================

    let (_, v_opt_end, t_opt_end) = mat_e_used.select(Axis(0), &[mat_e_used.shape()[0] - 1]).argmin().expect("even if no path was found, argmin should yield a value");
    let minimal_energy = mat_e_used[[mat_e_used.shape()[0] - 1, v_opt_end, t_opt_end]] * vehicle.mass;
    
    if !(minimal_energy < Energy::new::<kilowatt_hour>(PrefFloat::INFINITY)) {
        return Err(DPError::NoPathFound);
    }

    println!("minimal_energy: {:.4}", minimal_energy.into_format_args(kilowatt_hour, Abbreviation));

    // ==== BACKTRACKING ALONG OPTIMAL PATH ======================

    // initialize optimal schedule with Infinity
    let mut optimal_schedule = DrivingSchedule {times: vec![Time::new::<second>(PrefFloat::INFINITY); num_sections + 1],
                                                speeds: vec![Velocity::new::<meter_per_second>(PrefFloat::INFINITY); num_sections + 1]};

    let mut parent_flat;

    // optimal path ends in optimal state, so backtracking starts with it
    let mut v_opt_curr = v_opt_end;
    let mut t_opt_curr = t_opt_end;

    // backtrack along optimal path, starting at the end
    for step in (0..=num_sections).rev() {
        // save v and t of current step to optimal schedule
        optimal_schedule.speeds[step] = v_bin_to_mps(v_opt_curr, None, GLOBAL_V_MAX, v_res);
        optimal_schedule.times[step] = time_bin_to_seconds(t_opt_curr, None, max_time, t_res);

        // get parent index of current state
        parent_flat = mat_parents[[step, v_opt_curr, t_opt_curr]];

        // retrieve v and t state from parent index
        v_opt_curr = parent_flat / t_res; // integer division, truncating decimal part
        t_opt_curr = parent_flat % t_res;
    }

    let elapsed_time = start_time_dp.elapsed();
    println!("Running optim_energy() took {} ms", elapsed_time.as_millis());

    Ok((minimal_energy, optimal_schedule))
}


/// Optimizes used time for a vehicle on a route, given an energy budget.
/// 
/// parameters
/// ----------
/// route: route on which the optimization takes place
/// vehicle: vehicle traversing the route
/// soc: state of charge of the battery at the beginning of the route
/// e_res: resolution of the energy discretization
/// v_res: resolution of the velocity discretization
/// v_0 (optional): initial velocity, defaults to 0.0mps if None
/// v_end (optional): range for the end velocity, no constraints imposed if None^
/// 
/// returns
/// -------
/// on success: (optimal time, optimal schedule)
/// possibe errors: NoPathFound
pub fn optim_time(route: &Route, vehicle: &Vehicle, soc: Ratio, e_res: usize, v_res: usize, v_0: Option<Velocity>, v_end: Option<(Velocity, Velocity)>) -> Result<(Time, DrivingSchedule), DPError> {
    let start_time_dp = std::time::Instant::now();
    println!("optim_time: starting optimization...");

    // ==== PRELIMINARIES AND DEFINITIONS =================

    let bat_cap = vehicle.bat_cap / vehicle.mass; // battery capacity
    let e_0: AvailableEnergy = soc * bat_cap; // initial energy content

    // set moment bounds, including rho_rot
    let (min_moment, max_moment) = (-GLOBAL_MOM_MAX * vehicle.rho_rot, GLOBAL_MOM_MAX * vehicle.rho_rot);

    let route_resistances: Vec<Acceleration> = route.slopes.iter().zip(route.roll_res_factors.iter())
                                                .map(|(&slope, &roll_res_fac)| route_res(slope, vehicle.roll_res_coeff * roll_res_fac))
                                                .collect();

    // set default values for min/max allowed end speeds, if None were given
    let (min_allowed_end_speed, max_allowed_end_speed) = v_end.unwrap_or((Velocity::new::<meter_per_second>(0.0), GLOBAL_V_MAX));

    let mut min_speeds_discretized: Vec<usize> = route.min_speeds.iter().map(|&min_speed| discretize_v(min_speed, None, GLOBAL_V_MAX, v_res)).collect();
    let min_allowed_end_speed_discretized = discretize_v(min_allowed_end_speed, None, GLOBAL_V_MAX, v_res);
    min_speeds_discretized.push(min_allowed_end_speed_discretized);

    let mut max_speeds_discretized: Vec<usize> = route.max_speeds.iter().map(|&max_speed| discretize_v(max_speed, None, GLOBAL_V_MAX, v_res)).collect();
    let max_allowed_end_speed_discretized = discretize_v(max_allowed_end_speed, None, GLOBAL_V_MAX, v_res);
    max_speeds_discretized.push(max_allowed_end_speed_discretized);
    
    let num_sections = route.lengths.len();
    let parent_uninit = usize::MAX;

    // create matrices to store best paths and their used times
    let mut mat_t_used  = Array3::< Time>::zeros((num_sections + 1, v_res, e_res)); // contains time of best path to this state found so far
    let mut mat_parents = Array3::<usize>::zeros((num_sections + 1, v_res, e_res)); // each element is the flattened index of the parent state [t, v]

    // so far, no paths exist yet. So the minimal time is infinite and all parents uninitialized
    mat_t_used.fill(Time::new::<second>(PrefFloat::INFINITY));
    mat_parents.fill(parent_uninit);

    let v_0_idx = discretize_v(v_0.unwrap_or(Velocity::new::<meter_per_second>(0.0)), None, GLOBAL_V_MAX, v_res);
    let e_0_idx = discretize_energy(e_0, None, bat_cap, e_res);

    // initialize step 0 with [v0_idx, e_0_idx] as only populated state and 0 used time
    mat_t_used[[0, v_0_idx, e_0_idx]] = Time::new::<second>(0.0);
    mat_parents[[0, v_0_idx, e_0_idx]] = 0;

    // ==== ACTUAL OPTIMIZATION ============================

    // go through route section by section
    for step in 0..num_sections {
        let s = route.lengths[step];
        let route_res = route_resistances[step];
        let c_param = vehicle.get_c_param();
        let min_speed_discretized = std::cmp::max(min_speeds_discretized[step], min_speeds_discretized[step+1]);
        let max_speed_discretized = std::cmp::min(max_speeds_discretized[step], max_speeds_discretized[step+1]);

        // go through all populated states at the current step
        for state_v in 0..v_res {
            let ekin_curr = v_to_ekin(v_bin_to_mps(state_v, None, GLOBAL_V_MAX, v_res));

            let min_reachable_v_next = discretize_v(ekin_to_v(
                                        e_kin(s, min_moment - route_res, c_param, ekin_curr)).unwrap_or(Velocity::new::<meter_per_second>(0.0)),
                                        None, GLOBAL_V_MAX, v_res);

            // start either from lowest reachable or lowest allowed velocity
            let min_v_next = std::cmp::max(min_reachable_v_next, min_speed_discretized);

            for state_e in 0..e_res {
                
                // skip unpopulated states
                if mat_parents[[step, state_v, state_e]] == parent_uninit {
                    continue;
                }

                // branch out from populated states
                for v_next_discretized in min_v_next..=max_speed_discretized {
                    let ekin_next = v_to_ekin(v_bin_to_mps(v_next_discretized, None, GLOBAL_V_MAX, v_res));

                    // discard path if vehicle would stand still across the whole section
                    if approx_eq!(PrefFloat, ekin_curr.value, 0.0, ulps=100) && approx_eq!(PrefFloat, ekin_next.value, 0.0, ulps=100) {
                        continue;
                    }

                    // calculate necessary A to reach that next ekin
                    let a_param = retrieve_a_param(s, ekin_curr, ekin_next, c_param);
                    let mom = a_param + route_res; // mom includes rho_rot

                    // skip if necessary moment is not allowed
                    if mom < min_moment {
                        continue; // try again with next-larger velocity
                    }
                    if mom > max_moment {
                        break; // larger velocities will also exceed max_moment
                    }

                    // subtract energy used for the current section from energy in battery
                    let e_used_next = e_bin_to_J_p_kg(state_e, None, bat_cap, e_res) - energy_used(s, mom, vehicle.rec_eff) / vehicle.rho_rot;

                    // discard path if forbidden
                    if e_used_next < AvailableEnergy::new::<joule_per_kilogram>(0.0) {
                        break; // larger velocities will also completely empty the battery
                    }

                    // discretize and clamp energy to a maximum of bat_cap
                    let e_used_next_discretized = discretize_energy(e_used_next, None, bat_cap, e_res);

                    // add time used on current section to time used so far
                    let time_used_next = mat_t_used[[step, state_v, state_e]] + delta_t(s, a_param, c_param, ekin_curr);

                    // if current path to the reached state is optimal, replace parent of reached state by current path
                    if time_used_next < mat_t_used[[step+1, v_next_discretized, e_used_next_discretized]] {
                        // set current path as new optimal parent
                        mat_parents[[step+1, v_next_discretized, e_used_next_discretized]] = state_v * e_res + state_e; // calculate flattened index of parent
                        // set current used time as new optimal used time
                        mat_t_used[[step+1, v_next_discretized, e_used_next_discretized]] = time_used_next;
                    }
                }
            }
        }
        println!("{}% finished", (step + 1) * 100 / num_sections);
    }

    // ==== RETRIEVAL OF BEST END STATE ==========================

    let (_, v_opt_end, e_opt_end) = mat_t_used.select(Axis(0), &[mat_t_used.shape()[0] - 1]).argmin().expect("even if no path was found, argmin should yield a value");
    let minimal_time = mat_t_used[[mat_t_used.shape()[0] - 1, v_opt_end, e_opt_end]];

    if !(minimal_time < Time::new::<second>(PrefFloat::INFINITY)) {
        return Err(DPError::NoPathFound);
    }

    println!("minimal_time: {:.4}", minimal_time.into_format_args(second, Abbreviation));

    // ==== BACKTRACKING ALONG OPTIMAL PATH ======================

    // initialize optimal schedule with Infinity
    let mut optimal_schedule = DrivingSchedule {times: vec![Time::new::<second>(PrefFloat::INFINITY); num_sections + 1],
                                                speeds: vec![Velocity::new::<meter_per_second>(PrefFloat::INFINITY); num_sections + 1]};

    let mut parent_flat;

    // optimal path ends in optimal state, so backtracking starts with it
    let mut v_opt_curr = v_opt_end;
    let mut e_opt_curr = e_opt_end;

    // backtrack along optimal path, starting at the end
    for step in (0..=num_sections).rev() {
        // save v and t of current step to optimal schedule
        optimal_schedule.speeds[step] = v_bin_to_mps(v_opt_curr, None, GLOBAL_V_MAX, v_res);
        optimal_schedule.times[step] = mat_t_used[[step, v_opt_curr, e_opt_curr]];

        // get parent index of current state
        parent_flat = mat_parents[[step, v_opt_curr, e_opt_curr]];

        // retrieve v and e state from parent index
        v_opt_curr = parent_flat / e_res; // integer division, truncating decimal part
        e_opt_curr = parent_flat % e_res;
    }

    let elapsed_time = start_time_dp.elapsed();
    println!("Running optim_time() took {} ms", elapsed_time.as_millis());

    Ok((minimal_time, optimal_schedule))
}