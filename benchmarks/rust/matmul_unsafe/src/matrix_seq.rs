use super::Real;

pub(crate) enum Matrix {
    Internal {
        _00: Box<Matrix>,
        _01: Box<Matrix>,
        _10: Box<Matrix>,
        _11: Box<Matrix>,
    },
    Leaf {
        data: Vec<Real>,
        dim: usize,
    }
}

// matrix creation
impl Matrix {
    pub(crate) fn new(depth: usize, dim: usize, value: Real) -> Self {
        if depth <= 0 {
            Self::make_leaf(dim, value)
        } else {
            Matrix::Internal{
                _00: Box::new(Matrix::new(depth-1, dim, value)),
                _01: Box::new(Matrix::new(depth-1, dim, value)),
                _10: Box::new(Matrix::new(depth-1, dim, value)),
                _11: Box::new(Matrix::new(depth-1, dim, value)),
            }
        }
    }

    fn make_leaf(dim: usize, value: Real) -> Self {
        let data: Vec<Real> = vec![value; dim * dim];
        Matrix::Leaf{ data, dim }
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
            Matrix::Leaf{ dim, data } => {
                for i in 0..dim*dim {
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

// matrix multiplication
impl Matrix {
    pub(crate) fn matmul(&mut self, task: usize, a: &Matrix, b: &Matrix) {
        // threshold
        if task == 0 {
            self.multiply_stride2(a, b);
        } else {
            match(a, b) {
                (Matrix::Internal{_00: a00, _01: a01, _10: a10, _11: a11}, 
                Matrix::Internal{_00: b00, _01: b01, _10: b10, _11: b11}) => {
                    match self {
                        Matrix::Internal{_00: c00, _01: c01, _10: c10, _11: c11} => {
                            c00.matmul(task-1, a00, b00);
                            c01.matmul(task-1, a00, b01);

                            c10.matmul(task-1, a10, b00);
                            c11.matmul(task-1, a10, b01);

                            c00.matmul(task-1, a01, b10);
                            c01.matmul(task-1, a01, b11);

                            c10.matmul(task-1, a11, b10);
                            c11.matmul(task-1, a11, b11);
                        },
                        _ => panic!("C-matrix is a leaf when it shouldn't be"),
                    }
                },
                _ => panic!("multiplying on leaf nodes!")
            }
        }
    }

    fn multiply_stride2(&mut self, a: &Matrix, b: &Matrix) {
        match (self, a, b) {
            (Matrix::Leaf{ data:c, dim }, Matrix::Leaf{ data:a, dim:_ }, Matrix::Leaf{ data:b, dim:_ }) => {
                let n = *dim;
                // Get raw pointers to bypass slice metadata and bounds checks
                let a_ptr = a.as_ptr();
                let b_ptr = b.as_ptr();
                let c_ptr = c.as_mut_ptr();

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
            _ => panic!("multiply_stride not called on Leaves! "),
        }
    }
}