#![allow(non_snake_case)]
use velvet::prelude::*;
use std::time::Instant;
use std::env;
#[cfg(not(feature = "test_direct_rec"))]
include!(concat!(env!("OUT_DIR"), "/velvet_app.rs"));

const THRESHOLD: u64 = parse_threshold();
const fn parse_threshold() -> u64 {
    if let Some(string) = option_env!("THRESHOLD") {
        let mut res: u64 = 0;
        let mut bytes = string.as_bytes();
        while let [byte, rest @ ..] = bytes {
            assert!(b'0' <= *byte && *byte <= b'9', "invalid digit");
            res *= 10;
            res += (*byte - b'0') as u64;
            bytes = rest;
        }
        res
    } else {
        1500
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        println!("usage for adapint: cargo run [cargo_options] [velvet|seq|rayon] [a] [b] [epsilon]");
        println!("example: cargo run --release velvet 0.0 640000 0.0001");
        return;
    }
    let app = &args[1];
    let a: f64 = args[2].parse().unwrap();
    let b: f64 = args[3].parse().unwrap();
    let epsilon: f64 = args[4].parse().unwrap();

    if app.eq("velvet") {
        #[cfg(not(feature = "test_direct_rec"))]
        velvet_main(a, b, epsilon);
    } else if app.eq("rayon") {
        #[cfg(feature = "rayon")]
        {
            if args.len() < 6 {
                println!("must provide number of threads for Rayon");
                println!("usage: cargo run [cargo_options] [velvet|seq|rayon] [start] [end] [epsilon] [number_of_threads]");
                println!("example: cargo run --release rayon 0.0 640000 0.0001 8");
                return;
            }
            let num_workers: usize = args[5].parse().unwrap();
            rayon_main(a, b, epsilon, num_workers);
        }
        #[cfg(not(feature = "rayon"))]
        println!("COMPILE WITH RAYON!");
    } else if app.eq("test_direct") {
        #[cfg(feature = "test_direct_rec")]
        {
            if args.len() < 6 {
                println!("must provide number of workers!");
                return;
            }
            let num_workers: usize = args[5].parse().unwrap();
            test_direct_recursion(a, b, epsilon, num_workers);
        }
        #[cfg(not(feature = "test_direct_rec"))]
        println!("Compile with feature \"test_direct_rec\"");
    } else {
        // pin thread!
        let core_ids = core_affinity::get_core_ids().unwrap();
        let res = core_affinity::set_for_current(core_ids[0]);
        if !res {
            eprintln!("Could not pin Root thread id continuing without pinning...");
        }

        let start_seq = Instant::now();
        let oracle = adapint_seq(a, b, epsilon);
        let end_seq = start_seq.elapsed();
        eprintln!("ORACLE = {}, in seq time = {}", oracle, end_seq.as_secs_f32());
        println!("0,1,{},{},{},0,{}", a, b, epsilon, end_seq.as_secs_f32());
    }
}

#[inline(always)]
fn f(x: f64) -> f64 {
    x.sin() * 0.1 * x
}

fn adapint_seq(a: f64, b: f64, epsilon: f64) -> f64 {
    let delta = (b - a) / 2.0;
    let deltahalf = delta / 2.0;
    let mid = delta + a;
    let fa = f(a);
    let fb = f(b);
    let fmid = f(mid);
    let total = delta * (fa + fb);
    let left = deltahalf * (fa + fmid);
    let right = deltahalf * (fb + fmid);
    let mut diff = total - (left + right);
    if diff < 0.0 {
        diff = -diff;
    }
    
    if diff < epsilon {
        return total
    } else {
        let i2 = adapint_seq(a, mid, epsilon);
        let i1 = adapint_seq(mid, b, epsilon);
        return i1 + i2; 
    }
}

