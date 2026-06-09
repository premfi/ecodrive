use uom::si::f64::{Mass, Area, Acceleration, MassDensity, Length, Velocity, Ratio, AvailableEnergy, Time};
use uom::typenum::{N1, Z0};
use uom::si::{acceleration::meter_per_second_squared,
            available_energy::joule_per_kilogram,
            time::second};
use std::marker::PhantomData;
use float_cmp::approx_eq;

pub type PrefFloat = f64; // preferred floating point type
pub use std::f64 as floats;
pub type PerLength = uom::si::Quantity<uom::si::ISQ<N1, Z0, Z0, Z0, Z0, Z0, Z0>,
                                            uom::si::SI<PrefFloat>, PrefFloat>; // [1/m]

                                    

pub const GRAVITY_OF_EARTH: Acceleration = Acceleration {dimension:PhantomData, units: PhantomData, value: 9.81}; // gravitational acceleration [m/s^2]
// pub const C_R: PrefFloat = 0.01; // rolling resistance coefficient
// pub const MASS: Mass = Mass {dimension: PhantomData, units: PhantomData, value: 2000.0}; // vehicle mass [kg]
// pub const RHO_ROT: PrefFloat = 1.1; // factor for equivalent mass of rotating parts
pub const RHO_AIR: MassDensity = MassDensity {dimension: PhantomData, units: PhantomData, value: 1.2}; // air density [kg/m^3]
// pub const C_W: PrefFloat = 0.3;     // drag coefficient
// pub const A_FRONT: Area = Area {dimension: PhantomData, units: PhantomData, value: 2.0}; // frontal area [m^2]
// pub const C_DEFAULT: PrefFloat = RHO_AIR * C_W * A_FRONT / MASS;
// pub const REC_EFF: PrefFloat = 0.3; // regenerative braking efficiency

pub struct Vehicle {
    pub roll_res_coeff: PrefFloat,  // rolling resistance coefficient
    pub mass: Mass,                 // vehicle mass [kg]
    pub rho_rot: PrefFloat,         // factor for equivalent mass of rotating parts
    pub c_w: PrefFloat,             // drag coefficient
    pub frontal_area: Area,         // frontal area [m^2]
    pub rec_eff: PrefFloat,          // regenerative braking efficiency
    c_param: Option<uom::si::Quantity<uom::si::ISQ<N1, Z0, Z0, Z0, Z0, Z0, Z0>,
                                        uom::si::SI<PrefFloat>, PrefFloat>> // C parameter (mass-normalized air resistance prefactor) [1/m]. Automatically calculated.
}

impl Vehicle {
    pub fn builder(roll_res_coeff: PrefFloat, // rolling resistance coefficient
                mass: Mass,                 // vehicle mass [kg]
                rho_rot: PrefFloat,         // factor for equivalent mass of rotating parts
                c_w: PrefFloat,             // drag coefficient
                frontal_area: Area,         // frontal area [m^2]
                rec_eff: PrefFloat          // regenerative braking efficiency
                ) -> Vehicle {

        // fill the public fields
        let mut vhl = Vehicle {roll_res_coeff,
                    mass,
                    rho_rot,
                    c_w,
                    frontal_area,
                    rec_eff,
                    c_param: None};

        // calculate and set C parameter from given values
        vhl.c_param = Some(vhl.calc_c_param());

        vhl
    }

    fn calc_c_param(&self) -> uom::si::Quantity<uom::si::ISQ<N1, Z0, Z0, Z0, Z0, Z0, Z0>,
                                            uom::si::SI<PrefFloat>, PrefFloat> /* [1/m] */ {
        /* calculate C parameter from given values */
        RHO_AIR * self.c_w * self.frontal_area / self.mass
    }

    pub fn get_c_param(&self) -> uom::si::Quantity<uom::si::ISQ<N1, Z0, Z0, Z0, Z0, Z0, Z0>,
                                        uom::si::SI<PrefFloat>, PrefFloat> /* [1/m] */ {
        /* get C parameter */
        self.c_param.expect("c_param not set! Should have been calculated automatically.")
    }
}


pub struct Route {
    pub lengths: Vec<Length>,
    pub slopes: Vec<Ratio>,
    pub max_speeds: Vec<Velocity>, 

}

// fn e_kin(s, )

pub fn energy_used(s: Length, mom: Acceleration /* [N/kg] */, rec_eff: PrefFloat) -> AvailableEnergy /* [J/kg] */ {
    /* Specific used energy when applying moment `mom` on length `s` */
    if mom >= Acceleration::new::<meter_per_second_squared>(0.0) {
        s * mom
    } else {
        s * mom * rec_eff
    }
}

pub fn retrieve_a_param(s: Length, ekin_0: AvailableEnergy, ekin_s: AvailableEnergy, c_param: PerLength) -> Acceleration /* [N/kg] */ {
    /* Calculate A parameter necessary to reach `ekin_s` after length `s` when starting with `ekin_0` */
    let a_param = c_param * (ekin_s - ekin_0 * floats::consts::E.powf((-c_param * s).into())) / (1.0 - floats::consts::E.powf((-c_param * s).into()));
    a_param
}

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

pub fn v_to_ekin(v: Velocity) -> AvailableEnergy {
    v * v / 2.0
}

// fn ekin_to_v(ekin) // not necessary in direct DP version

// *discretize and undiscretice t and v (4 functions in total)*

pub fn discretize_time(t: Time, min: Option<Time>, max: Time, num: usize) -> usize {
    if t == Time::new::<second>(PrefFloat::INFINITY) {
        return num - 1;
    }

    assert!(t >= Time::new::<second>(0.0), "`t` must be non-negative! t={:?}", t);

    let min = min.unwrap_or(Time::new::<second>(0.0));

    assert!(min <= max, "`min` most not be larger than `max`! min={:?}, max={:?}", min, max);

    if t < min {
        return 0;
    }

    let stepsize = (max - min) / (num - 1) as PrefFloat;
    // println!("stepsize = {:?}", stepsize);
    let bin_unclipped = PrefFloat::from((t - min) / stepsize).ceil() as usize;
    let bin = std::cmp::min(bin_unclipped, num - 1);
    
    bin
}

pub fn time_bin_to_seconds(bin: usize, min: Option<Time>, max: Time, num: usize) -> Time {
    let min = min.unwrap_or(Time::new::<second>(0.0));
    let stepsize = (max - min) / (num - 1) as PrefFloat;

    stepsize * (bin as PrefFloat) + min
}

// fn DP(route, vehicle, max_time, time_res, v_res)