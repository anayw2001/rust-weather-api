use crate::{weather::entities::ProtoAdapter, weather_proto::weather_message};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct DoGeocodeResp {
    pub(crate) name: String,
    pub(crate) local_names: HashMap<String, String>,
    pub(crate) lat: f64,
    pub(crate) lon: f64,
    pub(crate) country: String,
    pub(crate) state: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReverseGeocode {
    pub name: String,
    pub country: String,
    pub state: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl ReverseGeocode {
    pub(crate) fn default() -> Self {
        Self {
            name: "".to_string(),
            country: "".to_string(),
            state: "".to_string(),
            latitude: 0.0,
            longitude: 0.0,
        }
    }
}

impl ProtoAdapter for ReverseGeocode {
    type ProtoType = weather_message::ReverseGeocode;

    fn to_proto(&self) -> Self::ProtoType {
        weather_message::ReverseGeocode {
            name: self.name.to_owned(),
            country: self.country.to_owned(),
            state: self.state.to_owned(),
            latitude: self.latitude,
            longitude: self.longitude,
            ..Default::default()
        }
    }
}
