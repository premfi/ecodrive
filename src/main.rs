mod ecodrive;
use ecodrive::*;

/* TODO:
    x put constants in own module
    x maybe put settings (like f64/f32) in own module, e.g. use uom::si::f64 as uom_si_preffloat; . Then in e.g. vehicle: use uom_si_preffloat::{Mass, Area};
    x re-export everything from ecodrive, so that only ecodrive needs to be added in main.rs
    x move route related definitions and functions into route.rs
    x create Schedule struct in mod.rs to hold result of dp_optim()
    x add serialization support for Schedule
    x plot and check result of dp_optim() on route3_res8
    x replace const::E::powf() by .exp()
    x implement ImpossibleTaskError to return if given time is too short
    x implement NoPathFoundError
    x retrieve best path in dp_optim() and return it
    x start with lowest reachable velocity
    o add argument for initial velocity. Set another entry (according to discretize(v0)) of mat_parents and mat_e_used to 0 for this
    o introduce minimum velocity (can also help optimization performance)
    o try out clever splitting of route into sections such that maximum acceleration can be used
    o add splitting function for routes or repeats/splits/etc. argument to load_route()
    o write inverse optimization with fixed energy budget and time to be optimized
    o add function to calculate used energy and actual time from given DrivingSchedule
    o add utils functions, e.g. max_s() are not used but helpful for understanding
    o clean up plotting functions and design interface for loading schedules from file to python
    o maybe add function that takes three paths: route, vehicles and returned schedule(s) and max_time, t_res, v_res that automatically calculates all of them
*/

use config::uom_si_preffloat::{Mass, Area, Length, Ratio, Velocity, Time, AvailableEnergy};
use uom::si::{mass::kilogram,
            area::square_meter, 
            length::meter, 
            ratio::percent, 
            velocity::{kilometer_per_hour, meter_per_second},
            time::second,
            available_energy::joule_per_kilogram};

use ndarray::Array3;

fn main() -> Result<(), std::io::Error> {    
    println!("Hello, world!");

    /* TODO: Instead of manually defining vehicles, they may be imported from a .csv file, stored in a list, and then executed one by one */
    let car1 = Vehicle::new(0.01,
                                0.3,
                                1.1,
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

    let route0 = Route {lengths: lengths.clone(), slopes: slopes.clone(), min_speeds: vec![Velocity::new::<kilometer_per_hour>(0.0); 4], max_speeds: max_speeds.clone()};

    let route3_res8 = Route {lengths: vec![Length::new::<meter>(50.0); 40],
                            slopes: vec![Ratio::new::<percent>(0.0); 40],
                            min_speeds: vec![Velocity::new::<kilometer_per_hour>(0.0); 40],
                            max_speeds: vec![Velocity::new::<kilometer_per_hour>(100.0); 40]};
    
    let max_time = Time::new::<second>(200.0);
    let time_res = 2000;
    let v_res = 201;

    let (optimal_energy, optimal_schedule) = dp_optim(&route0, &car1, max_time, time_res, v_res).unwrap();
    let _ = optimal_schedule.save("route0_result");
    println!("DP:\n{}", optimal_schedule);

    let vhcls = load_vehicles("../vehicle1.csv").unwrap();
    let vhcl0 = &vhcls[0];
    let vhcl1 = &vhcls[1];
    println!("vhcl0.get_c_param()={:?}", vhcl0.get_c_param());
    println!("vhcl1.get_c_param()={:?}", vhcl1.get_c_param());

    println!("ekin_to_v: {:?}", ekin_to_v(AvailableEnergy::new::<joule_per_kilogram>(15.0)));
    println!("ekin_to_v: {}", ekin_to_v(AvailableEnergy::new::<joule_per_kilogram>(-5.0)).unwrap_err());

    Ok(())
}