use proj::{Proj, ProjCreateError, ProjError};

use crate::location::GeodeticLocation;

#[derive(Clone, Debug)]
pub struct Span {
    pub x_low: f64,
    pub x_high: f64,
    pub y_low: f64,
    pub y_high: f64,
    pub projection: Proj,
}

impl Span {
    pub fn new(
        southwest_corner: GeodeticLocation,
        northeast_corner: GeodeticLocation,
        projection_string: &str,
    ) -> Result<Self, SpanCreationError> {
        if (southwest_corner.latitude() > northeast_corner.latitude()
            || southwest_corner.longitude() > northeast_corner.longitude())
        {
            return Err(SpanCreationError::MismatchedCorners);
        }
        let projection =
            Proj::new(projection_string).map_err(SpanCreationError::ProjectionCreation)?;
        let (x_low, y_low) = projection
            .project(southwest_corner.as_lon_lat_radians(), false)
            .map_err(SpanCreationError::Projection)?;
        let (x_high, y_high) = projection
            .project(northeast_corner.as_lon_lat_radians(), false)
            .map_err(SpanCreationError::Projection)?;
        Ok(Self {
            x_low,
            x_high,
            y_low,
            y_high,
            projection,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanCreationError {
    ProjectionCreation(ProjCreateError),
    Projection(ProjError),
    MismatchedCorners,
}

// TODO: tests for new projection code
/*#[cfg(test)]
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
        assert!(utm_coordinates_are_close(span.y_low, 5532939.813));
        assert!(utm_coordinates_are_close(span.x_low, 442014.147));
        assert!(utm_coordinates_are_close(span.y_high, 5558536.972));
        assert!(utm_coordinates_are_close(span.x_high, 480071.973));
    }

    #[test]
    fn buenos_aires_south_west() {
        let sw_corner = (-35.0186, -59.2153);
        let ne_corner = (-34.5346, -58.1501);
        let span = Span::new(sw_corner, ne_corner).unwrap();
        assert_eq!(span.utm_zone_number, 21);
        assert_eq!(span.utm_zone_letter, 'H');
        assert!(utm_coordinates_are_close(span.y_low, 6122651.349));
        assert!(utm_coordinates_are_close(span.x_low, 297878.626));
        assert!(utm_coordinates_are_close(span.y_high, 6177965.495));
        assert!(utm_coordinates_are_close(span.x_high, 394458.174));
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
        assert!(utm_coordinates_are_close(span.y_low, 6801692.808));
        assert!(utm_coordinates_are_close(span.x_low, 262487.212));
        assert!(utm_coordinates_are_close(span.y_high, 6807792.137));
        assert!(utm_coordinates_are_close(span.x_high, 275132.659));
    }

    #[test]
    fn mismatched_corners() {
        let sw_corner = (49.9460, 14.1918);
        let ne_corner = (50.1787, 14.7209);
        assert_eq!(
            Span::new(ne_corner, sw_corner).unwrap_err(),
            SpanCreationError::MismatchedCorners
        );
        assert_eq!(
            Span::new((sw_corner.0, ne_corner.1), (ne_corner.0, sw_corner.1)).unwrap_err(),
            SpanCreationError::MismatchedCorners
        );
        assert_eq!(
            Span::new((ne_corner.0, sw_corner.1), (sw_corner.0, ne_corner.1)).unwrap_err(),
            SpanCreationError::MismatchedCorners
        );
    }

    #[test]
    fn span_too_large() {
        let sw_corner = (48.4437, 12.0538);
        let ne_corner = (51.0137, 18.9446);
        assert_eq!(
            Span::new(sw_corner, ne_corner).unwrap_err(),
            SpanCreationError::SpanTooLarge
        );
    }

    #[test]
    fn latitude_outside_range() {
        let sw_corner = (86.0, -10.0);
        let ne_corner = (86.5, -9.5);
        assert_eq!(
            Span::new(sw_corner, ne_corner).unwrap_err(),
            SpanCreationError::LatitudeOutsideRange
        );
    }
}
*/
