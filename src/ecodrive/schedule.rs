use crate::ecodrive::config::uom_si_preffloat::{Velocity, Time};
use uom::si::{velocity::kilometer_per_hour,
            time::second};

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