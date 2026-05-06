// THREADSAFE MATRIX; represented as a quad tree
#![allow(non_snake_case)]
#[cfg(not(feature = "test_direct_rec"))]
use velvet::prelude::*;
use std::sync::{Arc, Mutex};
use super::Real;

pub(crate) enum Matrix {
    // internal nodes must hold Arcs bc they are recorded in VelvetFrames
    Internal {
        _00: Arc<Matrix>,
        _01: Arc<Matrix>,
        _10: Arc<Matrix>,
        _11: Arc<Matrix>,
    },
    // leaves are only operated on locally, so they can be Boxes
    Leaf {
        data: Box<Vec<Real>>,
    },
    // mutable leaves are needed for result matrices
    MutableLeaf {
        data: Box<Mutex<Vec<Real>>>,
        dim: usize,
    }
}

// matrix creation
impl Matrix {
    // creates a matrix of size dim * 2^depth
    // (splits matrix into quadrants depth-many times before creating the dim*dim vec)
    pub(crate) fn new(depth: usize, dim: usize, value: Real, result: bool) -> Self {
        if depth <= 0 {
            if result {
                Self::make_mutable_leaf(dim, value)
            } else {
                Self::make_leaf(dim, value)
            }
        } else {
            Matrix::Internal{
                _00: Arc::new(Matrix::new(depth-1, dim, value, result)),
                _01: Arc::new(Matrix::new(depth-1, dim, value, result)),
                _10: Arc::new(Matrix::new(depth-1, dim, value, result)),
                _11: Arc::new(Matrix::new(depth-1, dim, value, result)),
            }
        }
    }

    fn make_leaf(dim: usize, value: Real) -> Self {
        let data: Vec<Real> = vec![value; dim * dim];
        Matrix::Leaf{ data: Box::from(data) }
    }

    fn make_mutable_leaf(dim: usize, value: Real) -> Self {
        let data: Vec<Real> = vec![value; dim * dim];
        Matrix::MutableLeaf{ data: Box::from(Mutex::new(data)), dim: dim }
    }

    pub(crate) fn _check (&self, result: Real) -> bool {
        let mut ok = true;
        match self {
            Matrix::Internal{ _00, _01, _10, _11 } => {
                ok &= _00._check(result);
                ok &= _01._check(result);
                ok &= _10._check(result);
                ok &= _11._check(result);
            },
            Matrix::Leaf{ data } => {
                for i in 0..data.len() {
                    if data[i] != result {
                        eprintln!("ERROR in matrix!, i = {}, value = {}", i, data[i]);
                        ok = false;
                    }
                }
            },
            Matrix::MutableLeaf{ dim:_, data:locked_data } => {
                let data = locked_data.lock().unwrap();
                for i in 0..data.len(){
                    if data[i] != result {
                        eprintln!("ERROR in matrix!, i = {}, value = {}", i, data[i]);
                        ok = false;
                    }
                }
            }
        }
        return ok;
    }
}

