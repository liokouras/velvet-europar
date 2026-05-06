#![allow(non_snake_case)]
#[cfg(not(feature = "test_direct_rec"))]
use velvet::prelude::*;
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use std::{mem, fmt, sync::{Arc, OnceLock, RwLock}};
use super::{bh_par::THRESHOLD, THETA, par_body::Body, G, LEAF_CAP, quad::Quadrant};

type TreeBody = (usize, f64, f64, f64); // Body ID, mass, px, py

pub(super) static BODIES:OnceLock<Vec<RwLock<Body>>> = OnceLock::new();

#[derive(Clone, Debug)]
pub(super) enum TreeNode {
    Leaf(Vec<TreeBody>), // if vec is empty: uninit
    Aggregate((f64, f64, f64, usize, Vec<Arc<BHTree>>)), // mass, px, py, num. bodies, children [NW, NE, SW, SE]
}

#[derive(Clone, Debug)]
pub(super) struct BHTree {
    pub(super) body: TreeNode, // the body at the root of this (sub)tree; Leaf (Body ID), Aggregate (mass-px-py), or uninitialised
    pub(super) quad: Quadrant, // quadrant that this Tree represents
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
    fn split_leaf(&mut self) -> Vec<Arc<BHTree>> {
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

        vec![Arc::new(nw), Arc::new(ne), Arc::new(sw), Arc::new(se)]
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
                    let _ = mem::replace(&mut self.body, TreeNode::Aggregate((m, x, y, LEAF_CAP + 1, children)));
                    // recursively insert the body into the appropriate quadrant
                    self.put_body(body_info);
                }
            },
            TreeNode::Aggregate((ref mut mass, ref mut px, ref mut py, ref mut childcount, _)) => {
                // if this node is already an aggregate, update the center-of-mass and total mass
                let m = *mass + body_info.1;
                let x = ((*px * *mass) + (body_info.2 * body_info.1)) / m;
                let y = ((*py * *mass) + (body_info.3 * body_info.1)) / m;
                *mass = m;
                *px = x;
                *py = y;

                *childcount += 1;
            
                // recursively insert Body b into the appropriate quadrant
                self.put_body(body_info);
            },
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

    // insert a body into the appropriate quadrant
    fn put_body(&mut self, body_info: TreeBody) {
        if let TreeNode::Aggregate((_, _, _, _, ref mut children)) = self.body {
            for child in children {
                if child.quad.contains(body_info.2, body_info.3) {
                    Arc::make_mut(child).insert(body_info);
                    return;
                }
            }
        }
    }
}

// --------- SEPARATE FORCE COMPUTATION + UPDATE ---------
impl BHTree {
    pub(super) fn compute_force_seq(&self, body: &TreeBody) -> (f64, f64) {
        let eps: f64 = 30000.;
        match self.body {
            TreeNode::Leaf(ref bodies) => {
                let (mut fx, mut fy) = (0., 0.);
                for leaf_body in bodies {
                    if leaf_body.0 != body.0 {
                        // if the current node is a different leaf, compute the net force acting on b
                        let dx = leaf_body.2 - body.2;
                        let dy = leaf_body.3 - body.3;
                        let sq = dx*dx + dy*dy;
                        let dist = sq.sqrt();
                        let f = (G * body.1 * leaf_body.1) / (dist*dist + eps*eps);
                        fx += f * dx / dist;
                        fy += f * dy / dist;
                    }
                }
                return (fx, fy);
            },
            TreeNode::Aggregate((mass, px, py, _, ref children)) => {
                // compute Euclidean distance between body and center of aggregate
                let s = self.quad.length();
                let dx = px - body.2;
                let dy = py - body.3;
                let sq = dx*dx + dy*dy;
                let dist = sq.sqrt();
                let ratio = s / dist;

                // compare ratio to threshold value Theta
                if ratio < THETA {
                    // b is far away, use aggregate
                    let f = (G * body.1 * mass) / (dist*dist + eps*eps);
                    let fx = f * dx / dist;
                    let fy = f * dy / dist;
                    return (fx, fy);
                } else {
                    // b is close, recurse
                    let (x0, y0) = children[0].compute_force_seq(body);
                    let (x1, y1) = children[1].compute_force_seq(body);
                    let (x2, y2) = children[2].compute_force_seq(body);
                    let (x3, y3) = children[3].compute_force_seq(body);
                    let fx = x0 + x1 + x2 + x3;
                    let fy = y0 + y1 + y2 + y3;
                    return (fx, fy);
                }
            },
        }
    }

