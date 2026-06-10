// sets f64 as basic floating point type
pub type PrefFloat = f64; // preferred floating point type
pub use std::f64 as floats;
pub use uom::si::f64 as uom_si_preffloat;

/*
// uncomment this section to use f32 instead
pub type PrefFloat = f32; // preferred floating point type
pub use std::f32 as floats;
pub use uom::si::f32 as uom_si_preffloat;
*/