use uom::si::f64::{Mass, Area, Acceleration, MassDensity, Length, Velocity, Ratio, AvailableEnergy};
use uom::typenum::{N1, Z0};
use uom::si::acceleration::meter_per_second_squared;
use std::marker::PhantomData;

pub type PrefFloat = f64; // preferred floating point type
pub type PerMeter = uom::si::Quantity<uom::si::ISQ<N1, Z0, Z0, Z0, Z0, Z0, Z0>,
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
                rec_eff: PrefFloat     // regenerative braking efficiency
                ) -> Vehicle {
        // fill the public fields
        let mut vhl = Vehicle {roll_res_coeff,
                    mass,
                    rho_rot,
                    c_w,
                    frontal_area,
                    rec_eff,
                    c_param: None};

        // calculate and set c_param from given values
        vhl.c_param = Some(vhl.calc_c_param());

        vhl
    }

    fn calc_c_param(&self) -> uom::si::Quantity<uom::si::ISQ<N1, Z0, Z0, Z0, Z0, Z0, Z0>,
                                            uom::si::SI<PrefFloat>, PrefFloat> /* [1/m] */ {
        /* calculate c_param from given values */
        RHO_AIR * self.c_w * self.frontal_area / self.mass
    }

    pub fn get_c_param(&self) -> uom::si::Quantity<uom::si::ISQ<N1, Z0, Z0, Z0, Z0, Z0, Z0>,
                                        uom::si::SI<PrefFloat>, PrefFloat> /* [1/m] */ {
        /* get value from private c_param field */
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

// fn retrieve_A(s, )

// fn delta_t(s, A, C, e_kin0)

// fn v_to_ekin(v)

// fn ekin_to_v(ekin)

// *discretize and undiscretice t and v (4 functions in total)*

// fn DP(route, vehicle, max_time, time_res, v_res)