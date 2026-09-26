//! Pure mirror of the shared projected-tube geometry in `background.wgsl`.
//!
//! This is the measurement seam for the world-authored profile. Rendering
//! stays on the GPU; tests use these points to ask the real pixel output for
//! ink at reference landmarks rather than maintaining a second rasterizer.

pub const NEAR_Z: f32 = 0.72;
pub const BODY_Z: f32 = 10.8;
pub const FAR_Z: f32 = 24.0;
pub const FOCAL_FRAC: f32 = 0.72;
pub const RAILS: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn distance(self, other: Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Projection {
    pub width: f32,
    pub height: f32,
    pub vanish: Point,
    pub fold: f32,
    pub twist: f32,
    pub travel_z: f32,
    pub spin: f32,
}

impl Projection {
    pub fn focal(self) -> f32 {
        self.width.min(self.height) * FOCAL_FRAC
    }

    pub fn section_centre(self, z: f32) -> Point {
        let p = ((z - NEAR_Z) / (BODY_Z - NEAR_Z)).clamp(0.0, 1.0);
        let bend = p * p * (3.0 - 2.0 * p);
        Point {
            x: self.width * 0.5 + (self.vanish.x - self.width * 0.5) * bend,
            y: self.height * 0.5 + (self.vanish.y - self.height * 0.5) * bend,
        }
    }

    pub fn tube_centre(self, z: f32) -> Point {
        let centre = self.section_centre(z);
        let path = self.path(z + self.travel_z);
        let scale = self.focal() / z;
        Point {
            x: centre.x + path.x * scale,
            y: centre.y + path.y * scale,
        }
    }

    pub fn path(self, world_z: f32) -> Point {
        Point {
            x: 0.22 * (world_z * 0.48).sin() + 0.07 * (world_z * 1.17).sin(),
            y: 0.17 * (world_z * 0.39).cos() - 0.06 * (world_z * 0.91).sin(),
        }
    }

    pub fn roll(self, world_z: f32) -> f32 {
        0.12 * (world_z * 0.31).sin() + self.spin
    }

    pub fn radius(self, theta: f32, world_z: f32) -> f32 {
        let turn = theta + world_z * self.twist;
        let pulse =
            1.0 + 0.075 * (world_z * 1.25).sin() + 0.035 * (world_z * 2.7 + theta * 2.0).sin();
        let petals =
            self.fold * (0.46 * (3.0 * turn).cos() + 0.18 * (5.0 * turn - world_z * 0.35).sin());
        (1.0 + petals).max(0.46) * pulse
    }

    pub fn point(self, theta: f32, z: f32) -> Point {
        let world_z = z + self.travel_z;
        let path = self.path(world_z);
        let angle = theta + self.roll(world_z);
        let radius = self.radius(theta, world_z);
        let scale = self.focal() / z;
        let centre = self.section_centre(z);
        Point {
            x: centre.x + (path.x + radius * angle.cos()) * scale,
            y: centre.y + (path.y + radius * angle.sin()) * scale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approved() -> Projection {
        Projection {
            width: 1200.0,
            height: 800.0,
            vanish: Point { x: 960.0, y: 192.0 },
            fold: 0.34,
            twist: 0.72,
            travel_z: 0.0,
            spin: 0.0,
        }
    }

    fn section_radii(p: Projection, z: f32) -> Vec<f32> {
        let path_centre = p.tube_centre(z);
        (0..720)
            .map(|i| {
                p.point(std::f32::consts::TAU * i as f32 / 720.0, z)
                    .distance(path_centre)
            })
            .collect()
    }

    #[test]
    fn approved_sections_keep_the_full_multiplicative_radial_span() {
        let _g = crate::testlock::serial();
        for z in [NEAR_Z, 1.5, 3.0, 6.0, FAR_Z] {
            let radii = section_radii(approved(), z);
            let lo = radii.iter().copied().fold(f32::INFINITY, f32::min);
            let hi = radii.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let span = (hi - lo) / ((hi + lo) * 0.5);
            assert!(
                (0.40..=0.48).contains(&span),
                "z={z}: radial span {span:.3}, want the approved 42–45%-class silhouette"
            );
        }
    }

    #[test]
    fn centreline_path_is_depth_dependent_and_independent_of_fold() {
        let _g = crate::testlock::serial();
        let p = approved();
        let straight = Projection { fold: 0.0, ..p };
        for z in [NEAR_Z, 1.5, 3.0, 6.0, FAR_Z] {
            assert_eq!(p.section_centre(z), straight.section_centre(z));
            assert_eq!(p.path(z), straight.path(z));
        }
        assert!(p.section_centre(3.0).distance(p.section_centre(9.0)) > 200.0);
        let path_extent = p.path(3.0).distance(p.path(9.0));
        assert!(path_extent > 0.20, "path extent {path_extent:.3}");
    }

    #[test]
    fn section_roll_and_fixed_theta_longitudinal_curvature_are_real() {
        let _g = crate::testlock::serial();
        let p = approved();
        assert!((p.roll(1.5) - p.roll(6.0)).abs() > 0.05);
        let a = p.point(0.37, 1.5);
        let b = p.point(0.37, 6.0);
        let mid = p.point(0.37, 3.75);
        let chord_mid = Point {
            x: (a.x + b.x) * 0.5,
            y: (a.y + b.y) * 0.5,
        };
        assert!(
            mid.distance(chord_mid) > 25.0,
            "fixed-theta rib collapsed toward a straight chord"
        );
    }

    #[test]
    fn transit_sections_do_not_collapse_to_concentric_targets() {
        let _g = crate::testlock::serial();
        let p = Projection {
            vanish: Point { x: 600.0, y: 400.0 },
            ..approved()
        };
        let near = p.tube_centre(1.5);
        let far = p.tube_centre(9.0);
        assert!(
            near.distance(far) > 25.0,
            "path-owned section centres still move in transit"
        );
        let radii = section_radii(p, 3.0);
        let lo = radii.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = radii.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!((hi - lo) / ((hi + lo) * 0.5) > 0.40);
        assert_eq!(
            RAILS, 24,
            "the longitudinal roster is independent of 58 sections"
        );
    }
}
