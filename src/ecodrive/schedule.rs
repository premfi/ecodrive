use crate::ecodrive::config::uom_si_preffloat::{Velocity, Time};
use uom::si::{velocity::{kilometer_per_hour, meter_per_second},
            time::second};
use crate::config::PrefFloat;

use serde::{Serialize, Serializer};

fn serialize_time_to_s<S>(time: &Time, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let seconds = time.get::<second>();
    serializer.serialize_f64(seconds as f64)
}

fn serialize_velocity_to_kph<S>(velocity: &Velocity, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let kilometers_per_hour = velocity.get::<kilometer_per_hour>();
    serializer.serialize_f64(kilometers_per_hour as f64)
}

#[derive(Serialize)]
struct DrivingScheduleRow {
    #[serde(serialize_with="serialize_time_to_s", rename="time [s]")]
    time: Time,
    #[serde(serialize_with="serialize_velocity_to_kph", rename="speed [km/h]")]
    speed: Velocity
}

#[derive(Debug)]
pub struct DrivingSchedule {
    pub times: Vec<Time>,
    pub speeds: Vec<Velocity>,
}

impl DrivingSchedule {
    pub fn save(&self, path: &str) -> Result<(), csv::Error> {
        let path = if !path.ends_with(".csv") { &format!("{path}.csv") } else { path };
        println!("Saving DrivingSchedule to {}", path);

        let mut wtr = csv::Writer::from_path(path)?; // TODO: create new file "path(1)" or find other way to handle this without aborting. Create necessary folder if it doesn't exist already

        for (&t, &v) in self.times.iter().zip(self.speeds.iter()) {
            let row = DrivingScheduleRow {time: t, speed: v};
            // println!("t={:?}, v={:?}", t, v);
            wtr.serialize(row)?;
        }

        wtr.flush()?;

        Ok(())
    }
}

use uom::fmt::DisplayStyle::Abbreviation;

impl std::fmt::Display for DrivingSchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let len_t = self.times.len();
        let len_v = self.speeds.len();

        let mut t_iter = self.times.iter();
        let mut v_iter = self.speeds.iter();

        let dec_places = 3;
        let width_t = 9;
        let width_v = 7;
        let char_per_row = width_t + width_v + 8; // add length of " s," + " m/s\n"
        let mut s = String::with_capacity(char_per_row * len_t.max(len_v));

        s.push_str(&format!("{:>wid_t$},{:>wid_v$}", "times", "speeds", wid_t=width_t+2, wid_v=width_v+4));

        // iterate through times and speeds. If one ends before the other, print empty while the other continues
        for i in 0..len_t.max(len_v) {
            let t_next = t_iter.next();
            let v_next = v_iter.next();
            match (t_next, v_next) {
                (Some(t), Some(v)) => s.push_str(&format!("\n{:>width_t$.dec_places$},{:>width_v$.dec_places$}",
                                                        t.into_format_args(second, Abbreviation),
                                                        v.into_format_args(meter_per_second, Abbreviation))),
                (None, Some(v)) => s.push_str(&format!("\n{:>wid_t$},{:>width_v$.dec_places$}", "<empty>",
                                                        v.into_format_args(meter_per_second, Abbreviation),
                                                        wid_t=width_t+2)),
                (Some(t), None) => s.push_str(&format!("\n{:>width_t$.dec_places$},{:>wid_v$}",
                                                        t.into_format_args(second, Abbreviation),
                                                        "<empty>", wid_v=width_v+4)),
                (None, None) => break,
            }
        }

        // in case times and speeds are both empty, print empty once
        if len_t == 0 && len_v == 0 {
            s.push_str(&format!("\n{:>wid_t$},{:>wid_v$}", "<empty>", "<empty>", wid_t=width_t+2, wid_v=width_v+4))
        }

        write!(f, "{}", s)
    }
}