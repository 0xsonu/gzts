use chrono::{Duration, Timelike, Utc};

pub struct RopInfo {
    pub rop_start_date: String,
    pub rop_start_hhmm: String,
    pub rop_end_hhmm: String,
    pub rop_start_time: String,
    pub rop_end_time: String,
    pub epoch_range: String,
    pub new_date: String,
}

impl RopInfo {
    pub fn from_current_utc() -> Self {
        let now = Utc::now();
        let rounded_min = (now.minute() / 15) * 15;
        let rop_start = now
            .with_minute(rounded_min).unwrap()
            .with_second(0).unwrap()
            .with_nanosecond(0).unwrap();
        let rop_end = rop_start + Duration::minutes(15);

        let rop_start_date = rop_start.format("%Y%m%d").to_string();
        let rop_start_hhmm = rop_start.format("%H%M").to_string();
        let rop_end_hhmm = rop_end.format("%H%M").to_string();
        let rop_start_time = rop_start.format("%H:%M").to_string();
        let rop_end_time = rop_end.format("%H:%M").to_string();
        let epoch_range = format!("{}_{}", rop_start.timestamp(), rop_end.timestamp());

        let new_date = format!(
            "{}-{}-{}",
            &rop_start_date[..4],
            &rop_start_date[4..6],
            &rop_start_date[6..8]
        );

        Self {
            rop_start_date,
            rop_start_hhmm,
            rop_end_hhmm,
            rop_start_time,
            rop_end_time,
            epoch_range,
            new_date,
        }
    }

    pub fn current_rop(&self) -> String {
        format!("{}-{}", self.rop_start_hhmm, self.rop_end_hhmm)
    }
}
