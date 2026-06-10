use uom::si::f64::{Mass, Area, MassDensity};
use uom::typenum::{N1, Z0};

use std::marker::PhantomData;

pub type PrefFloat = f64; // preferred floating point type
pub type PerLength = uom::si::Quantity<uom::si::ISQ<N1, Z0, Z0, Z0, Z0, Z0, Z0>,
                                            uom::si::SI<PrefFloat>, PrefFloat>; // [1/m]

pub const RHO_AIR: MassDensity = MassDensity {dimension: PhantomData, units: PhantomData, value: 1.2}; // air density [kg/m^3]

pub struct Vehicle {
    pub roll_res_coeff: PrefFloat,  // rolling resistance coefficient
    pub mass: Mass,                 // vehicle mass [kg]
    pub rho_rot: PrefFloat,         // factor for equivalent mass of rotating parts
    pub c_w: PrefFloat,             // drag coefficient
    pub frontal_area: Area,         // frontal area [m^2]
    pub rec_eff: PrefFloat,          // regenerative braking efficiency
    c_param: Option<PerLength> // C parameter (mass-normalized air resistance prefactor) [1/m]. Automatically calculated.
}

impl Vehicle {
    pub fn new(roll_res_coeff: PrefFloat, // rolling resistance coefficient
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

    fn calc_c_param(&self) -> PerLength /* [1/m] */ {
        /* calculate C parameter from given values */
        RHO_AIR * self.c_w * self.frontal_area / self.mass
    }

    pub fn get_c_param(&self) -> PerLength /* [1/m] */ {
        /* get C parameter */
        self.c_param.expect("c_param not set! Should have been calculated automatically.")
    }

    // pub fn set_mass(&mut self, mass: Mass) {
    //     self.mass = mass;
    //     self.calc_c_param();
    // }
}