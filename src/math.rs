use crate::Location;

const EARTH_RADIUS_KM: i32 = 6371;

/// Returns distance in kilometeres between location1 and location2, which represent
/// latitude and longitude coordinates of two points on Earth.
pub(crate) fn haversine(location1: Location, location2: Location) -> i32 {
    let location1_lat_radians = location1.latitude.to_radians();
    let location2_lat_radians = location2.longitude.to_radians();
    let location_lat_diff_radians = (location2.latitude - location1.latitude).to_radians();
    let location_long_diff_radians = (location2.longitude - location1.longitude).to_radians();
    let central_angle_inner = (location_lat_diff_radians / 2.0).sin().powi(2)
        + location1_lat_radians.cos()
            * location2_lat_radians.cos()
            * (location_long_diff_radians / 2.0).sin().powi(2);
    let central_angle = 2.0 * central_angle_inner.sqrt().asin();

    let distance = earth_radius_kilometer * central_angle;
    distance
}
