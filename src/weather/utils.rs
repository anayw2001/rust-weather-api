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
