mod ecodrive;
use ecodrive::*;

/* TODO:
    x put constants in own module
    x maybe put settings (like f64/f32) in own module, e.g. use uom::si::f64 as uom_si_preffloat; . Then in e.g. vehicle: use uom_si_preffloat::{Mass, Area};
    o re-export everything from ecodrive, so that only ecodrive needs to be added in main.rs
*/

use config::uom_si_preffloat::{Mass, Area, Length, Ratio, Velocity, Acceleration, AvailableEnergy, Time};
use uom::si::{mass::kilogram,
            area::square_meter, 
            length::meter, 
            ratio::percent, 
            velocity::{meter_per_second, kilometer_per_hour}, 
            acceleration::meter_per_second_squared, 
            available_energy::joule_per_kilogram,
            time::second};
use float_cmp::approx_eq;


fn main() {    
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
    
    let route0 = Route {lengths, slopes, max_speeds};

    /* println!("g={:?}", GRAVITY_OF_EARTH);
    println!("{:?}", car1.mass * car1.frontal_area);
    let g = uom::si::acceleration::meter_per_second_squared{};
    let test_mom = Acceleration::new::<meter_per_second_squared>(-2.8);
    let test_s = Length::new::<meter>(50.0);
    let test_a_param = Acceleration::new::<meter_per_second_squared>(-0.0);
    let test_ekin_0 = AvailableEnergy::new::<joule_per_kilogram>(0.0);
    let test_ekin_s = AvailableEnergy::new::<joule_per_kilogram>(75.0);
    println!("delta t = {:?}\n", delta_t(test_s, test_a_param, car1.get_c_param(), test_ekin_0));

    let test_a_retrieved = retrieve_a_param(test_s, test_ekin_0, test_ekin_s, car1.get_c_param());
    println!("retrieved A = {:?}", test_a_retrieved);
    */

    /*
    println!("energy_used = {:?}", energy_used(test_s, test_mom, car1.rec_eff));
    let test_v = Velocity::new::<kilometer_per_hour>(55.0);
    println!("velocity = {:?}", v_to_ekin(test_v));
    */

    /*
    // test time discretization
    let t = 50;
    let test_time = Time::new::<second>((2 * t) as PrefFloat);
    let (max, num) = (Time::new::<second>(200.0), 2000);
    let t_discrete = discretize_time(test_time, None, max, num);
    let t_restored = time_bin_to_seconds(t_discrete, None, max, num);
    */

    /*
    // test velocity discretization
    let v = 18;
    let test_velocity = Velocity::new::<meter_per_second>((2 *v) as PrefFloat);
    let (max, num) = (Velocity::new::<meter_per_second>(200.0), 2000);
    let v_discrete = discretize_v(test_velocity, None, max, num);
    let v_restored = v_bin_to_mps(v_discrete, None, max, num);
    
    println!("v = {:?}, v disc = {:?}, v rest = {:?}", test_velocity, v_discrete, v_restored);
    */

}