#[cfg(not(feature = "test_direct_rec"))]
#[velvet_main(adapint)]
fn velvet_main(a: f64, b: f64, epsilon: f64) {
    let start = Instant::now();
    let res = adapint(a, b, epsilon);
    let end = start.elapsed();
    eprintln!("VELVET RESULT = {}, in parallel time = {}", res, end.as_secs_f32());

   let version = match velvet_get_queue_name().as_str() {
    "safe" => 2,
    "unsafe" => 3,
    "crossbeam" => 4,
    _ => -1,
   };
   
    println!("{},{},{},{},{},{},{}", version, velvet_get_num_workers(), a, b, epsilon, THRESHOLD, end.as_secs_f32());
}

#[cfg(not(feature = "test_direct_rec"))]
#[spawnable]
fn adapint(a: f64, b: f64, epsilon: f64) -> f64 {
    let delta = (b - a) / 2.0;
    let deltahalf = delta / 2.0;
    let mid = delta + a;
    let fa = f(a);
    let fb = f(b);
    let fmid = f(mid);
    let total = delta * (fa + fb);
    let left = deltahalf * (fa + fmid);
    let right = deltahalf * (fb + fmid);
    let mut diff = total - (left + right);
    if diff < 0.0 {
        diff = -diff;
    }
    
    if diff < epsilon {
        return total
    }

    if diff <= THRESHOLD as f64 {
        let i1 = adapint_seq(mid, b, epsilon);
        let i2 = adapint_seq(a, mid, epsilon);
        return i1 + i2;
    }

    let i1 = adapint(mid, b, epsilon);
    let i2 = adapint(a, mid, epsilon);
    return i1 + i2;
}

#[cfg(feature = "rayon")]
fn rayon_main(a: f64, b: f64, epsilon: f64, num_threads: usize) {
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
    let res = adapint_rayon(a, b, epsilon);
    let end = start.elapsed();
    eprintln!("RAYON RESULT = {}, in parallel time = {}", res, end.as_secs_f32());

    let _version = 1;

    println!("{},{},{},{},{},{},{}", _version, num_threads, a, b, epsilon, THRESHOLD, end.as_secs_f32());
}
#[cfg(feature = "rayon")]
fn adapint_rayon(a: f64, b: f64, epsilon: f64) -> f64 {
    let delta = (b - a) / 2.0;
    let deltahalf = delta / 2.0;
    let mid = delta + a;
    let fa = f(a);
    let fb = f(b);
    let fmid = f(mid);
    let total = delta * (fa + fb);
    let left = deltahalf * (fa + fmid);
    let right = deltahalf * (fb + fmid);
    let mut diff = total - (left + right);
    if diff < 0.0 {
        diff = -diff;
    }
    
    if diff < epsilon {
        return total
    } else if diff <= THRESHOLD as f64 {
        let i1 = adapint_seq(mid, b, epsilon);
        let i2 = adapint_seq(a, mid, epsilon);
        return i1 + i2; 
    } else {
        let (i1, i2) = rayon::join(
            || adapint_rayon(mid, b, epsilon),
            || adapint_rayon(a, mid, epsilon)
        );
        return i1 + i2;
    }
}

// ---------------------- FOR CHECKING EFFECT OF DIRECT RECURSION ----------
#[cfg(feature = "test_direct_rec")]
enum __Frame__ {
    Stolen(std::sync::Arc<std::sync::Mutex<Option<__Frame__>>>),
    InputAdapint(usize, f64, f64, f64),
    OutputAdapint(f64),
}
#[cfg(feature = "test_direct_rec")]
impl velvet::Identifiable for __Frame__ {
    fn get_id(&self) -> usize {
        if let __Frame__::InputAdapint(uid, ..) = self {
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
                __Frame__::InputAdapint(_, a0, a1, a2) => {
                    let result = adapint(worker, a0, a1, a2);
                    *lock = Some(__Frame__::OutputAdapint(result));
                }
                _ => panic!("WRONG STOLEN WORK FRAME!"),
            }
            return;
        }
        n = (n + 1) % len;
    }
}

