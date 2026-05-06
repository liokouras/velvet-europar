use std::{mem, fmt};
use super::{G, THETA, LEAF_CAP, seq_body::Body, quad::Quadrant};

type TreeBody = (usize, f64, f64, f64); // Body ID, mass, px, py [might be aggregate!]

#[derive(Debug)]
pub(super) enum TreeNode {
    Leaf(Vec<TreeBody>), // if vec is empty: uninit
    Aggregate((f64, f64, f64, Vec<BHTree>)), // mass, px, py, children [NW, NE, SW, SE]
}

#[derive(Debug)]
pub(super) struct BHTree {
    body: TreeNode, // the body at the root of this (sub)tree; Leaf (Body ID), Aggregate (mass-px-py), or uninitialised
    quad: Quadrant, // quadrant that this Tree represents
}

impl BHTree {
    pub(super) fn new(quad: Quadrant) -> Self {
       Self { quad, body: TreeNode::Leaf(Vec::with_capacity(LEAF_CAP)) }
    }

    fn append_leaf(&mut self, body: TreeBody) {
        if let TreeNode::Leaf(ref mut bodies) = self.body {
            bodies.push(body); // assumes space !
        }
    }

    // only called on full leaf
    fn split_leaf(&mut self) -> Vec<BHTree> {
        let nw_quad = self.quad.nw();
        let mut nw = BHTree::new(nw_quad);

        let ne_quad = self.quad.ne();
        let mut ne = BHTree::new(ne_quad);

        let sw_quad = self.quad.sw();
        let mut sw = BHTree::new(sw_quad);

        let se_quad = self.quad.se();
        let mut se = BHTree::new(se_quad);

        if let TreeNode::Leaf(ref mut bodies) = self.body {
            for body in bodies.iter() {
                if nw_quad.contains(body.2, body.3) {
                    nw.append_leaf(*body);
                } else if ne_quad.contains(body.2, body.3) {
                    ne.append_leaf(*body);
                } else if sw_quad.contains(body.2, body.3) {
                    sw.append_leaf(*body);
                } else if se_quad.contains(body.2, body.3) {
                    se.append_leaf(*body);
                }
            }
        }

        vec![nw, ne, sw, se]
    }
}

// --------- COMBINED INSERTION + AGGREGATION CALC ----------
impl BHTree {
    // add the body b to the invoking Barnes-Hut tree
    pub(super) fn insert(&mut self, body_info: TreeBody) {
        match self.body {
            TreeNode::Leaf(ref mut bodies) => {
                // adding a body to a Leaf node;
                if bodies.len() < bodies.capacity() {
                    // append body
                    bodies.push(body_info);
                } else {
                    // split leaf
                    // first compute aggregated center-of-mass and total mass
                    let (m, x, y) = self.compute_leaf_aggregate(body_info);

                    // subdivide the region further by creating four children
                    // also distributes current bodies into the children
                    let children = self.split_leaf();

                    // replace self.body with aggregate node
                    let _ = mem::replace(&mut self.body, TreeNode::Aggregate((m, x, y, children)));
                    // recursively insert the body into the appropriate quadrant
                    self.put_body(body_info);
                }
            },
            TreeNode::Aggregate((ref mut mass, ref mut px, ref mut py, _)) => {
                // if this node is already an aggregate, update the center-of-mass and total mass
                let m = *mass + body_info.1;
                let x = ((*px * *mass) + (body_info.2 * body_info.1)) / m;
                let y = ((*py * *mass) + (body_info.3 * body_info.1)) / m;
                *mass = m;
                *px = x;
                *py = y;
            
                // recursively insert Body b into the appropriate quadrant
                self.put_body(body_info);
            },
        }
    }

    // insert a body into the appropriate quadrant
    fn put_body(&mut self, body_info: TreeBody) {
        if let TreeNode::Aggregate((_, _, _, ref mut children)) = self.body {
            for child in children {
                if child.quad.contains(body_info.2, body_info.3) {
                    child.insert(body_info);
                    return;
                }
            }
        }
    }

    fn compute_leaf_aggregate(&self, (_, mut mass, mut px, mut py): TreeBody) -> (f64, f64, f64) {
        if let TreeNode::Leaf(bodies) = &self.body {
            px *= mass;
            py *= mass;
            for body in bodies {
                // accumulate mass, px and py
                mass += body.1;
                px += body.2 * body.1;
                py += body.3 * body.1;
            }
            px /= mass;
            py /= mass;
            (mass, px, py)
        } else {
            (0., 0., 0.)
        }
    }
}

