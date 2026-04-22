#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    latitude: f64,
    longitude: f64,
}

impl Location {
    pub const MIN_LATITUDE: f64 = -90.0;
    pub const MAX_LATITUDE: f64 = 90.0;
    pub const MIN_LONGITUDE: f64 = -180.0;
    pub const MAX_LONGITUDE: f64 = 180.0;

    pub fn new(latitude: f64, longitude: f64) -> Result<Location, LocationError> {
        if (Self::MIN_LATITUDE..=Self::MAX_LATITUDE).contains(&latitude)
            && (Self::MIN_LONGITUDE..=Self::MAX_LONGITUDE).contains(&longitude)
        {
            Ok(Location {
                latitude,
                longitude,
            })
        } else {
            Err(LocationError::ValueOutsideRange)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocationError {
    ValueOutsideRange,
}