    // traverses the tree and calls compute_force on each leaf
    pub(super) fn traverse_update_force_seq(&self, root: &BHTree) {
        match self.body {
            TreeNode::Leaf(ref leaf_bodies) => {
                let bodies = BODIES.get().unwrap();
                for body in leaf_bodies {
                    let (fx, fy) = root.compute_force_seq(&body);
                    bodies[body.0].write().unwrap().set_force(fx, fy);
                }
            },
            TreeNode::Aggregate((_, _, _, _, ref children)) => {
                for child in children {
                    child.traverse_update_force_seq(root);
                }
            },
        }
    }

    pub(super) fn traverse_update_force(&self, root: Arc<BHTree>) {
        match self.body {
            TreeNode::Leaf(ref leaf_bodies) => {
                let bodies = BODIES.get().unwrap();
                for body in leaf_bodies {
                    let (fx, fy) = root.compute_force_seq(&body);
                    bodies[body.0].write().unwrap().set_force(fx, fy);
                }
            },
            TreeNode::Aggregate((_, _, _, childcount, ref children)) => {
                if childcount < THRESHOLD {
                    self.traverse_update_force_seq(&root);
                    return;
                }
                children[0].traverse_update_force(root.clone());
                children[1].traverse_update_force(root.clone());
                children[2].traverse_update_force(root.clone());
                children[3].traverse_update_force(root.clone());
            },
        }
    }

    #[cfg(not(feature = "test_direct_rec"))]
    #[spawnable]
    pub(super) fn traverse_spawn(self: Arc<Self>, root: Arc<BHTree>) {
        match self.body {
            TreeNode::Leaf(ref leaf_bodies) => {
                let bodies = BODIES.get().unwrap();
                for body in leaf_bodies {
                    let (fx, fy) = root.compute_force_seq(&body);
                    bodies[body.0].write().unwrap().set_force(fx, fy);
                }
            },
            TreeNode::Aggregate((_, _, _, childcount, ref children)) => {
                if childcount < THRESHOLD {
                    self.traverse_update_force_seq(&root);
                    return;
                }
                children[0].clone().traverse_spawn(root.clone());
                children[1].clone().traverse_spawn(root.clone());
                children[2].clone().traverse_spawn(root.clone());
                children[3].clone().traverse_spawn(root.clone());
            },
        }
    }

