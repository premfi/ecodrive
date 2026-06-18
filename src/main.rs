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
    o implement ImpossibleTaskError to return if given time is too short
    x implement NoPathFoundError
    o retrieve best path in dp_optim() and return it
    o start with lowest reachable velocity
    o add argument for initial velocity. Set another entry (according to discretize(v0)) of mat_parents and mat_e_used to 0 for this
    o introduce minimum velocity (can also help optimization performance)
    o try out clever splitting of route into sections such that maximum acceleration can be used
    o add splitting function for routes or repeats/splits/etc. argument to load_route()
    o write inverse optimization with fixed energy budget and time to be optimized
    o add utils functions, e.g. max_s() are not used but helpful for understanding
    o clean up plotting functions and design interface for loading schedules from file to python
    o maybe add function that takes three paths: route, vehicles and returned schedule(s) and max_time, t_res, v_res that automatically calculates all of them
*/

use config::uom_si_preffloat::{Mass, Area, Length, Ratio, Velocity, Time};
use uom::si::{mass::kilogram,
            area::square_meter, 
            length::meter, 
            ratio::percent, 
            velocity::{kilometer_per_hour, meter_per_second},
            time::second};

use ndarray::Array3;

// fn load_vehicles(path: &str) -> Option<Vehicle> {
//     let mut vehicles: Vec<Vehicle> = vec![];
//     let mut reader = csv::ReaderBuilder::new().trim(csv::Trim::All).from_path(path).unwrap();

