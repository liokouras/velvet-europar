#[derive(Debug, Clone, Copy)]
pub struct Quadrant {
    x_mid: f64,
    y_mid: f64,
    length: f64,
}

impl Quadrant {
    pub(super) const fn new(x_mid: f64, y_mid: f64, length: f64) -> Self {
        Self { x_mid, y_mid, length }
    }
    
    // returns true if (x, y) is in the region, and false otherwise.
    pub(super) fn contains(&self, x: f64, y:f64) -> bool {
        let half = self.length / 2.;

        x <= (self.x_mid + half) && 
        x >= (self.x_mid - half) &&
        y <= (self.y_mid + half) && 
        y >= (self.y_mid - half)
    }
    
    pub(super) fn length(&self) -> f64 {
        self.length
    }
    
    // these four methods create and return the sub-quadrant of this quadrant
    pub(super) fn nw(&self) -> Self {
        Self {
            x_mid: self.x_mid - self.length / 4.,
            y_mid: self.y_mid + self.length / 4.,
            length: self.length / 2.,
        }
    }

    pub(super) fn ne(&self) -> Self {
        Self {
            x_mid: self.x_mid + self.length / 4.,
            y_mid: self.y_mid + self.length / 4.,
            length: self.length / 2.,
        }
        
    }

    pub(super) fn sw(&self) -> Self {
        Self {
            x_mid: self.x_mid - self.length / 4.,
            y_mid: self.y_mid - self.length / 4.,
            length: self.length / 2.,
        }
        
    }

    pub(super) fn se(&self) -> Self {
        Self {
            x_mid: self.x_mid + self.length / 4.,
            y_mid: self.y_mid - self.length / 4.,
            length: self.length / 2.,
        }
    }
}
