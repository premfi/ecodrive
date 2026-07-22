/// main file, including examples how to call ecodrive optimization functions
/// edit this file to run your desired optimization

mod ecodrive;
use ecodrive::*;

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

    // Instead of manually defining vehicles, they may be imported from a .csv file and stored in a list. See "load vehicles" below.
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
    // let route1_res3 = route1.partition(3);

    // ==== EXAMPLES FOR OPTIMIZING SINGLE VEHICLE ======================

    // set optimization parameters
    let max_time = Time::new::<second>(1200.0);
    let time_res = 2000;
    let v_res    = 201;
    let v_0      = Velocity::new::<kilometer_per_hour>(0.0);

    // run energy optimization with given time budget
    let (optimal_energy, optimal_schedule_e) = optim_energy(&route1, &car1, max_time, time_res, v_res, Some(v_0), None, None).unwrap();
    println!("optimal_schedule_e:\n{}", optimal_schedule_e);
    let _ = optimal_schedule_e.save("results/route1_result_e");

    // set optimization parameters
    let e_res = 8000;
    let soc   = Ratio::new::<percent>(80.0);

    // run time optimization with given energy budget
    // let (optimal_time, optimal_schedule_t) = optim_time(&route1, &car1, soc, e_res, v_res, Some(v_0), None).unwrap();
    // println!("optimal_schedule_t:\n{}", optimal_schedule_t);
    // let _ = optimal_schedule_t.save("results/route1_result_t_80pct");

    // // ==== EXAMPLE FOR OPTIMIZING A WHOLE LIST OF VEHICLES ==============

    // // if different optimization parameters are wanted for different vehicles, store them in a vec
    // let socs = vec![Ratio::new::<percent>(5.0),
    //            Ratio::new::<percent>(7.0)];

    // for (i, vhcl) in vhcls.iter().enumerate() {
    //     println!("{}: {:?}", i, vhcl);
    //     let soc   = socs[i]; // retrieve parameter for current vehicle
    //     let e_res = 12000;
    //     let v_res = 201;
    //     let v_0   = Velocity::new::<kilometer_per_hour>(0.0);
    //     match optim_time(&route1_res16, &vhcl, soc, e_res, v_res, Some(v_0), None) {
    //         Ok((optimal_time, optimal_schedule_t)) => optimal_schedule_t.save(&format!("results/route1_res16_vhcl{}", i))?,
    //         Err(e) => println!("optimization number {} failed with: {:?}", i, e),
    //     }

    // }

    Ok(())
}