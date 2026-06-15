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
