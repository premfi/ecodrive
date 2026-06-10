use crate::ecodrive::config::PrefFloat;
use crate::ecodrive::config::uom_si_preffloat::{Mass, Area, MassDensity};
use uom::typenum::{N1, Z0};

use std::marker::PhantomData;

use crate::ecodrive::constants::{RHO_AIR};
use crate::ecodrive::PerLength;

pub struct Vehicle {
    pub roll_res_coeff: PrefFloat,  // rolling resistance coefficient
    pub rho_rot: PrefFloat,     // factor for equivalent mass of rotating parts
    pub rec_eff: PrefFloat,     // regenerative braking efficiency
    mass: Mass,                 // vehicle mass [kg]
    frontal_area: Area,         // frontal area [m^2]
    c_w: PrefFloat,             // drag coefficient
    c_param: Option<PerLength>  // C parameter (mass-normalized air resistance prefactor) [1/m]. Automatically calculated.
}

impl Vehicle {
    pub fn new(roll_res_coeff: PrefFloat,   // rolling resistance coefficient
                rec_eff: PrefFloat,     // regenerative braking efficiency
                rho_rot: PrefFloat,     // factor for equivalent mass of rotating parts
                mass: Mass,             // vehicle mass [kg]
                frontal_area: Area,     // frontal area [m^2]
                c_w: PrefFloat          // drag coefficient
                ) -> Vehicle {

        // fill the public fields
        let mut vhl = Vehicle {roll_res_coeff,
                    rec_eff,
                    rho_rot,
                    mass,
                    frontal_area,
                    c_w,
                    c_param: None};

        // calculate C parameter from given values
        vhl.update_c_param();

        vhl
    }


    fn update_c_param(&mut self) {
        /* calculate C parameter from c_w, frontal_area and mass */
        self.c_param = Some(RHO_AIR * self.c_w * self.frontal_area / self.mass)
    }

    pub fn get_c_param(&self) -> PerLength /* [1/m] */ {
        /* get C parameter */
        self.c_param.expect("c_param not set! Should have been calculated automatically.")
    }


    pub fn set_mass(&mut self, mass: Mass) {
        self.mass = mass;
        self.update_c_param();
    }

    pub fn get_mass(&self) -> Mass {
        self.mass
    }
    

    pub fn set_c_w(&mut self, c_w: PrefFloat) {
        self.c_w = c_w;
        self.update_c_param();
    }

    pub fn get_c_w(&self) -> PrefFloat {
        self.c_w
    }


    pub fn set_frontal_area(&mut self, frontal_area: Area) {
        self.frontal_area = frontal_area;
        self.update_c_param();
    }

    pub fn get_frontal_area(&self) -> Area {
        self.frontal_area
    }
}