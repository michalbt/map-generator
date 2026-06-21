use std::ops::{Add, Mul, Neg, Sub};

use utm::{WSG84ToLatLonError, to_utm_wgs84, wsg84_utm_to_lat_lon};

use crate::span::Span;

/// Map location in the [UTM coordinate system](https://en.wikipedia.org/wiki/Universal_Transverse_Mercator_coordinate_system)
/// without a zone number and letter, which are implied and stored in the [`Storage`](crate::storage::Storage) struct.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    pub northing: f64,
    pub easting: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocationOffset {
    pub northing: f64,
    pub easting: f64,
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
        map_span: &Span,
    ) -> Result<Self, LocationFromLatitudeAndLongitudeError> {
        if !(Self::MIN_LATITUDE..=Self::MAX_LATITUDE).contains(&latitude) {
            Err(LocationFromLatitudeAndLongitudeError::LatitudeOutsideRange(
                latitude,
            ))
        } else if !(Self::MIN_LONGITUDE..=Self::MAX_LONGITUDE).contains(&longitude) {
            Err(LocationFromLatitudeAndLongitudeError::LongitudeOutsideRange(longitude))
        } else {
            let (northing, easting, _) =
                to_utm_wgs84(latitude, longitude, map_span.utm_zone_number);
            Ok(Self::new(northing, easting))
        }
    }

    pub fn to_latitude_and_longitude(
        self,
        map_span: &Span,
    ) -> Result<(f64, f64), LocationToLatitudeAndLongitudeError> {
        wsg84_utm_to_lat_lon(
            self.easting,
            self.northing,
            map_span.utm_zone_number,
            map_span.utm_zone_letter,
        )
        .map_err(|err| match err {
            WSG84ToLatLonError::NorthingOutOfRange => {
                LocationToLatitudeAndLongitudeError::NorthingOutsideRange(self.northing)
            }
            WSG84ToLatLonError::EastingOutOfRange => {
                LocationToLatitudeAndLongitudeError::EastingOutsideRange(self.easting)
            }
            WSG84ToLatLonError::ZoneNumOutOfRange => {
                LocationToLatitudeAndLongitudeError::InvalidZoneNumber(map_span.utm_zone_number)
            }
            WSG84ToLatLonError::ZoneLetterOutOfRange => {
                LocationToLatitudeAndLongitudeError::InvalidZoneLetter(map_span.utm_zone_letter)
            }
        })
    }
}

impl LocationOffset {
    pub fn length(&self) -> f64 {
        (self.northing * self.northing + self.easting * self.easting).sqrt()
    }

    /// Get a direction in the range [-pi, +pi] such that north is 0 and east is +pi/2.
    pub fn direction(&self) -> f64 {
        f64::atan2(self.easting, self.northing)
    }
}

impl Add<LocationOffset> for Location {
    type Output = Location;

    fn add(self, rhs: LocationOffset) -> Self::Output {
        Location {
            northing: self.northing + rhs.northing,
            easting: self.easting + rhs.easting,
        }
    }
}

impl Sub<LocationOffset> for Location {
    type Output = Location;

    fn sub(self, rhs: LocationOffset) -> Self::Output {
        self + (-rhs)
    }
}

impl Add for LocationOffset {
    type Output = LocationOffset;

    fn add(self, rhs: Self) -> Self::Output {
        LocationOffset {
            northing: self.northing + rhs.northing,
            easting: self.easting + rhs.easting,
        }
    }
}

impl Sub for LocationOffset {
    type Output = LocationOffset;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl Neg for LocationOffset {
    type Output = LocationOffset;

    fn neg(self) -> Self::Output {
        LocationOffset {
            northing: -self.northing,
            easting: -self.easting,
        }
    }
}

impl Mul<f64> for LocationOffset {
    type Output = LocationOffset;

    fn mul(self, rhs: f64) -> Self::Output {
        LocationOffset {
            northing: rhs * self.northing,
            easting: rhs * self.easting,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LocationFromLatitudeAndLongitudeError {
    LatitudeOutsideRange(f64),
    LongitudeOutsideRange(f64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LocationToLatitudeAndLongitudeError {
    NorthingOutsideRange(f64),
    EastingOutsideRange(f64),
    InvalidZoneNumber(u8),
    InvalidZoneLetter(char),
}
