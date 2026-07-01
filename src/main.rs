mod ecodrive;
use ecodrive::*;

/* TODO:
    x put constants in own module
    x maybe put settings (like f64/f32) in own module, e.g. use uom::si::f64 as uom_si_preffloat; . Then in e.g. vehicle: use uom_si_preffloat::{Mass, Area};
    x re-export everything from ecodrive, so that only ecodrive needs to be added in main.rs
    x move route related definitions and functions into route.rs
    x create Schedule struct in mod.rs to hold result of optim_energy()
    x add serialization support for Schedule
    x plot and check result of optim_energy() on route3_res8
    x replace const::E::powf() by .exp()
    x implement ImpossibleTaskError to return if given time is too short
    x implement NoPathFoundError
    x retrieve best path in optim_energy() and return it
    x start with lowest reachable velocity
    x add argument for initial velocity. Set another entry (according to discretize(v0)) of mat_parents and mat_e_used to 0 for this
    x introduce minimum velocity (can also help optimization performance). Use it in discretization? -> Don't use it in discretization, keep that linear and clear
    x write inverse optimization with fixed energy budget and time to be optimized
    x in optim_energy? : add way to include percentage of initial charge, so that it's clear how much more can be loaded into the battery before it's full. Ensure that this way, the discrete energy is always positive (between 0% and 100%) -> don't do this, use optim_time for this use case instead.
    x add rolling resistance factor for each section
    x add air resistance factor for each section? How to balance frontal_area and c_w?
    x check using f32 instead
    x probably remove c_param as a field from vehicle and only have get_c_param() for recalculation every time
    x add function to calculate used energy and actual time from given DrivingSchedule (not necessary as it was correct with optim_energy)
    o add utils functions, e.g. max_s() are not used but helpful for understanding
    o include slope into c_param to account for longer distance if slope is higher?
    o cleanup print statements
    o go through TODOs
    o put custom errors in their own file "error.rs"
    o try out clever splitting of route into sections such that maximum acceleration can be used
    o add splitting function for routes or repeats/splits/etc. argument to load_route()
    o add truck as an example vehicle in vehicle1.csv
    x clean up plotting functions and design interface for loading schedules from file to python
    o add example in main that takes three paths: route, vehicles and returned schedule(s) and max_time, t_res, v_res that automatically calculates all of them
*/

use config::uom_si_preffloat::{Mass, Area, Length, Ratio, Velocity, Time, AvailableEnergy, Energy};
use uom::si::{mass::kilogram,
            area::square_meter, 
            length::meter, 
            ratio::percent, 
            velocity::{kilometer_per_hour, meter_per_second},
            time::second,
            available_energy::joule_per_kilogram,
            energy::kilowatt_hour};

use ndarray::Array3;

fn main() -> Result<(), std::io::Error> {    
    println!("Hello, world!");

    /* TODO: Instead of manually defining vehicles, they may be imported from a .csv file, stored in a list, and then executed one by one */
    let car1 = Vehicle::new(0.01,
                            0.3,
                            1.1,
                            Energy::new::<kilowatt_hour>(65.0),
                            Mass::new::<kilogram>(2000.0),
                            Area::new::<square_meter>(2.0),
                            0.3);

    println!("c={:?}", car1.get_c_param());

    let lengths = vec![Length::new::<meter>(50.0),
                        Length::new::<meter>(100.0),
                        Length::new::<meter>(100.0),
                        Length::new::<meter>(50.0),];

    let slopes = vec![Ratio::new::<percent>(0.0),
                        Ratio::new::<percent>(5.0),
                        Ratio::new::<percent>(-5.0),
                        Ratio::new::<percent>(0.0)];

    let max_speeds = vec![Velocity::new::<kilometer_per_hour>(100.0), // is automatically converted to m/s
                        Velocity::new::<kilometer_per_hour>(100.0),
                        Velocity::new::<kilometer_per_hour>(130.0),
                        Velocity::new::<kilometer_per_hour>(130.0)];
    
    let route_res = route_res(slopes[1], car1.roll_res_coeff);

    let route0 = Route {lengths: lengths.clone(), slopes: slopes.clone(), min_speeds: vec![Velocity::new::<kilometer_per_hour>(0.0); 4], max_speeds: max_speeds.clone(), roll_res_factors: vec![1.0; 4]};

    let route3_res8 = Route {lengths: vec![Length::new::<meter>(50.0); 40],
                            slopes: vec![Ratio::new::<percent>(0.0); 40],
                            min_speeds: vec![Velocity::new::<kilometer_per_hour>(0.0); 40],
                            max_speeds: vec![Velocity::new::<kilometer_per_hour>(100.0); 40],
                            roll_res_factors: vec![1.0; 40]};
    
    let max_time = Time::new::<second>(500.0);
    let time_res = 2000;
    let v_res = 201;

    let route2 = load_route("routes/route2.csv").unwrap();

    let (optimal_energy, optimal_schedule_e) = optim_energy(&route2, &car1, max_time, time_res, v_res, Some(Velocity::new::<kilometer_per_hour>(38.0)), None, None).unwrap();
    let _ = optimal_schedule_e.save("results/route2_result");
    println!("DP:\n{}", optimal_schedule_e);

    let e_res = 2000;
    let (optimal_time, optimal_schedule_t) = optim_time(&route2, &car1, Ratio::new::<percent>(2.0), e_res, v_res, Some(Velocity::new::<kilometer_per_hour>(38.0)), None).unwrap();
    let _ = optimal_schedule_t.save("results/route2_result_t_2pct");

    let vhcls = load_vehicles("../vehicle1.csv").unwrap();
    let vhcl0 = &vhcls[0];
    let vhcl1 = &vhcls[1];

    // let route1 = load_route("routes/route1.csv").unwrap();
    // println!("route1: {:?}", route1);

    Ok(())
}