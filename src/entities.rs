pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

impl Location {
    pub fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
        }
    }
}
