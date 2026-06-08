mod ecodrive;
use ecodrive::*;

use uom::si::f64::{Mass, Area, Length, Ratio, Velocity, Acceleration, AvailableEnergy};
use uom::si::{mass::kilogram,
            area::square_meter, 
            length::meter, 
            ratio::percent, 
            velocity::{meter_per_second, kilometer_per_hour}, 
            acceleration::meter_per_second_squared, 
            available_energy::joule_per_kilogram};
use uom;
use float_cmp::approx_eq;


fn main() {
    /* TODO: Instead of manually defining vehicles, they may be imported from a .csv file, stored in a list, and then executed one by one */
    let car1 = Vehicle::builder(0.01,
                                Mass::new::<kilogram>(2000.0),
                                1.1,
                                0.3,
                                Area::new::<square_meter>(2.0),
                                0.3);

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

    println!("Hello, world!");
    println!("g={:?}", GRAVITY_OF_EARTH);
    println!("{:?}", car1.mass * car1.frontal_area);
    let g = uom::si::acceleration::meter_per_second_squared{};
    let test_mom = Acceleration::new::<meter_per_second_squared>(-2.8);
    let test_s = Length::new::<meter>(50.0);
    let test_a_param = Acceleration::new::<meter_per_second_squared>(-0.0);
    let test_ekin_0 = AvailableEnergy::new::<joule_per_kilogram>(0.0);
    let test_ekin_s = AvailableEnergy::new::<joule_per_kilogram>(75.0);
    println!("delta t = {:?}\n", delta_t(test_s, test_a_param, car1.get_c_param(), test_ekin_0));

    println!("energy_used = {:?}", energy_used(test_s, test_mom, car1.rec_eff));
    let test_v = Velocity::new::<kilometer_per_hour>(55.0);
    println!("velocity = {:?}", v_to_ekin(test_v));
    println!("c={:?}", car1.get_c_param());
    println!("{:?}", g);
    println!("sqrt {}", PrefFloat::sqrt(2.0));
    let test_a_retrieved = retrieve_a_param(test_s, test_ekin_0, test_ekin_s, car1.get_c_param());
    println!("retrieved A = {:?}", test_a_retrieved);
    println!("approx eq? {}", approx_eq!(f64, 1.0_f64, 1.000000000000001_f64, ulps = 2));
}