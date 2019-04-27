use crate::shared::generated::improbable::Coordinates;

pub fn squared_distance(c1: &Coordinates, c2: &Coordinates) -> f64 {
    (c1.x - c2.x).powi(2) + (c1.y - c2.y).powi(2) + (c1.z - c2.z).powi(2)
}

pub fn normalized_direction(c1: &Coordinates, c2: &Coordinates) -> Coordinates {
    let mut diff = Coordinates {
        x: c1.x - c2.x,
        y: c1.y - c2.y,
        z: c1.z - c2.z,
    };

    let magnitude = squared_distance(
        &diff,
        &Coordinates {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    )
    .sqrt();

    diff.x = diff.x / magnitude;
    diff.y = diff.y / magnitude;
    diff.z = diff.z / magnitude;

    diff
}

pub fn multiply(c1: &mut Coordinates, n: f64) {
    c1.x = c1.x * n;
    c1.y = c1.y * n;
    c1.z = c1.z * n;
}

pub fn add_coords(c1: &Coordinates, c2: &Coordinates) -> Coordinates {
    Coordinates {
        x: c1.x + c2.x,
        y: c1.y + c2.y,
        z: c1.z + c2.z,
    }
}