//     for record in reader.deserialize() {
//         let mut vehicle: Vehicle = record.unwrap();
//         vehicle.update_c_param();
//         println!("{:?}", vehicle);
//         return Some(vehicle);
//     }
//     None
// }

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
    println!("route_res= {:?}", route_res);

    let route0 = Route {lengths: lengths.clone(), slopes: slopes.clone(), min_speeds: vec![Velocity::new::<kilometer_per_hour>(0.0); 4], max_speeds: max_speeds.clone()};

    let route3_res8 = Route {lengths: vec![Length::new::<meter>(50.0); 40],
                            slopes: vec![Ratio::new::<percent>(0.0); 40],
                            min_speeds: vec![Velocity::new::<kilometer_per_hour>(0.0); 40],
                            max_speeds: vec![Velocity::new::<kilometer_per_hour>(100.0); 40]};
    
    let max_time = Time::new::<second>(200.0);
    let time_res = 2000;
    let v_res = 201;

    let ekin_stuck = v_to_ekin(Velocity::new::<meter_per_second>(0.75));
    println!("ekin_stuck: {:?}", ekin_stuck);
    let a_param_retrieved = retrieve_a_param(Length::new::<meter>(50.0), ekin_stuck, ekin_stuck, car1.get_c_param());
    println!("a_param_retrieved: {:?}", a_param_retrieved);

    let mom = a_param_retrieved + route_res;
    println!("mom: {:?}", mom);

    let state_t_in_seconds = time_bin_to_seconds(971, None, max_time, time_res);
    println!("state_t_in_seconds: {:?}", state_t_in_seconds);
    let dt = delta_t(Length::new::<meter>(50.0), a_param_retrieved, car1.get_c_param(), ekin_stuck);
    println!("dt: {:?}", dt);
    let time_used_next = dt + state_t_in_seconds;

    println!("time_used_next: {:?}", time_used_next);

    let optimal_schedule_result = dp_optim(&route0, &car1, max_time, time_res, v_res);
    println!("DP: {:?}", optimal_schedule_result.unwrap());
    println!("time value: {:<10.4}", (max_time / 3.).into_format_args(second, uom::fmt::DisplayStyle::Abbreviation));
    println!("nice time value: {:#?}", max_time);

    // for t in 960..=980 {
    //     let t_sec = time_bin_to_seconds(t, None, max_time, time_res);
    //     println!("t={}, t_sec={:?}", t, t_sec);
    // }

    // let mut arr4 = Array3::from_shape_vec((3, 3, 3), (0..27).collect()).unwrap();
    // arr4[[1, 0, 1]] = 15;
    // arr4[[1, 2, 0]] = 18;
    // let loaded_route = load_route("../route3.csv").unwrap();
    // println!("load_route:\n{:?}", loaded_route.lengths);
    let vhcls = load_vehicles("../vehicle1.csv").unwrap();
    let vhcl0 = &vhcls[0];
    let vhcl1 = &vhcls[1];
    println!("vhcl0.get_c_param()={:?}", vhcl0.get_c_param());
    println!("vhcl1.get_c_param()={:?}", vhcl1.get_c_param());

    // use ndarray_stats::QuantileExt;
    // let max4 = arr4.select(ndarray::Axis(0), &[1]).map_axis(ndarray::Axis(2), |view| view.argmax().unwrap());
    // let max5 = arr4.select(ndarray::Axis(0), &[1, 2]).argmax().unwrap();
    // let total_max = arr4.argmax();//.map(|view| view.min());
    // println!("max4={:?}", max4);
    // println!("max5={:?}", max5);
    // println!("total_max={:?}", total_max);
    // println!("arr4={:?}", arr4);
    // println!("arr4.shape: {:?}", arr4.shape()[0]);

    // fn el_wise<A, B, C>(f: fn(A, B) -> C, va: &Vec<A>, vb: &Vec<B>) -> Vec<C> {
    //     println!("el_wise called!");
    //     std::iter::zip(va, vb).map(|(a, b)| f(a, b)).collect()
    // }

    // let res = el_wise(<&i64 as std::ops::Add>::add, &vec![&101_i64], &vec![&3_i64]);
    // println!("el_wise={:?}", res);

    // println!("1 + 2 = {:?}", &1 + &2);

    // let div: Vec<Time> = std::iter::zip(lengths, max_speeds).map(|(s, v)| {s / v}).collect();
    // // for d in div {
    // //     println!("={:?}", d);
    // // }
    // println!("{:?}", div);
    // println!("{:?}", div.into_iter().sum());

    // let sum = std::iter::zip(lengths, max_speeds).map(|(s, v)| {s / v}).sum::<Time>();
    // println!("sum={:?}", sum);


    // let mut arr1 = ndarray::Array3::<f64>::zeros((3, 4, 5));
    // arr1.fill(f64::INFINITY);
    // // println!("{:?}", arr1);
    // arr1[[2, 2, 2]] = 15.5;

    // let arr2 = ndarray::Array3::<f64>::ones((3, 4, 5));
    // let arr3 = 3.0 * arr2 / arr1;
    // // println!("{:?}", arr3);

    // use serde::Serialize;

    // #[derive(Serialize)]
    // struct Record<'a> {
    //     name: &'a str,
    //     place: &'a str,
    //     id: u64,
    // }

    // #[derive(Serialize)]
    // struct TestStruct {
    //     vec1: Vec<i32>,
    //     vec2: Vec<f32>,
    // }

    // let mut wtr = csv::Writer::from_writer(std::io::stdout());

    // let rec1 = Record { name: "Mark", place: "Melbourne", id: 56};
    // let rec2 = Record { name: "Ashley", place: "Sydney", id: 64};
    // let rec3 = Record { name: "Akshat", place: "Delhi", id: 98};
    // // let rec4 = TestStruct {vec1: vec![0, 1, 2, 3, 4],
    // //                        vec2: vec![0.0, 0.1, 0.2, 0.3, 0.4]};

    // wtr.serialize(rec1)?;
    // wtr.serialize(rec2)?;
    // wtr.serialize(rec3)?;
    // // wtr.serialize(rec4)?;

    // wtr.flush()?;

    // let sched = DrivingSchedule {times: vec![Time::new::<second>(1.1), Time::new::<second>(2.2), Time::new::<second>(3.3)],
    //                             speeds: vec![Velocity::new::<kilometer_per_hour>(15.0), Velocity::new::<kilometer_per_hour>(30.0), Velocity::new::<kilometer_per_hour>(20.0)]};

    // let saved_res = sched.save("blablub_path.csv");
    // println!("{saved_res:?}");

    // println!("{:?}", sched.times[2]);

    // let v_arr1 = Array1::from_vec(vec![Velocity::new::<meter_per_second>(1.), Velocity::new::<meter_per_second>(2.), Velocity::new::<meter_per_second>(3.)]);
    // let v_arr2 = Array1::from_vec(vec![Velocity::new::<meter_per_second>(2.), Velocity::new::<meter_per_second>(2.), Velocity::new::<meter_per_second>(4.)]);

    // let v_arr3 = v_arr1 / v_arr2;

    // println!("{:?}", v_arr3);

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
    Ok(())
}