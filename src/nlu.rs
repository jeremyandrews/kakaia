use snips_nlu_lib::SnipsNluEngine;

pub struct NLU {
    pub engine: SnipsNluEngine,
}

impl NLU {
    pub fn new() -> Self {
        // @TODO: install the kakaia_engine and make the path configurable
        NLU {
            engine: match SnipsNluEngine::from_path("./nlu/kakaia_engine/") {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Error loadinh SnipsNluEngine: {}", e);
                    std::process::exit(1);
                }
            },
        }
    }

    pub fn duration_as_seconds(&self, timer_values: &serde_json::Value) -> i64 {
        let seconds: i64;
        if timer_values["seconds"].is_i64() {
            seconds = match serde_json::from_value(timer_values["seconds"].clone()) {
                Ok(s) => s,
                Err(_) => 0,
            };
        } else {
            seconds = 0;
        }

        let minutes: i64;
        if timer_values["minutes"].is_i64() {
            minutes = match serde_json::from_value(timer_values["minutes"].clone()) {
                Ok(m) => m,
                Err(_) => 0,
            };
        } else {
            minutes = 0;
        }

        let hours: i64;
        if timer_values["hours"].is_i64() {
            hours = match serde_json::from_value(timer_values["hours"].clone()) {
                Ok(h) => h,
                Err(_) => 0,
            };
        } else {
            hours = 0;
        }

        let days: i64;
        if timer_values["days"].is_i64() {
            days = match serde_json::from_value(timer_values["days"].clone()) {
                Ok(d) => d,
                Err(_) => 0,
            };
        } else {
            days = 0;
        }

        let weeks: i64;
        if timer_values["weeks"].is_i64() {
            weeks = match serde_json::from_value(timer_values["weeks"].clone()) {
                Ok(w) => w,
                Err(_) => 0,
            };
        } else {
            weeks = 0;
        }

        let months: i64;
        if timer_values["months"].is_i64() {
            months = match serde_json::from_value(timer_values["months"].clone()) {
                Ok(m) => m,
                Err(_) => 0,
            };
        } else {
            months = 0;
        }

        let years: i64;
        if timer_values["years"].is_i64() {
            years = match serde_json::from_value(timer_values["years"].clone()) {
                Ok(y) => y,
                Err(_) => 0,
            };
        } else {
            years = 0;
        }

        seconds + minutes * 60 +
            hours * 60 * 60 +
            days * 86400 +
            weeks * 86400 * 7 +
            months * 86400 * 30 +
            years * 86400 * 365
    }
}
