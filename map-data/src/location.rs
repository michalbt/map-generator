use std::ops::{Add, Mul, Neg, Sub};

use proj::ProjError;

use crate::span::Span;

/// Location of a point on the Earth surface. Both coordinates are in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeodeticLocation {
    latitude: f64,
    longitude: f64,
}

impl GeodeticLocation {
    pub const MIN_LATITUDE: f64 = -90.0;
    pub const MAX_LATITUDE: f64 = 90.0;
    pub const MIN_LONGITUDE: f64 = -180.0;
    pub const MAX_LONGITUDE: f64 = 180.0;

    pub fn new(latitude: f64, longitude: f64) -> Result<Self, GeodeticLocationCreateError> {
        if !(Self::MIN_LATITUDE..=Self::MAX_LATITUDE).contains(&latitude) {
            Err(GeodeticLocationCreateError::LatitudeOutsideRange(latitude))
        } else if !(Self::MIN_LONGITUDE..=Self::MAX_LONGITUDE).contains(&longitude) {
            Err(GeodeticLocationCreateError::LongitudeOutsideRange(
                longitude,
            ))
        } else {
            Ok(Self {
                latitude,
                longitude,
            })
        }
    }

    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    pub fn as_lon_lat_radians(&self) -> (f64, f64) {
        (self.longitude.to_radians(), self.latitude.to_radians())
    }
}

/// Location on a map. The X axis runs from left to right and the Y axis runs from bottom to top.
/// Location of the origin relative to the map is not specified. The scale of the units is also not specified,
/// however, it should be the same on both axes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocationOffset {
    pub x: f64,
    pub y: f64,
}

impl Location {
    pub fn from_geodetic(
        geodetic_location: GeodeticLocation,
        map_span: &Span,
    ) -> Result<Self, LocationProjectionError> {
        map_span
            .projection
            .project(geodetic_location.as_lon_lat_radians(), false)
            .map(|(x, y)| Location { x, y })
            .map_err(LocationProjectionError::Projection)
    }

    pub fn to_geodetic(
        self,
        map_span: &Span,
    ) -> Result<GeodeticLocation, LocationInverseProjectionError> {
        map_span
            .projection
            .project((self.x, self.y), true)
            .map_err(LocationInverseProjectionError::Projection)
            .and_then(|(lon, lat)| {
                GeodeticLocation::new(lat, lon)
                    .map_err(LocationInverseProjectionError::GeodeticLocationCreation)
            })
    }
}

impl LocationOffset {
    pub fn new(y: f64, x: f64) -> Self {
        Self { y, x }
    }

    pub fn length(&self) -> f64 {
        (self.y * self.y + self.x * self.x).sqrt()
    }

    /// Get a direction in the range [-pi, +pi] such that north is 0 and east is +pi/2.
    pub fn direction(&self) -> f64 {
        f64::atan2(self.x, self.y)
    }
}

impl Add<LocationOffset> for Location {
    type Output = Location;

    fn add(self, rhs: LocationOffset) -> Self::Output {
        Location {
            y: self.y + rhs.y,
            x: self.x + rhs.x,
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
            y: self.y + rhs.y,
            x: self.x + rhs.x,
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
            y: -self.y,
            x: -self.x,
        }
    }
}

impl Mul<f64> for LocationOffset {
    type Output = LocationOffset;

    fn mul(self, rhs: f64) -> Self::Output {
        LocationOffset {
            y: rhs * self.y,
            x: rhs * self.x,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GeodeticLocationCreateError {
    LatitudeOutsideRange(f64),
    LongitudeOutsideRange(f64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LocationProjectionError {
    Projection(ProjError),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LocationInverseProjectionError {
    Projection(ProjError),
    GeodeticLocationCreation(GeodeticLocationCreateError),
}

#[cfg(test)]
pub(crate) fn utm_coordinates_are_close(left: f64, right: f64) -> bool {
    const EPSILON: f64 = 1e-2;
    return (left - right).abs() < EPSILON;
}

#[cfg(test)]
pub(crate) fn lat_lon_coordinates_are_close(left: f64, right: f64) -> bool {
    const EPSILON: f64 = 1e-7;
    return (left - right).abs() < EPSILON;
}

// TODO: tests for new projection code
/*#[cfg(test)]
mod tests {
    use super::*;
    use std::{assert_matches, f64::consts::PI};

    #[test]
    fn lat_lon_to_utm_and_back() {
        let span = Span::new((49.9460, 14.1918), (50.1787, 14.7209)).unwrap();
        let lat = 50.0881;
        let lon = 14.4044;
        let location = Location::from_latitude_and_longitude(lat, lon, &span).unwrap();
        assert!(utm_coordinates_are_close(location.y, 5548596.002));
        assert!(utm_coordinates_are_close(location.x, 457393.278));
        let (new_lat, new_lon) = location.to_latitude_and_longitude(&span).unwrap();
        assert!(lat_lon_coordinates_are_close(new_lat, lat));
        assert!(lat_lon_coordinates_are_close(new_lon, lon));
    }

    #[test]
    fn lat_lon_to_utm_errors() {
        let span = Span::new((49.9460, 14.1918), (50.1787, 14.7209)).unwrap();
        assert_eq!(
            Location::from_latitude_and_longitude(-95.0, 30.0, &span),
            Err(LocationProjectionError::LatitudeOutsideRange(-95.0))
        );
        assert_eq!(
            Location::from_latitude_and_longitude(10.0, 190.0, &span),
            Err(LocationProjectionError::LongitudeOutsideRange(190.0))
        );
    }

    #[test]
    fn utm_to_lat_lon_errors() {
        let span = Span::new((49.9460, 14.1918), (50.1787, 14.7209)).unwrap();
        let location = Location::from_latitude_and_longitude(50.0881, 14.4044, &span).unwrap();
        let far_north = location + LocationOffset::new(7_000_000.0, 0.0);
        assert_matches!(
            far_north.to_latitude_and_longitude(&span),
            Err(LocationInverseProjectionError::NorthingOutsideRange(_))
        );
        let far_west = location + LocationOffset::new(0.0, -1_000_000.0);
        assert_matches!(
            far_west.to_latitude_and_longitude(&span),
            Err(LocationInverseProjectionError::EastingOutsideRange(_))
        );
        let invalid_number_span = Span {
            utm_zone_number: 61,
            ..span
        };
        assert_eq!(
            location.to_latitude_and_longitude(&invalid_number_span),
            Err(LocationInverseProjectionError::InvalidZoneNumber(61))
        );
        let invalid_letter_span = Span {
            utm_zone_letter: '#',
            ..span
        };
        assert_eq!(
            location.to_latitude_and_longitude(&invalid_letter_span),
            Err(LocationInverseProjectionError::InvalidZoneLetter('#'))
        );
    }

    #[test]
    fn location_operations() {
        let location = Location::new(20.0, 30.0);
        let offset = LocationOffset::new(5.0, 2.0);
        let other_offset = LocationOffset::new(-3.0, 4.0);
        assert_eq!(location + offset, Location::new(25.0, 32.0));
        assert_eq!(location - offset, Location::new(15.0, 28.0));
        assert_eq!(-offset, LocationOffset::new(-5.0, -2.0));
        assert_eq!(offset + other_offset, LocationOffset::new(2.0, 6.0));
        assert_eq!(offset - other_offset, LocationOffset::new(8.0, -2.0));
        assert!(other_offset.length() - 5.0 < 1e-9);
        assert!(other_offset.direction() - (90.0 + (3.0f64 / 4.0).atan() * 180.0 / PI) < 1e-9);
    }
}
*/
