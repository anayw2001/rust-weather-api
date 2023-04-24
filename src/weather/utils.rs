use crate::weather::entities::Conditions;

pub(crate) fn convert_aqi_to_string(aqi: i64) -> String {
    String::from(match aqi {
        1 => "Good",
        2 => "Fair",
        3 => "Moderate",
        4 => "Poor",
        _ => "Very Poor",
    })
}

pub(crate) fn convert_id_to_condition(current_weather_id: i64) -> Conditions {
    if current_weather_id == 800 {
        Conditions::Clear
    } else if current_weather_id > 800 && current_weather_id < 805 {
        if current_weather_id == 804 {
            Conditions::Overcast
        } else {
            Conditions::Cloudy
        }
    } else if current_weather_id > 599 && current_weather_id < 700 {
        Conditions::Snow
    } else if current_weather_id > 199 && current_weather_id < 300 {
        Conditions::Storm
    } else {
        Conditions::Rainy
    }
}

/// converts Earth surface co-ordinates in degrees of latitude and longitude to 3D cartesian coordinates on a unit sphere
///
/// We use this when populating our tree, to convert from the `f32` lat/lng data into `f32` (x,y,z) co-ordinates to store in our tree, as well as
/// allowing us to query the created tree using lat/lng query points.
pub fn degrees_lat_lng_to_unit_sphere(lat: f64, lng: f64) -> [f64; 3] {
    // convert from degrees to radians
    let lat = lat.to_radians();
    let lng = lng.to_radians();

    // convert from ra/dec to xyz coords on unit sphere
    [lat.cos() * lng.cos(), lat.cos() * lng.sin(), lat.sin()]
}

pub const EARTH_RADIUS_IN_KM: f64 = 6371.0;

/// Converts a squared euclidean unit sphere distance (like what we'd get back from
/// our kd-tree) into kilometres for user convenience.
#[allow(dead_code)]
pub fn unit_sphere_squared_euclidean_to_kilometres(sq_euc_dist: f64) -> f64{
    sq_euc_dist.sqrt() * EARTH_RADIUS_IN_KM
}

/// Converts a value in km to squared euclidean distance on a unit sphere representing Earth.
///
/// This allows us to query using kilometres as distances in our kd-tree.
#[allow(dead_code)]
pub fn kilometres_to_unit_sphere_squared_euclidean(km_dist: f64) -> f64 {
    (km_dist / EARTH_RADIUS_IN_KM).powi(2)
}
