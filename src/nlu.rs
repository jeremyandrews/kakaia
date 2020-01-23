use snips_nlu_lib::SnipsNluEngine;
use serde_json::value::{Value, Map};

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

    // Parse string with NLU engine and return as json value
    pub fn parse(&self, text: &str) -> Value {
        // @TODO: error handling
        let parsed = self.engine.parse(text, None, None).unwrap();
        serde_json::to_value(&parsed).unwrap()
    }

    // Get command String
    pub fn get_command(&self, parsed_json: &Value) -> String {
        let intent = match &parsed_json["intent"].as_object() {
            Some(i) => i.clone(),
            None => return "none".to_string(),
        };
        match intent["intentName"].as_str() {
            Some(i) => i.to_string(),
            None => "none".to_string(),
        }
    }

    // Confirm the response has the expected number of slots
    pub fn has_expected_slots(&self, parsed_json: &Value, count: usize) -> bool {
        let slots = match parsed_json["slots"].as_array() {
            Some(s) => s,
            None => return false
        };
        slots.len() == count
    }

    // Get the value of a specific slot
    pub fn get_slot_value<'a>(&self, parsed_json: &'a Value, entity: &str, slot_name: &str) -> Option<&'a Map<String, Value>> {
        let slots = match parsed_json["slots"].as_array() {
            Some(s) => s,
            None => return None
        };
        for slot in slots {
            match slot.as_object() {
                Some(slot) => {
                    if slot["entity"] == entity && slot["slotName"] == slot_name {
                        return slot["value"].as_object()
                    }
                },
                None => (),
            }
        }
        None
    }

    pub fn get_float(&self, value: Option<&Map<String, Value>>) -> f64 {
        match value {
            Some(v) => {
                match v["value"].as_f64() {
                    Some(n) => n,
                    None => 0.0,
                }
            },
            None => 0.0,
        }
    }

    pub fn get_string(&self, value: Option<&Map<String, Value>>) -> String {
        match value {
            Some(v) => {
                match v["value"].as_str() {
                    Some(n) => n.to_string(),
                    None => "".to_string(),
                }
            },
            None => "".to_string(),
        }
    }

    pub fn get_string_custom(&self, value: Option<&Map<String, Value>>, custom: &str) -> String {
        match value {
            Some(v) => {
                match v[custom].as_str() {
                    Some(n) => n.to_string(),
                    None => "".to_string(),
                }
            },
            None => "".to_string(),
        }
    }

    pub fn duration_as_seconds(&self, timer_values: &Map<String, Value>) -> f64 {
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

        (seconds + minutes * 60 +
            hours * 60 * 60 +
            days * 86400 +
            weeks * 86400 * 7 +
            months * 86400 * 30 +
            years * 86400 * 365) as f64
    }
}
