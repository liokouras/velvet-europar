mod matrix_par;
use matrix_par::Matrix;
mod matrix_seq;
use std::{sync::Arc, env, time::Instant};

use velvet::prelude::*;
#[cfg(not(feature = "test_direct_rec"))]
include!(concat!(env!("OUT_DIR"), "/velvet_app.rs"));

// pub(crate) type Real = f32;
pub(crate) type Real = f64;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        println!("usage for matmul: cargo run [cargo_options] [velvet|seq|rayon] [depth] [dim]");
        println!("example: cargo run --release velvet 2 4");
        return;
    }
    let app = &args[1];
    let depth: usize = args[2].parse().unwrap();
    let dim: usize = args[3].parse().unwrap();

    if app.eq("seq") {
        seq_main(depth, dim);
        return;
    }

    let matrix_a = Arc::new(Matrix::new(depth, dim, 1.0, false));
    let matrix_b= Arc::new(Matrix::new(depth, dim, 2.0, false));
    let matrix_c = Arc::new(Matrix::new(depth, dim, 0.0, true));

    if app.eq("velvet") {
        #[cfg(not(feature = "test_direct_rec"))]
        velvet_main(depth, dim, matrix_a, matrix_b, matrix_c);
    } else if app.eq("rayon") {
        #[cfg(feature = "rayon")]
        {
            if args.len() < 5 {
                println!("must provide number of threads for Rayon");
                println!("usage for matmul: cargo run [cargo_options] [velvet|seq|rayon] [depth] [dim] [numer_of_threads]");
                println!("example: cargo run --release rayon 2 4 8");
                return;
            }
            let num_threads: usize = args[4].parse().unwrap();
            rayon_main(depth, dim, matrix_a, matrix_b, matrix_c, num_threads);
        }
        #[cfg(not(feature = "rayon"))]
        println!("COMPILE WITH RAYON!");
    } else if app.eq("par") {
        par_main(depth, dim, matrix_a, matrix_b, matrix_c);
    } else if app.eq("test_direct") {
        #[cfg(feature = "test_direct_rec")]
        {
            if args.len() < 5 {
                println!("must provide number of workers!");
                return;
            }
            let num_workers: usize = args[4].parse().unwrap();
            test_direct_recursion(depth, dim, matrix_a, matrix_b, matrix_c, num_workers);
        }
        #[cfg(not(feature = "test_direct_rec"))]
        println!("Compile with feature \"test_direct_rec\"");
    } else {
        println!("Unrecognised app: {}", app);
    }
}

fn seq_main(depth: usize, dim: usize) {
    // pin thread!
    let core_ids = core_affinity::get_core_ids().unwrap();
    let res = core_affinity::set_for_current(core_ids[0]);
    if !res {
        eprintln!("Could not pin Root thread id continuing without pinning...");
    }

    let matrix_a = matrix_seq::Matrix::new(depth, dim, 1.0);
    let matrix_b = matrix_seq::Matrix::new(depth, dim, 2.0);
    let mut matrix_c = matrix_seq::Matrix::new(depth, dim, 0.0);

    let start = Instant::now();
    matrix_c.matmul(depth, &matrix_a, &matrix_b);
    let end = start.elapsed();

    // let full_dim: usize = 2_usize.pow(depth.try_into().unwrap()) * dim;
    // let exp = (full_dim * 2) as Real;
    // let ok = matrix_c._check(exp);
    // eprintln!("checked. Ok = {}", ok);

    println!("-3,1,{},{},{}", depth, dim, end.as_secs_f32());
}

#[cfg(not(feature = "test_direct_rec"))]
#[velvet_main(spawn_matmul)]
fn velvet_main(depth: usize, dim: usize, matrix_a: Arc<Matrix>, matrix_b: Arc<Matrix>, matrix_c: Arc<Matrix>) {
    let start = Instant::now();
    matrix_c.spawn_matmul(depth, matrix_a, matrix_b);
    let end = start.elapsed();

    let version = match velvet_get_queue_name().as_str() {
        "safe" => 3,
        "unsafe" => 22,
        "crossbeam" => 24,
        _ => -1,
   };

    println!("{},{},{},{},{}", version, velvet_get_num_workers(), depth, dim, end.as_secs_f32());

    // let full_dim: usize = 2_usize.pow(depth.try_into().unwrap()) * dim;
    // let exp = (full_dim * 2) as Real;
    // let ok = matrix_c._check(exp);
    // eprintln!("checked. exp = {}. Ok = {}", exp, ok);
        
}

