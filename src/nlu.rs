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
        let seconds: i64 = match serde_json::from_value(timer_values["seconds"].clone()) {
            Ok(s) => s,
            Err(_) => 0,
        };
        let minutes: i64 = match serde_json::from_value(timer_values["minutes"].clone()) {
            Ok(m) => m,
            Err(_) => 0,
        };
        let hours: i64 = match serde_json::from_value(timer_values["hours"].clone()) {
            Ok(h) => h,
            Err(_) => 0,
        };
        let days: i64 = match serde_json::from_value(timer_values["days"].clone()) {
            Ok(d) => d,
            Err(_) => 0,
        };
        let weeks: i64 = match serde_json::from_value(timer_values["weeks"].clone()) {
            Ok(w) => w,
            Err(_) => 0,
        };
        seconds + minutes * 60 + hours * 60 * 60 + days * 86400 + weeks * 86400 * 7
    }
}
