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
    x add utils functions, e.g. max_s() are not used but helpful for understanding
    x include slope into c_param to account for longer distance if slope is higher? -> don't include it, would be complicated to differentiate between C and s
    x cleanup print statements
    x go through TODOs
    x put custom errors in their own file "error.rs"
    o try out clever splitting of route into sections such that maximum acceleration can be used
    o add calculation of realistic schedule for comparison (with maximum moments, but capped max velocity)
    x add splitting function for routes or repeats/splits/etc. argument to load_route()
    x add truck as an example vehicle in vehicle1.csv
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
                            1.1,
                            0.3,
                            Energy::new::<kilowatt_hour>(65.0),
                            Mass::new::<kilogram>(2000.0),
                            Area::new::<square_meter>(2.0),
                            0.3);

    // load vehicles
    let vhcls = load_vehicles("../vehicle1.csv").unwrap();
    let car   = &vhcls[0];
    let truck = &vhcls[1];
    
    // load route
    let route1 = load_route("routes/route1.csv").unwrap();
    let route1_res2 = load_route("routes/route1_res2.csv").unwrap();

    // // set optimization parameters
    // let max_time = Time::new::<second>(1175.0);
    // let time_res = 2000;
    // let v_res    = 201;
    // let v_0      = Velocity::new::<kilometer_per_hour>(38.0);

    // // run energy optimization with given time budget
    // let (optimal_energy, optimal_schedule_e) = optim_energy(&route1, &car1, max_time, time_res, v_res, Some(v_0), None, None).unwrap();
    // println!("optimal_schedule_e:\n{}", optimal_schedule_e);
    // let _ = optimal_schedule_e.save("results/route1_result_e_inversed");

    // // set optimization parameters
    // let e_res = 8000;
    // let soc   = Ratio::new::<percent>(5.0);

    // // run time optimization with given energy budget
    // let (optimal_time, optimal_schedule_t) = optim_time(&route1, &car1, soc, e_res, v_res, Some(v_0), None).unwrap();
    // println!("optimal_schedule_t:\n{}", optimal_schedule_t);
    // let _ = optimal_schedule_t.save("results/route1_result_t");

    // example for optimizing the whole list of vehicles on a route

    // if different optimization parameters are wanted for different vehicles, store them in a vec
    let socs = vec![Ratio::new::<percent>(5.0),
               Ratio::new::<percent>(7.0)];

    for (i, vhcl) in vhcls.iter().enumerate() {
        println!("{}: {:?}", i, vhcl);
        let soc   = socs[i]; // retrieve parameter for current vehicle
        let e_res = 8000;
        let v_res = 201;
        let v_0   = Velocity::new::<kilometer_per_hour>(38.0);
        let (optimal_time, optimal_schedule_t) = optim_time(&route1, &vhcl, soc, e_res, v_res, Some(v_0), None).unwrap();
        optimal_schedule_t.save(&format!("results/route1_vhcl{}", i));
    }

    Ok(())
}