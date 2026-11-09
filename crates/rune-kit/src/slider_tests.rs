use super::*;

fn geo(min: f32, max: f32, step: f32) -> Geometry {
    Geometry { value: min, min, max, step, knob: 16.0 }
}

#[test]
fn values_clamp_and_snap_from_the_minimum() {
    let g = geo(1.0, 3.0, 0.5);
    assert_eq!(g.snap(0.0), 1.0);
    assert_eq!(g.snap(9.0), 3.0);
    assert_eq!(g.snap(1.74), 1.5);
    assert_eq!(g.snap(1.76), 2.0);
    assert_eq!(geo(0.0, 1.0, 0.0).snap(0.333), 0.333, "continuous");
}

#[test]
fn the_pointer_maps_through_the_knobs_travel() {
    let g = geo(0.0, 1.0, 0.0);
    assert_eq!(g.at(8.0, 168.0), 0.0, "knob centre at the left end");
    assert_eq!(g.at(160.0, 168.0), 1.0, "knob centre at the right end");
    assert_eq!(g.at(84.0, 168.0), 0.5);
    assert_eq!(g.knob_x(0.5, 168.0), 84.0);
}

#[test]
fn keys_move_a_step_or_a_hundredth() {
    assert_eq!(geo(0.0, 10.0, 2.0).key_step(), 2.0);
    assert_eq!(geo(0.0, 10.0, 0.0).key_step(), 0.1);
}

#[test]
fn numbers_read_with_the_steps_decimals() {
    assert_eq!(format_number(3.0, 1.0), "3");
    assert_eq!(format_number(0.25, 0.05), "0.25");
    assert_eq!(format_number(1.5, 0.5), "1.5");
    assert_eq!(format_number(1.0 / 3.0, 0.0), "0.33");
}