#[cfg(feature = "test_direct_rec")]
fn test_direct_recursion(a: f64, b: f64, epsilon: f64, num_workers: usize) {
    let mut __root__worker__ = velvet::VelvetWorker::prepare_workers(num_workers, 64, __velvet_steal__);
    __root__worker__.wait();
    let start = Instant::now();
    let res = adapint(&mut __root__worker__, a, b, epsilon);
    let end = start.elapsed();
    
    let version = match velvet_get_queue_name().as_str() {
        "safe" => 7,
        "unsafe" => 8,
        "crossbeam" => 9,
        _ => -1,
    };

    let _threshold = THRESHOLD;
    
    println!("{},{},{},{},{},{},{}", version, num_workers, a, b, epsilon, _threshold, end.as_secs_f32());
    eprintln!("VELVET RESULT WITHOUT DIRECT RECURSION OPT = {}, in parallel time = {}",  res, end.as_secs_f32());
}
#[cfg(feature = "test_direct_rec")]
fn adapint(
    __worker__: &mut velvet::VelvetWorker<__Frame__>,
    a: f64,
    b: f64,
    epsilon: f64,
) -> f64 {
    let delta = (b - a) / 2.0;
    let deltahalf = delta / 2.0;
    let mid = delta + a;
    let fa = f(a);
    let fb = f(b);
    let fmid = f(mid);
    let total = delta * (fa + fb);
    let left = deltahalf * (fa + fmid);
    let right = deltahalf * (fb + fmid);
    let mut diff = total - (left + right);
    if diff < 0.0 {
        diff = -diff;
    }
    if diff < epsilon {
        return total;
    }

    if diff <= THRESHOLD as f64 {
        let i1 = adapint_seq(mid, b, epsilon);
        let i2 = adapint_seq(a, mid, epsilon);
        return i1 + i2;
    }
    
    let __0__ = __worker__.get_seq();
    __worker__.spawn(__Frame__::InputAdapint(__0__, mid, b, epsilon));
    let __1__ = __worker__.get_seq();
    __worker__.spawn(__Frame__::InputAdapint(__1__, a, mid, epsilon));
    let __SYNC__ = __worker__.sync(__1__);
    let __SYNC_RES__ = match __SYNC__ {
        __Frame__::InputAdapint(_, a0, a1, a2) => {
            adapint(__worker__, a0, a1, a2)
        }
        __Frame__::Stolen(ptr) => {
            let mut try_lock = ptr.try_lock();
            loop {
                if let Ok(mut _value) = try_lock {
                    if let Some(__Frame__::OutputAdapint(result)) = (*_value).take() {
                        break result
                    } else {
                        panic!("WRONG STOLEN RESULT FRAME!");
                    }
                } else {
                    __worker__.steal();
                    try_lock = ptr.try_lock();
                }
            }
        },
        _ => panic!("WRONG STOLEN WORK FRAME!"),
    };
    let i2 = __SYNC_RES__;
    let __SYNC__ = __worker__.sync(__0__);
    let __SYNC_RES__ = match __SYNC__ {
        __Frame__::InputAdapint(_, a0, a1, a2) => {
            adapint(__worker__, a0, a1, a2)
        }
        __Frame__::Stolen(ptr) => {
            let mut try_lock = ptr.try_lock();
            loop {
                if let Ok(mut _value) = try_lock {
                    if let Some(__Frame__::OutputAdapint(result)) = (*_value).take() {
                        break result
                    } else {
                        panic!("WRONG STOLEN RESULT FRAME!");
                    }
                } else {
                    __worker__.steal();
                    try_lock = ptr.try_lock();
                }
            }
        },
        _ => panic!("WRONG STOLEN WORK FRAME!"),
    };
    let i1 = __SYNC_RES__;
    return i1 + i2;
}