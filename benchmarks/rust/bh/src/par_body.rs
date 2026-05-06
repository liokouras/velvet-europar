use std::fmt;
use super::quad::Quadrant;

#[derive(Clone, Debug)]
pub struct Body {
    id: usize,
    
    mass: f64,

    // position
    px: f64,
    py: f64,

    // velocity
    vx: f64,
    vy: f64,

    // force
    fx: f64,
    fy: f64,
}

impl Body {
    // create a new initialised  body
    pub(super) fn new(id: usize, mass: f64, px: f64, py: f64, vx: f64, vy: f64) -> Self {
        Self { id, mass, px, py, vx, vy, fx: 0., fy: 0., }
    }

    pub(super) fn id(&self) -> usize {
        self.id
    }
    pub(super) fn mass(&self) -> f64 {
        self.mass
    }
    pub(super) fn px(&self) -> f64 {
        self.px
    }
    pub(super) fn py(&self) -> f64 {
        self.py
    }
    
    pub(super) fn set_force(&mut self, fx: f64, fy: f64) {
        self.fx = fx;
        self.fy = fy;
    }
    // update the velocity and position of the invoking body and resets the forces
    // dt is the timestep
    pub(super) fn update(&mut self, dt: f64) {
        self.vx += dt * self.fx / self.mass;
        self.vy += dt * self.fy / self.mass;
        self.px += dt * self.vx;
        self.py += dt * self.vy;
        self.fx = 0.;
        self.fy = 0.;
    }

    // returns true if the body is in quadrant q.
    pub(super) fn inside(&self, q: &Quadrant) -> bool {
        q.contains(self.px, self.py)
    }
}

impl fmt::Display for Body {
    // for the f64s: %10.3E %10.3E %10.3E %10.3E %10.3E
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} {} {} {}", self.id, self.px, self.py, self.vx, self.vy, self.mass)
    }
}