// --------- SEPARATE FORCE COMPUTATION + UPDATE ---------
impl BHTree {
    pub(super) fn compute_force(&self, body: &Body) -> (f64, f64) {
        let eps: f64 = 30000.;
        match self.body {
            TreeNode::Leaf(ref bodies) => {
                let (mut fx, mut fy) = (0., 0.);
                for leaf_body in bodies {
                    if leaf_body.0 != body.id() {
                        // if the current node is a different leaf, compute the net force acting on b
                        let dx = leaf_body.2 - body.px();
                        let dy = leaf_body.3 - body.py();
                        let sq = dx*dx + dy*dy;
                        let dist = sq.sqrt();
                        let f = (G * body.mass() * leaf_body.1) / (dist*dist + eps*eps);
                        fx += f * dx / dist;
                        fy += f * dy / dist;
                    }
                }
                return (fx, fy);
            },
            TreeNode::Aggregate((mass, px, py, ref children)) => {
                // compute Euclidean distance between body and center of aggregate
                let s = self.quad.length();
                let dx = px - body.px();
                let dy = py - body.py();
                let sq = dx*dx + dy*dy;
                let dist = sq.sqrt();
                let ratio = s / dist;

                // compare ratio to threshold value Theta
                if ratio < THETA {
                    // b is far away, use aggregate
                    let f = (G * body.mass() * mass) / (dist*dist + eps*eps);
                    let fx = f * dx / dist;
                    let fy = f * dy / dist;
                    return (fx, fy);
                } else {
                    // b is close, recurse
                    let (x0, y0) = children[0].compute_force(body);
                    let (x1, y1) = children[1].compute_force(body);
                    let (x2, y2) = children[2].compute_force(body);
                    let (x3, y3) = children[3].compute_force(body);
                    let fx = x0 + x1 + x2 + x3;
                    let fy = y0 + y1 + y2 + y3;
                    return (fx, fy);
                }
            }
        }
    }

    // traverses the tree and calls compute_force on each leaf
    pub(super) fn traverse_update_force(&self, root: &BHTree, bodies: &mut Vec<Body>) {
        match self.body {
            TreeNode::Leaf(ref leaf_bodies) => {
                for body in leaf_bodies {
                    let id = body.0;
                    let (fx, fy) = root.compute_force(&bodies[id]);
                    bodies[id].set_force(fx, fy);
                }
            },
            TreeNode::Aggregate((_, _, _, ref children)) => {
                for child in children {
                    child.traverse_update_force(root, bodies);
                }
            }
        }
    }
}

// --------- DISPLAYING ---------
impl BHTree {
    fn fmt_tree(&self, f: &mut std::fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        let indent = "  ".repeat(depth);
        match &self.body {
            TreeNode::Leaf(bodies) => {
                write!(f, " - LEAF [")?;
                for body in bodies {
                    write!(f, " {}", body.0)?;
                }
                writeln!(f," ] ")
            },
            TreeNode::Aggregate((_, px, py, children)) => {
                writeln!(f, " AGG ({}, {})", px, py)?;
                write!(f, "{}NW ", indent)?;
                children[0].fmt_tree(f, depth + 1)?;
                write!(f, "{}NE ", indent)?;
                children[1].fmt_tree(f, depth + 1)?;
                write!(f, "{}SW ", indent)?;
                children[2].fmt_tree(f, depth + 1)?;
                write!(f, "{}SE ", indent)?;
                children[3].fmt_tree(f, depth + 1)
            },
        }
    }
}

impl fmt::Display for BHTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_tree(f, 0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_tree() {
        let quad = Quadrant::new(0., 0., 4.);
        let mut tree = BHTree::new(quad, TreeNode::Uninitialised);
        tree.insert((0, 3.0, 0.5, 0.5));
        tree.insert((1, 6.4, -0.5, 0.5));
        tree.insert((2, 7.0, -0.5, -0.5));
        tree.insert((3, 9.1, 0.5, -0.5));
        tree.insert((4, 1.6, 0.5, 1.5));
        tree.insert((5, 4.4, 0.5, 0.9));
        tree.insert((6, 18.0, 1.5, 1.5));

        print!("{}", tree);
    }

    #[test]
    fn update_forces() {
        let quad = Quadrant::new(0., 0., 4.);
        let mut tree = BHTree::new(quad, TreeNode::Uninitialised);
        tree.insert((0, 0.1, 0.5, 0.5));
        tree.insert((1, 0.1, -0.5, 0.5));
        tree.insert((2, 0.1, -0.5, -0.5));
        tree.insert((3, 0.1, 0.5, -0.5));
        tree.insert((4, 0.1, 0.5, 1.5));
        tree.insert((5, 0.1, 0.5, 0.9));
        tree.insert((6, 0.1, 1.5, 1.5));

        let mut bodies = vec![Body::new(0, 0.1, 0.5, 0.5, 0., 0.),
            Body::new(1, 0.1, -0.5, 0.5, 0., 0.),
            Body::new(2, 0.1, -0.5, -0.5, 0., 0.),
            Body::new(3, 0.1, 0.5, -0.5, 0., 0.),
            Body::new(4, 0.1, 0.5, 1.5, 0., 0.),
            Body::new(5, 0.1, 0.5, 0.9, 0., 0.),
            Body::new(6, 0.1, 1.5, 1.5, 0., 0.)
        ];

        
        tree.traverse_update_force(&tree, &mut bodies);
        
        for body in &bodies {
            println!("{:?}", body);
        }
    }

}