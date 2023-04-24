use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct DoGeocodeResp {
    pub(crate) name: String,
    pub(crate) local_names: HashMap<String, String>,
    pub(crate) lat: f64,
    pub(crate) lon: f64,
    pub(crate) country: String,
}
