use utm::{lat_lon_to_zone_number, lat_to_zone_letter, to_utm_wgs84};

#[derive(Clone, Debug)]
pub struct Span {
    pub northing_low: f64,
    pub northing_high: f64,
    pub easting_low: f64,
    pub easting_high: f64,
    pub utm_zone_number: u8,
    pub utm_zone_letter: char,
}

impl Span {
    pub const MAX_NORTH_SOUTH_SPAN: f64 = 100_000.0;
    pub const MAX_EAST_WEST_SPAN: f64 = 100_000.0;

    pub fn new(
        southwest_corner: (f64, f64),
        northeast_corner: (f64, f64),
    ) -> Result<Self, SpanError> {
        let (sw_lat, sw_lon) = southwest_corner;
        let (ne_lat, ne_lon) = northeast_corner;
        let center_lat = (sw_lat + ne_lat) / 2.0;
        let center_lon = (sw_lon + ne_lon) / 2.0;
        let zone_number = lat_lon_to_zone_number(center_lat, center_lon);
        let zone_letter = lat_to_zone_letter(center_lat).ok_or(SpanError::LatitudeOutsideRange)?;

        let (northing_low, easting_low, _) = to_utm_wgs84(sw_lat, sw_lon, zone_number);
        let (northing_high, easting_high, _) = to_utm_wgs84(ne_lat, ne_lon, zone_number);

        if northing_low > northing_high || easting_low > easting_high {
            Err(SpanError::MismatchedCorners)
        } else if northing_high - northing_low > Self::MAX_NORTH_SOUTH_SPAN
            || easting_high - easting_low > Self::MAX_EAST_WEST_SPAN
        {
            Err(SpanError::SpanTooLarge)
        } else {
            Ok(Self {
                northing_low,
                northing_high,
                easting_low,
                easting_high,
                utm_zone_number: zone_number,
                utm_zone_letter: zone_letter,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanError {
    LatitudeOutsideRange,
    SpanTooLarge,
    MismatchedCorners,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::utm_coordinates_are_close;

    #[test]
    fn prague_north_east() {
        let sw_corner = (49.9460, 14.1918);
        let ne_corner = (50.1787, 14.7209);
        let span = Span::new(sw_corner, ne_corner).unwrap();
        assert_eq!(span.utm_zone_number, 33);
        assert_eq!(span.utm_zone_letter, 'U');
        assert!(utm_coordinates_are_close(span.northing_low, 5532939.813));
        assert!(utm_coordinates_are_close(span.easting_low, 442014.147));
        assert!(utm_coordinates_are_close(span.northing_high, 5558536.972));
        assert!(utm_coordinates_are_close(span.easting_high, 480071.973));
    }

    #[test]
    fn buenos_aires_south_west() {
        let sw_corner = (-35.0186, -59.2153);
        let ne_corner = (-34.5346, -58.1501);
        let span = Span::new(sw_corner, ne_corner).unwrap();
        assert_eq!(span.utm_zone_number, 21);
        assert_eq!(span.utm_zone_letter, 'H');
        assert!(utm_coordinates_are_close(span.northing_low, 6122651.349));
        assert!(utm_coordinates_are_close(span.easting_low, 297878.626));
        assert!(utm_coordinates_are_close(span.northing_high, 6177965.495));
        assert!(utm_coordinates_are_close(span.easting_high, 394458.174));
    }

    // FIXME: Corners on different sides of the equator do not work correctly
    /*
    #[test]
    fn cayambe_ecuador_four_zones() {
        let sw_corner = (-0.0140, -78.0181);
        let ne_corner = (0.0470, -77.9455);
        let span = Span::new(sw_corner, ne_corner).unwrap();
        assert_eq!(span.utm_zone_number, 18);
        assert_eq!(span.utm_zone_letter, 'N');
        assert!(utm_coordinates_are_close(span.northing_low, -1549.533));
        assert!(utm_coordinates_are_close(span.easting_low, 164004.568));
        assert!(utm_coordinates_are_close(span.northing_high, 5201.831));
        assert!(utm_coordinates_are_close(span.easting_high, 172094.266));
    }
    */

    #[test]
    fn norway_exception() {
        let sw_corner = (61.2769, 4.5676);
        let ne_corner = (61.3390, 4.7955);
        let span = Span::new(sw_corner, ne_corner).unwrap();
        assert_eq!(span.utm_zone_number, 32);
        assert_eq!(span.utm_zone_letter, 'V');
        assert!(utm_coordinates_are_close(span.northing_low, 6801692.808));
        assert!(utm_coordinates_are_close(span.easting_low, 262487.212));
        assert!(utm_coordinates_are_close(span.northing_high, 6807792.137));
        assert!(utm_coordinates_are_close(span.easting_high, 275132.659));
    }

    #[test]
    fn mismatched_corners() {
        let sw_corner = (49.9460, 14.1918);
        let ne_corner = (50.1787, 14.7209);
        assert_eq!(
            Span::new(ne_corner, sw_corner).unwrap_err(),
            SpanError::MismatchedCorners
        );
        assert_eq!(
            Span::new((sw_corner.0, ne_corner.1), (ne_corner.0, sw_corner.1)).unwrap_err(),
            SpanError::MismatchedCorners
        );
        assert_eq!(
            Span::new((ne_corner.0, sw_corner.1), (sw_corner.0, ne_corner.1)).unwrap_err(),
            SpanError::MismatchedCorners
        );
    }

    #[test]
    fn span_too_large() {
        let sw_corner = (48.4437, 12.0538);
        let ne_corner = (51.0137, 18.9446);
        assert_eq!(
            Span::new(sw_corner, ne_corner).unwrap_err(),
            SpanError::SpanTooLarge
        );
    }

    #[test]
    fn latitude_outside_range() {
        let sw_corner = (86.0, -10.0);
        let ne_corner = (86.5, -9.5);
        assert_eq!(
            Span::new(sw_corner, ne_corner).unwrap_err(),
            SpanError::LatitudeOutsideRange
        );
    }
}