// matrix multiplicaiton
impl Matrix {
    #[cfg(not(feature = "test_direct_rec"))]
    #[spawnable]
    pub(crate) fn spawn_matmul(self: Arc<Self>, task: usize, a: Arc<Matrix>, b: Arc<Matrix>) {
        // threshold
        if task == 0 {
            self.multiply_stride2(&a, &b);
        } else {
            match(&*a, &*b) {
                (Matrix::Internal{_00: a00, _01: a01, _10: a10, _11: a11}, 
                Matrix::Internal{_00: b00, _01: b01, _10: b10, _11: b11}) => {
                    match &*self {
                        Matrix::Internal{_00: c00, _01: c01, _10: c10, _11: c11} => {
                            c00.clone().spawn_matmul(task-1, a00.clone(), b00.clone());
                            c01.clone().spawn_matmul(task-1, a00.clone(), b01.clone());

                            c10.clone().spawn_matmul(task-1, a10.clone(), b00.clone());
                            c11.clone().spawn_matmul(task-1, a10.clone(), b01.clone());

                            c00.clone().spawn_matmul(task-1, a01.clone(), b10.clone());
                            c01.clone().spawn_matmul(task-1, a01.clone(), b11.clone());

                            c10.clone().spawn_matmul(task-1, a11.clone(), b10.clone());
                            c11.clone().spawn_matmul(task-1, a11.clone(), b11.clone());
                        },
                        _ => panic!("C-matrix is a leaf when it shouldn't be"),
                    }
                },
                _ => panic!("multiplying on leaf nodes!")
            }
        }
    }
    // FOR CHECKING EFFECT OF DIRECT RECURSION
    #[cfg(feature = "test_direct_rec")]
    pub(crate) fn spawn_matmul(&self, __worker__: &mut velvet::VelvetWorker<crate::__Frame__>, task: usize, a: &Matrix, b: &Matrix) {
        if task == 0 {
            self.multiply_stride2(a, b);
        } else {
            match (a, b) {
                (Matrix::Internal { _00: a00, _01: a01, _10: a10, _11: a11 }, Matrix::Internal { _00: b00, _01: b01, _10: b10, _11: b11 }) => {
                    match self {
                        Matrix::Internal {_00: c00, _01: c01, _10: c10, _11: c11} => {
                            let __0__ = __worker__.get_seq();
                            __worker__.spawn(
                                crate::__Frame__::InputSpawnMatmul(__0__, c00.clone(), task - 1, a00.clone(), b00.clone())
                            );
                            let __1__ = __worker__.get_seq();
                            __worker__.spawn(
                                crate::__Frame__::InputSpawnMatmul(__1__, c01.clone(),task - 1, a00.clone(), b01.clone())
                            );
                            let __2__ = __worker__.get_seq();
                            __worker__.spawn(
                                crate::__Frame__::InputSpawnMatmul(__2__, c10.clone(), task - 1, a10.clone(), b00.clone())
                            );
                            let __3__ = __worker__.get_seq();
                            __worker__.spawn(
                                crate::__Frame__::InputSpawnMatmul(__3__, c11.clone(), task - 1, a10.clone(), b01.clone())
                            );
                            let __4__ = __worker__.get_seq();
                            __worker__.spawn(
                                crate::__Frame__::InputSpawnMatmul(__4__, c00.clone(), task - 1, a01.clone(), b10.clone())
                            );
                            let __5__ = __worker__.get_seq();
                            __worker__.spawn(
                                crate::__Frame__::InputSpawnMatmul(__5__, c01.clone(), task - 1, a01.clone(), b11.clone())
                            );
                            let __6__ = __worker__.get_seq();
                            __worker__.spawn(
                                crate::__Frame__::InputSpawnMatmul(__6__, c10.clone(), task - 1, a11.clone(), b10.clone())
                            );
                            let __7__ = __worker__.get_seq();
                            __worker__.spawn(
                                crate::__Frame__::InputSpawnMatmul(__7__, c11.clone(), task - 1, a11.clone(), b11.clone())
                            );

                            let __SYNC__ = __worker__.sync(__7__);
                            let __SYNC_RES__ = match __SYNC__ {
                                crate::__Frame__::InputSpawnMatmul(_, a0, a1, a2, a3) => {
                                    a0.spawn_matmul(__worker__, a1, &a2, &a3)
                                }
                                crate::__Frame__::Stolen(ptr) => {
                                    let mut try_lock = ptr.try_lock();
                                    loop {
                                        if let Ok(_value) = try_lock {
                                            break;
                                        } else {
                                            __worker__.steal();
                                            try_lock = ptr.try_lock();
                                        }
                                    }
                                }
                            };
                            let __SYNC__ = __worker__.sync(__6__);
                            let __SYNC_RES__ = match __SYNC__ {
                                crate::__Frame__::InputSpawnMatmul(_, a0, a1, a2, a3) => {
                                    a0.spawn_matmul(__worker__, a1, &a2, &a3)
                                }
                                crate::__Frame__::Stolen(ptr) => {
                                    let mut try_lock = ptr.try_lock();
                                    loop {
                                        if let Ok(_value) = try_lock {
                                            break;
                                        } else {
                                            __worker__.steal();
                                            try_lock = ptr.try_lock();
                                        }
                                    }
                                }
                            };
                            let __SYNC__ = __worker__.sync(__5__);
                            let __SYNC_RES__ = match __SYNC__ {
                                crate::__Frame__::InputSpawnMatmul(_, a0, a1, a2, a3) => {
                                    a0.spawn_matmul(__worker__, a1, &a2, &a3)
                                }
                                crate::__Frame__::Stolen(ptr) => {
                                    let mut try_lock = ptr.try_lock();
                                    loop {
                                        if let Ok(_value) = try_lock {
                                            break;
                                        } else {
                                            __worker__.steal();
                                            try_lock = ptr.try_lock();
                                        }
                                    }
                                }
                            };
                            let __SYNC__ = __worker__.sync(__4__);
                            let __SYNC_RES__ = match __SYNC__ {
                                crate::__Frame__::InputSpawnMatmul(_, a0, a1, a2, a3) => {
                                    a0.spawn_matmul(__worker__, a1, &a2, &a3)
                                }
                                crate::__Frame__::Stolen(ptr) => {
                                    let mut try_lock = ptr.try_lock();
                                    loop {
                                        if let Ok(_value) = try_lock {
                                            break;
                                        } else {
                                            __worker__.steal();
                                            try_lock = ptr.try_lock();
                                        }
                                    }
                                }
                            };
                            let __SYNC__ = __worker__.sync(__3__);
                            let __SYNC_RES__ = match __SYNC__ {
                                crate::__Frame__::InputSpawnMatmul(_, a0, a1, a2, a3) => {
                                    a0.spawn_matmul(__worker__, a1, &a2, &a3)
                                }
                                crate::__Frame__::Stolen(ptr) => {
                                    let mut try_lock = ptr.try_lock();
                                    loop {
                                        if let Ok(_value) = try_lock {
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
                                crate::__Frame__::InputSpawnMatmul(_, a0, a1, a2, a3) => {
                                    a0.spawn_matmul(__worker__, a1, &a2, &a3)
                                }
                                crate::__Frame__::Stolen(ptr) => {
                                    let mut try_lock = ptr.try_lock();
                                    loop {
                                        if let Ok(_value) = try_lock {
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
                                crate::__Frame__::InputSpawnMatmul(_, a0, a1, a2, a3) => {
                                    a0.spawn_matmul(__worker__, a1, &a2, &a3)
                                }
                                crate::__Frame__::Stolen(ptr) => {
                                    let mut try_lock = ptr.try_lock();
                                    loop {
                                        if let Ok(_value) = try_lock {
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
                                crate::__Frame__::InputSpawnMatmul(_, a0, a1, a2, a3) => {
                                    a0.spawn_matmul(__worker__, a1, &a2, &a3)
                                }
                                crate::__Frame__::Stolen(ptr) => {
                                    let mut try_lock = ptr.try_lock();
                                    loop {
                                        if let Ok(_value) = try_lock {
                                            break;
                                        } else {
                                            __worker__.steal();
                                            try_lock = ptr.try_lock();
                                        }
                                    }
                                }
                            };
                        }
                        _ => panic!("C-matrix is a leaf when it shouldn\'t be"),
                    }
                }
                _ => panic!("multiplying on leaf nodes!"),
            }
        }
    }

    pub(crate) fn matmul_par(&self, task: usize, a: &Matrix, b: &Matrix) {
        // threshold
        if task == 0 {
            self.multiply_stride2(a, b);
        } else {
            match(a, b) {
                (Matrix::Internal{_00: a00, _01: a01, _10: a10, _11: a11}, 
                Matrix::Internal{_00: b00, _01: b01, _10: b10, _11: b11}) => {
                    match self {
                        Matrix::Internal{_00: c00, _01: c01, _10: c10, _11: c11} => {
                            c00.matmul_par(task-1, a00, b00);
                            c01.matmul_par(task-1, a00, b01);

                            c10.matmul_par(task-1, a10, b00);
                            c11.matmul_par(task-1, a10, b01);

                            c00.matmul_par(task-1, a01, b10);
                            c01.matmul_par(task-1, a01, b11);

                            c10.matmul_par(task-1, a11, b10);
                            c11.matmul_par(task-1, a11, b11);
                        },
                        _ => panic!("C-matrix is a leaf when it shouldn't be"),
                    }
                },
                _ => panic!("multiplying on leaf nodes!")
            }
        }
    }

    #[cfg(feature = "rayon")]
    pub(crate) fn rayon_matmul(&self, task: usize, a: &Matrix, b: &Matrix) {
        // threshold
        if task == 0 {
            self.multiply_stride2(a, b);
        } else {
            match(a, b) {
                (Matrix::Internal{_00: a00, _01: a01, _10: a10, _11: a11}, 
                Matrix::Internal{_00: b00, _01: b01, _10: b10, _11: b11}) => {
                    match self {
                        Matrix::Internal{_00: c00, _01: c01, _10: c10, _11: c11} => {
                            join8(
                                || c00.rayon_matmul(task-1, a00, b00),
                                || c01.rayon_matmul(task-1, a00, b01),

                                || c10.rayon_matmul(task-1, a10, b00),
                                || c11.rayon_matmul(task-1, a10, b01),

                                || c00.rayon_matmul(task-1, a01, b10),
                                || c01.rayon_matmul(task-1, a01, b11),

                                || c10.rayon_matmul(task-1, a11, b10),
                                || c11.rayon_matmul(task-1, a11, b11),
                            );
                        },
                        _ => panic!("C-matrix is a leaf when it shouldn't be"),
                    }
                },
                _ => panic!("multiplying on leaf nodes!")
            }
        }
    }

    fn multiply_stride2(&self, a: &Matrix, b: &Matrix) {
        match (self, a, b) {
            (Matrix::MutableLeaf{ data:c_locked, dim }, Matrix::Leaf{ data:a }, Matrix::Leaf{ data:b }) => {
                let n = *dim;
                // Get raw pointers to bypass slice metadata and bounds checks
                let a_ptr = a.as_ptr();
                let b_ptr = b.as_ptr();

                let mut a0;
                let mut a1;

                for i in (0..n).step_by(2) {
                    unsafe {
                        a0 = a_ptr.add(i * n);
                        a1 = a_ptr.add((i + 1) * n);
                    }
                    for j in (0..n).step_by(2) {
                        let mut s00 = 0.0;
                        let mut s01 = 0.0;
                        let mut s10 = 0.0;
                        let mut s11 = 0.0;

                        for k in (0..n).step_by(2) {
                            unsafe {
                                let b0 = b_ptr.add(k * n);
                                let b1 = b_ptr.add((k + 1) * n);

                                s00 += *a0.add(k) * *b0.add(j) + *a0.add(k + 1) * *b1.add(j);
                                s10 += *a1.add(k) * *b0.add(j) + *a1.add(k + 1) * *b1.add(j);
                                s01 += *a0.add(k) * *b0.add(j + 1) + *a0.add(k + 1) * *b1.add(j + 1);
                                s11 += *a1.add(k) * *b0.add(j + 1) + *a1.add(k + 1) * *b1.add(j + 1);
                            }
                        }

                        {
                            let mut c = c_locked.lock().unwrap();
                            let c_ptr = c.as_mut_ptr();

                            unsafe {
                                let c0 = c_ptr.add(i * n);
                                let c1 = c_ptr.add((i + 1) * n);

                                *c0.add(j) += s00;
                                *c0.add(j + 1) += s01;
                                *c1.add(j) += s10;
                                *c1.add(j + 1) += s11;
                            }
                        }
                    }
                }
            }
            _ => panic!("multiply_stride not called on Leaves! "),
        }
    }
}

#[cfg(feature = "rayon")]
#[allow(clippy::too_many_arguments)]
fn join8<F1, F2, F3, F4, F5, F6, F7, F8, R1, R2, R3, R4, R5, R6, R7, R8>(
    f1: F1,
    f2: F2,
    f3: F3,
    f4: F4,
    f5: F5,
    f6: F6,
    f7: F7,
    f8: F8,
) -> (R1, R2, R3, R4, R5, R6, R7, R8)
where
    F1: FnOnce() -> R1 + Send,
    R1: Send,
    F2: FnOnce() -> R2 + Send,
    R2: Send,
    F3: FnOnce() -> R3 + Send,
    R3: Send,
    F4: FnOnce() -> R4 + Send,
    R4: Send,
    F5: FnOnce() -> R5 + Send,
    R5: Send,
    F6: FnOnce() -> R6 + Send,
    R6: Send,
    F7: FnOnce() -> R7 + Send,
    R7: Send,
    F8: FnOnce() -> R8 + Send,
    R8: Send,
{
    let (((r1, r2), (r3, r4)), ((r5, r6), (r7, r8))) = rayon::join(
        || rayon::join(|| rayon::join(f1, f2), || rayon::join(f3, f4)),
        || rayon::join(|| rayon::join(f5, f6), || rayon::join(f7, f8)),
    );
    (r1, r2, r3, r4, r5, r6, r7, r8)
}