    // FOR CHECKING EFFECT OF DIRECT RECURSION
    #[cfg(feature = "test_direct_rec")]
    pub(super) fn traverse_spawn(
        self: Arc<Self>,
        __worker__: &mut velvet::VelvetWorker<crate::__Frame__>,
        root: Arc<BHTree>,
    ) {
        match self.body {
            TreeNode::Leaf(ref leaf_bodies) => {
                let bodies = BODIES.get().unwrap();
                for body in leaf_bodies {
                    let (fx, fy) = root.compute_force_seq(&body);
                    bodies[body.0].write().unwrap().set_force(fx, fy);
                }
            }
            TreeNode::Aggregate((_, _, _, childcount, ref children)) => {
                if childcount < THRESHOLD {
                    self.traverse_update_force_seq(&root);
                    return;
                }
                let __0__ = __worker__.get_seq();
                __worker__
                    .spawn(
                        crate::__Frame__::InputTraverseSpawn(
                            __0__,
                            children[0].clone(),
                            root.clone(),
                        ),
                    );
                let __1__ = __worker__.get_seq();
                __worker__
                    .spawn(
                        crate::__Frame__::InputTraverseSpawn(
                            __1__,
                            children[1].clone(),
                            root.clone(),
                        ),
                    );
                let __2__ = __worker__.get_seq();
                __worker__
                    .spawn(
                        crate::__Frame__::InputTraverseSpawn(
                            __2__,
                            children[2].clone(),
                            root.clone(),
                        ),
                    );
                let __3__ = __worker__.get_seq();
                __worker__
                    .spawn(
                        crate::__Frame__::InputTraverseSpawn(
                            __3__,
                            children[3].clone(),
                            root.clone(),
                        ),
                    );

                let __SYNC__ = __worker__.sync(__3__);
                let __SYNC_RES__ = match __SYNC__ {
                    crate::__Frame__::InputTraverseSpawn(_, a0, a1) => {
                        a0.traverse_spawn(__worker__, a1)
                    }
                    crate::__Frame__::Stolen(ptr) => {
                        let mut try_lock = ptr.try_lock();
                        loop {
                            if let Ok(mut _value) = try_lock {
                                break;
                            } else {
                                __worker__.steal();
                                try_lock = ptr.try_lock();
                            }
                        }
                    }
                };
                let __SYNC__ = __worker__.sync(__2__);
                let __SYNC_RES__ = match __SYNC__ {
                    crate::__Frame__::InputTraverseSpawn(_, a0, a1) => {
                        a0.traverse_spawn(__worker__, a1)
                    }
                    crate::__Frame__::Stolen(ptr) => {
                        let mut try_lock = ptr.try_lock();
                        loop {
                            if let Ok(mut _value) = try_lock {
                                break;
                            } else {
                                __worker__.steal();
                                try_lock = ptr.try_lock();
                            }
                        }
                    }
                };
                let __SYNC__ = __worker__.sync(__1__);
                let __SYNC_RES__ = match __SYNC__ {
                    crate::__Frame__::InputTraverseSpawn(_, a0, a1) => {
                        a0.traverse_spawn(__worker__, a1)
                    }
                    crate::__Frame__::Stolen(ptr) => {
                        let mut try_lock = ptr.try_lock();
                        loop {
                            if let Ok(mut _value) = try_lock {
                                break;
                            } else {
                                __worker__.steal();
                                try_lock = ptr.try_lock();
                            }
                        }
                    }
                };
                let __SYNC__ = __worker__.sync(__0__);
                let __SYNC_RES__ = match __SYNC__ {
                    crate::__Frame__::InputTraverseSpawn(_, a0, a1) => {
                        a0.traverse_spawn(__worker__, a1)
                    }
                    crate::__Frame__::Stolen(ptr) => {
                        let mut try_lock = ptr.try_lock();
                        loop {
                            if let Ok(mut _value) = try_lock {
                                break;
                            } else {
                                __worker__.steal();
                                try_lock = ptr.try_lock();
                            }
                        }
                    }
                };
            }
        }
    }

    #[cfg(feature = "rayon")]
    pub(super) fn traverse_rayon(&self, root: &BHTree) {
        match self.body {
            TreeNode::Leaf(ref leaf_bodies) => {
                let bodies = BODIES.get().unwrap();
                leaf_bodies.par_iter().for_each(|body| {
                    let (fx, fy) = root.compute_force_seq(body);
                    bodies[body.0].write().unwrap().set_force(fx, fy);
                });
            },
            
            TreeNode::Aggregate((_, _, _, childcount, ref children)) => {
                if childcount < THRESHOLD {
                    self.traverse_update_force_seq(root);
                    return;
                }
                children.par_iter().for_each(|child| child.traverse_rayon(root));
            },
        }
    }


}

// --------- ITERATING ---------
#[cfg(feature = "rayon")]
pub(super) struct TreeIterator<'a> {
    stack: Vec<&'a BHTree>,
}
#[cfg(feature = "rayon")]
impl BHTree {
    // method to get an iterator for the tree
    pub(super) fn iter(&self) -> TreeIterator<'_> {
        let mut stack = Vec::new();
        match self.body {
            TreeNode::Leaf(_) => stack = vec![self],
            TreeNode::Aggregate((_, _, _, _, ref children)) => {
                for child in children.iter().rev() {
                    stack.push(&child);
                }
            }
        }

        TreeIterator { stack, }
    }
}
#[cfg(feature = "rayon")]
impl<'a> Iterator for TreeIterator<'a> {
    type Item = &'a BHTree;

    fn next(&mut self) -> Option<Self::Item> {
        // pop the next node from the stack
        if let Some(tree) = self.stack.pop() {
            if let TreeNode::Aggregate((_, _, _, _, ref children)) = tree.body {
                for child in children.iter().rev() {
                    self.stack.push(&child);
                }
            }
            return Some(tree);
        } else {
            None // end of traversal
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
            }
            TreeNode::Aggregate((_, px, py, _, children)) => {
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