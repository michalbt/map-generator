use std::ops::{Add, Mul, Neg, Sub};

use utm::{WSG84ToLatLonError, to_utm_wgs84, wsg84_utm_to_lat_lon};

use crate::span::Span;

/// Map location in the [UTM coordinate system](https://en.wikipedia.org/wiki/Universal_Transverse_Mercator_coordinate_system)
/// without a zone number and letter, which are implied and stored in the [`Storage`](crate::storage::Storage) struct as a
/// [`Span`](crate::span::Span).
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
    pub fn new(northing: f64, easting: f64) -> Self {
        Self { northing, easting }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{assert_matches, f64::consts::PI};

    #[test]
    fn lat_lon_to_utm_and_back() {
        let span = Span::new((49.9460, 14.1918), (50.1787, 14.7209)).unwrap();
        let lat = 50.0881;
        let lon = 14.4044;
        let location = Location::from_latitude_and_longitude(lat, lon, &span).unwrap();
        assert!(utm_coordinates_are_close(location.northing, 5548596.002));
        assert!(utm_coordinates_are_close(location.easting, 457393.278));
        let (new_lat, new_lon) = location.to_latitude_and_longitude(&span).unwrap();
        assert!(lat_lon_coordinates_are_close(new_lat, lat));
        assert!(lat_lon_coordinates_are_close(new_lon, lon));
    }

    #[test]
    fn lat_lon_to_utm_errors() {
        let span = Span::new((49.9460, 14.1918), (50.1787, 14.7209)).unwrap();
        assert_eq!(
            Location::from_latitude_and_longitude(-95.0, 30.0, &span),
            Err(LocationFromLatitudeAndLongitudeError::LatitudeOutsideRange(
                -95.0
            ))
        );
        assert_eq!(
            Location::from_latitude_and_longitude(10.0, 190.0, &span),
            Err(LocationFromLatitudeAndLongitudeError::LongitudeOutsideRange(190.0))
        );
    }

    #[test]
    fn utm_to_lat_lon_errors() {
        let span = Span::new((49.9460, 14.1918), (50.1787, 14.7209)).unwrap();
        let location = Location::from_latitude_and_longitude(50.0881, 14.4044, &span).unwrap();
        let far_north = location + LocationOffset::new(7_000_000.0, 0.0);
        assert_matches!(
            far_north.to_latitude_and_longitude(&span),
            Err(LocationToLatitudeAndLongitudeError::NorthingOutsideRange(_))
        );
        let far_west = location + LocationOffset::new(0.0, -1_000_000.0);
        assert_matches!(
            far_west.to_latitude_and_longitude(&span),
            Err(LocationToLatitudeAndLongitudeError::EastingOutsideRange(_))
        );
        let invalid_number_span = Span {
            utm_zone_number: 61,
            ..span
        };
        assert_eq!(
            location.to_latitude_and_longitude(&invalid_number_span),
            Err(LocationToLatitudeAndLongitudeError::InvalidZoneNumber(61))
        );
        let invalid_letter_span = Span {
            utm_zone_letter: '#',
            ..span
        };
        assert_eq!(
            location.to_latitude_and_longitude(&invalid_letter_span),
            Err(LocationToLatitudeAndLongitudeError::InvalidZoneLetter('#'))
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
