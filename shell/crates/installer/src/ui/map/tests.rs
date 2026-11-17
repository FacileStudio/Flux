use super::*;

fn point(zone: &str, latitude: f64, longitude: f64) -> ZonePoint {
    ZonePoint { zone: zone.to_string(), latitude, longitude }
}

#[test]
fn the_map_ships_land() {
    let rings = land();
    assert!(rings.len() > 100, "Natural Earth 110m has about 128 land rings, got {}", rings.len());
    assert!(rings.iter().flatten().all(|(lon, lat)| (-180.0..=180.0).contains(lon) && (-90.0..=90.0).contains(lat)));
}

#[test]
fn a_click_picks_the_nearest_city() {
    let points = [point("Europe/Paris", 48.86, 2.33), point("America/New_York", 40.71, -74.0), point("Asia/Tokyo", 35.65, 139.74)];
    let projection = Projection::new(720.0, 284.0);
    let (x, y) = projection.point(3.0, 47.0);
    assert_eq!(nearest(&points, projection, x, y).as_deref(), Some("Europe/Paris"));
    let (x, y) = projection.point(-70.0, 42.0);
    assert_eq!(nearest(&points, projection, x, y).as_deref(), Some("America/New_York"));
    assert_eq!(nearest(&[], projection, x, y), None);
}

#[test]
fn the_projection_keeps_its_proportions_and_centres() {
    let wide = Projection::new(1000.0, 284.0);
    let (left, _) = wide.point(-180.0, 0.0);
    let (right, _) = wide.point(180.0, 0.0);
    assert!((left - (1000.0 - right)).abs() < 1e-6, "the map is centred");
    let (_, top) = wide.point(0.0, NORTH);
    assert!(top.abs() < 1e-6);
}