fn par_main(depth: usize, dim: usize, matrix_a: Arc<Matrix>, matrix_b: Arc<Matrix>, matrix_c: Arc<Matrix>) {
    // pin thread!
    let core_ids = core_affinity::get_core_ids().unwrap();
    let res = core_affinity::set_for_current(core_ids[0]);
    if !res {
        eprintln!("Could not pin Root thread id continuing without pinning...");
    }

    let start = Instant::now();
    matrix_c.matmul_par(depth, &matrix_a, &matrix_b);
    let end = start.elapsed();

    println!("-2,1,{},{},{}", depth, dim, end.as_secs_f32());

    // let full_dim: usize = 2_usize.pow(depth.try_into().unwrap()) * dim;
    // let exp = (full_dim * 2) as Real;
    // let ok = matrix_c._check(exp);
    // eprintln!("checked. Ok = {}", ok);
}

#[cfg(feature = "rayon")]
fn rayon_main(depth: usize, dim: usize,matrix_a: Arc<Matrix>, matrix_b: Arc<Matrix>, matrix_c: Arc<Matrix>, num_threads: usize) {
    let cores = core_affinity::get_core_ids().unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .start_handler(move |index| {
            let core_id = cores[index % cores.len()];
            let res = core_affinity::set_for_current(core_id);
            if !res {
                eprintln!("Could not pin worker thread id {:?}, continuing without pinning...", core_id.id);
            }
        })
        .build_global()
        .unwrap();

    let start = Instant::now();
    matrix_c.rayon_matmul(depth, &matrix_a, &matrix_b);
    let elapsed = start.elapsed();

    // let full_dim: usize = 2_usize.pow(depth.try_into().unwrap()) * dim;
    // let res = (full_dim * 2) as Real;
    // let ok = matrix_c._check(res);
    // eprintln!("checked. Ok = {}", ok);

    println!("4,{},{},{},{}", num_threads, depth, dim, elapsed.as_secs_f32());
}
// ---------------------- FOR CHECKING EFFECT OF DIRECT RECURSION ----------
#[cfg(feature = "test_direct_rec")]
#[allow(non_snake_case)]
fn test_direct_recursion(depth: usize, dim: usize, matrix_a: Arc<Matrix>, matrix_b: Arc<Matrix>, matrix_c: Arc<Matrix>, num_workers: usize) {
    let mut __root__worker__ = velvet::VelvetWorker::prepare_workers(num_workers, 64, crate::__velvet_steal__);
    __root__worker__.wait();

    let _full_dim: usize = 2_usize.pow(depth.try_into().unwrap()) * dim;

    let start = Instant::now();
    matrix_c.spawn_matmul(&mut __root__worker__, depth, &matrix_a, &matrix_b);
    let end = start.elapsed();

    let version = match velvet_get_queue_name().as_str() {
        "safe" => 8,
        "unsafe" => 18,
        "crossbeam" => 19,
        _ => -1,
    };

    println!("{},{},{},{},{}", version, num_workers, depth, dim, end.as_secs_f32());
}

#[cfg(feature = "test_direct_rec")]
pub(crate) enum __Frame__ {
    Stolen(std::sync::Arc<std::sync::Mutex<Option<__Frame__>>>),
    InputSpawnMatmul(
        usize,
        std::sync::Arc<crate::matrix_par::Matrix>,
        usize,
        std::sync::Arc<crate::matrix_par::Matrix>,
        std::sync::Arc<crate::matrix_par::Matrix>,
    ),
}
#[cfg(feature = "test_direct_rec")]
impl velvet::Identifiable for __Frame__ {
    fn get_id(&self) -> usize {
        if let __Frame__::InputSpawnMatmul(uid, ..) = self {
            return *uid;
        }
        return 0;
    }
}
#[cfg(feature = "test_direct_rec")]
fn __velvet_steal__(worker: &mut velvet::VelvetWorker<__Frame__>) {
    let stealers = &worker.stealers;
    let len = stealers.len();
    let mut n = worker.get_random(len);
    let result_slot = std::sync::Arc::new(std::sync::Mutex::new(None));
    let mut lock = result_slot.lock().unwrap();
    for _ in 0..len {
        let maybe_frame = stealers[n].steal(__Frame__::Stolen(result_slot.clone()));
        if let Some(frame) = maybe_frame {
            match frame {
                __Frame__::InputSpawnMatmul(_, a0, a1, a2, a3) => {
                    a0.spawn_matmul(worker, a1, &a2, &a3);
                    *lock = None;
                }
                _ => panic!("WRONG STOLEN WORK FRAME!"),
            }
            return;
        }
        n = (n + 1) % len;
    }
}