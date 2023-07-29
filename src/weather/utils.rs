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
    } else if current_weather_id > 699 && current_weather_id < 799 {
        if current_weather_id == 711 {
            Conditions::Smoke
        } else {
            Conditions::Haze
        }
    } else {
        Conditions::Rainy
    }
}

pub const EARTH_RADIUS_IN_KM: f64 = 6371.0;

// location1 and location2 are [latitude, longitude].
pub(crate) fn haversine(location1: &[f64; 2], location2: &[f64; 2]) -> f64 {
    tracing::info!("location1: {:?}, location2: {:?}", location1, location2);
    let d_lat: f64 = (location2[0] - location1[0]).to_radians();
    let d_lon: f64 = (location2[1] - location1[1]).to_radians();
    let lat1: f64 = (location1[0]).to_radians();
    let lat2: f64 = (location2[0]).to_radians();

    let a: f64 = ((d_lat / 2.0).sin()) * ((d_lat / 2.0).sin())
        + ((d_lon / 2.0).sin()) * ((d_lon / 2.0).sin()) * (lat1.cos()) * (lat2.cos());
    let c: f64 = 2.0 * ((a.sqrt()).atan2((1.0 - a).sqrt()));

    EARTH_RADIUS_IN_KM * c
}

#[test]
fn test_haversine_zero_dist() {
    let loc1 = [37.549521, -121.942765];
    let loc2 = [37.549521, -121.942765];
    assert_eq!(haversine(&loc1, &loc2), 0_f64);
}
