use utm::to_utm_wgs84;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    northing: f64,
    easting: f64,
}

impl Location {
    pub const MIN_LATITUDE: f64 = -90.0;
    pub const MAX_LATITUDE: f64 = 90.0;
    pub const MIN_LONGITUDE: f64 = -180.0;
    pub const MAX_LONGITUDE: f64 = 180.0;

    pub fn new(northing: f64, easting: f64) -> Self {
        Self { northing, easting }
    }

    pub fn from_latitude_and_longitude(
        latitude: f64,
        longitude: f64,
        zone_number: u8,
    ) -> Result<Self, LocationError> {
        if (Self::MIN_LATITUDE..=Self::MAX_LATITUDE).contains(&latitude)
            && (Self::MIN_LONGITUDE..=Self::MAX_LONGITUDE).contains(&longitude)
        {
            let (northing, easting, _) = to_utm_wgs84(latitude, longitude, zone_number);
            Ok(Self::new(northing, easting))
        } else {
            Err(LocationError::ValueOutsideRange)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocationError {
    ValueOutsideRange,
}
