#![allow(non_snake_case)]
use velvet::prelude::*;
use std::time::Instant;
use std::env;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[cfg(not(feature = "test_direct_rec"))]
include!(concat!(env!("OUT_DIR"), "/velvet_app.rs"));

const THRESHOLD: usize = parse_threshold();
const fn parse_threshold() -> usize {
    if let Some(string) = option_env!("THRESHOLD") {
        let mut res: usize = 0;
        let mut bytes = string.as_bytes();
        while let [byte, rest @ ..] = bytes {
            assert!(b'0' <= *byte && *byte <= b'9', "invalid digit");
            res *= 10;
            res += (*byte - b'0') as usize;
            bytes = rest;
        }
        res
    } else {
        6
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("usage for nqueens: cargo run [cargo_options] [velvet|seq|rayon] [n]");
        println!("example: cargo run --release velvet 15");
        return;
    }
    let app = &args[1];
    let n: usize = args[2].parse().unwrap();
    
    if app.eq("velvet") {
        #[cfg(not(feature = "test_direct_rec"))]
        velvet_main(n);
        #[cfg(feature = "test_direct_rec")]
        println!("Compile without feature \"test_direct_rec\"");
    } else if app.eq("rayon") {
        #[cfg(feature = "rayon")]
        {
            if args.len() < 4 {
                println!("must provide number of threads for Rayon");
                println!("usage: cargo run [cargo_options] [velvet|seq|rayon] [n] [number_of_threads]");
                println!("example: cargo run --release rayon 15 8");
                return;
            }
            let num_workers: usize = args[3].parse().unwrap();
            rayon_main(n, num_workers);
        }
        #[cfg(not(feature = "rayon"))]
        println!("COMPILE WITH RAYON!");
    } else if app.eq("par") {
        // pin thread!
        let core_ids = core_affinity::get_core_ids().unwrap();
        let res = core_affinity::set_for_current(core_ids[0]);
        if !res {
            eprintln!("Could not pin Root thread id continuing without pinning...");
        }

        let board = vec![0;n];

        let start = Instant::now();
        let res = nqueens_par(board, 0, n);
        let end = start.elapsed();
        eprintln!("PAR-SEQ RESULT = {}, in time = {}", res, end.as_secs_f32());
    
        println!("-1,1,{},{},{}", n, THRESHOLD, end.as_secs_f32());
    } else if app.eq("test_direct") {
        #[cfg(feature = "test_direct_rec")]
        {
            if args.len() < 4 {
                println!("must provide number of workers!");
                return;
            }
            let num_workers: usize = args[3].parse().unwrap();
            test_direct_recursion(n, num_workers);
        }
        #[cfg(not(feature = "test_direct_rec"))]
        println!("Compile with feature \"test_direct_rec\"");
    } else if app.eq("seq") {
        // pin thread!
        let core_ids = core_affinity::get_core_ids().unwrap();
        let res = core_affinity::set_for_current(core_ids[0]);
        if !res {
            eprintln!("Could not pin Root thread id continuing without pinning...");
        }

        let mut board = vec![0;n];

        let start_seq = Instant::now();
        let oracle = nqueens(&mut board, 0, n);
        let end_seq = start_seq.elapsed();
        eprintln!("ORACLE = {}, in seq time = {}", oracle, end_seq.as_secs_f32());
        println!("0,1,{},0,{}", n, end_seq.as_secs_f32());
    } else {
        eprintln!("Unrecognised app {}", app);
    }
}

#[cfg(not(feature = "test_direct_rec"))]
#[velvet_main(nqueens_spawn)]
fn velvet_main(n: usize) {
    let board = vec![0;n];

    let start = Instant::now();
    let res = nqueens_spawn(board, 0, n);
    let end = start.elapsed();
    eprintln!("VELVET RESULT = {}, in parallel time = {}", res, end.as_secs_f32());

   let version = match velvet_get_queue_name().as_str() {
    "safe" => 2,
    "unsafe" => 3,
    "crossbeam" => 4,
    _ => -1,
   };
   
    println!("{},{},{},{},{}", version, velvet_get_num_workers(), n, THRESHOLD, end.as_secs_f32());
}

#[cfg(not(feature = "test_direct_rec"))]
#[spawnable]
fn nqueens_spawn(mut board: Vec<u8>, row: usize, size: usize) -> usize {
    if row > THRESHOLD {
        return nqueens(&mut board, row, size);
    }
    if row >= size {
        return 1;
    }

    let mut solutions = 0;

    'try_new_row: for q in 0..size {
        // incremental conflict check
        for i in 0..row {
            let p = board[i] as isize - q as isize;
            let d = (row - i) as isize;
            if p == 0 || p == d || p == -d {
                continue 'try_new_row;
            }
        }

        // par recursion: copy board
        let mut new_board = board.clone();
        new_board[row] = q as u8;
        solutions += nqueens_spawn(new_board, row + 1, size);
    }

    solutions
}

fn nqueens_par(mut board: Vec<u8>, row: usize, size: usize) -> usize {
    if row > THRESHOLD {
        return nqueens(&mut board, row, size);
    }
    if row >= size {
        return 1;
    }

    let mut solutions = 0;

    'try_new_row: for q in 0..size {
        // incremental conflict check
        for i in 0..row {
            let p = board[i] as isize - q as isize;
            let d = (row - i) as isize;
            if p == 0 || p == d || p == -d {
                continue 'try_new_row;
            }
        }

        // par recursion: copy board
        let mut new_board = board.clone();
        new_board[row] = q as u8;
        solutions += nqueens_par(new_board, row + 1, size);
    }

    solutions
}

fn nqueens(board: &mut [u8], row: usize, size: usize) -> usize { // difference: &mut vs owned board
    if row >= size {
        return 1;
    }

    let mut solutions = 0;

    'try_new_row: for q in 0..size {
        // incremental conflict check
        for i in 0..row {
            let p = board[i] as isize - q as isize;
            let d = (row - i) as isize;
            if p == 0 || p == d || p == -d {
                continue 'try_new_row;
            }
        }

        // sequential recursion: reuse board <-- different!! no cloning to new board
        board[row] = q as u8;
        solutions += nqueens(board, row + 1, size);
    }

    solutions
}

fn _nqueens_fast(board: &mut Vec<u8>, row: usize, size: usize) -> usize {
    if row > THRESHOLD {
        return nqueens(board, row, size);
    }
    if row >= size {
        return 1;
    }

    let solutions = (0..size)
        .into_iter()
        .map(|q| {
            for i in 0..row {
                let p = board[i] as isize - q as isize;
                let d = (row - i) as isize;
                if p == 0 || p == d || p == -d {
                    return 0;
                }
            }

            // par recursion: copy board
            let mut new_board = board.clone();
            new_board[row] = q as u8;
            _nqueens_fast(&mut new_board, row + 1, size)
        }).sum();

    solutions
}

#[cfg(feature = "rayon")]
fn rayon_main(n: usize, threads: usize) {
    let cores = core_affinity::get_core_ids().unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .start_handler(move |index| {
            let core_id = cores[index % cores.len()];
            let res = core_affinity::set_for_current(core_id);
            if !res {
                eprintln!("Could not pin worker thread id {:?}, continuing without pinning...", core_id.id);
            }
        })
        .build_global()
        .unwrap();
    
    let mut board = vec![0;n];

    let start = Instant::now();
    let res = nqueens_rayon(&mut board, 0, n);
    let end = start.elapsed();

    eprintln!("RAYON RESULT = {}, in parallel time = {}", res, end.as_secs_f32());
    println!("1,{},{},{},{}", threads, n, THRESHOLD, end.as_secs_f32());
}

#[cfg(feature = "rayon")]
fn nqueens_rayon(board: &mut Vec<u8>, row: usize, size: usize) -> usize {
    if row > THRESHOLD {
        return nqueens(board, row, size);
    }
    if row >= size {
        return 1;
    }

    let solutions = (0..size)
        .into_par_iter()
        .map(|q| {
            for i in 0..row {
                let p = board[i] as isize - q as isize;
                let d = (row - i) as isize;
                if p == 0 || p == d || p == -d {
                    return 0;
                }
            }

            // par recursion: copy board
            let mut new_board = board.clone();
            new_board[row] = q as u8;
            nqueens_rayon(&mut new_board, row + 1, size)
        }).sum();

    solutions
}


// ---------------------- FOR CHECKING EFFECT OF DIRECT RECURSION ----------
#[cfg(feature = "test_direct_rec")]
enum __Frame__ {
    Stolen(std::sync::Arc<std::sync::Mutex<Option<__Frame__>>>),
    InputNqueensSpawn(usize, Vec<u8>, usize, usize),
    OutputNqueensSpawn(usize),
}
#[cfg(feature = "test_direct_rec")]
impl velvet::Identifiable for __Frame__ {
    fn get_id(&self) -> usize {
        if let __Frame__::InputNqueensSpawn(uid, ..) = self {
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
                __Frame__::InputNqueensSpawn(_, a0, a1, a2) => {
                    let result = crate::nqueens_spawn(worker, a0, a1, a2);
                    *lock = Some(__Frame__::OutputNqueensSpawn(result));
                }
                _ => panic!("WRONG STOLEN WORK FRAME!"),
            }
            return;
        }
        n = (n + 1) % len;
    }
}

#[cfg(feature = "test_direct_rec")]
fn test_direct_recursion(n: usize, num_workers: usize) {
    let mut __root__worker__ = velvet::VelvetWorker::prepare_workers(num_workers, 64, __velvet_steal__);
    __root__worker__.wait();
    let board = vec![0;n];
    let start = Instant::now();
    let res = nqueens_spawn(&mut __root__worker__, board, 0, n);
    let end = start.elapsed();
    eprintln!("VELVET RESULT = {}, in parallel time = {}", res, end.as_secs_f32());
    let version = match velvet_get_queue_name().as_str() {
        "safe" => 7,
        "unsafe" => 8,
        "crossbeam" => 9,
        _ => -1,
    };
    println!("{},{},{},{},{}", version, num_workers, n, THRESHOLD, end.as_secs_f32());
}

#[cfg(feature = "test_direct_rec")]
fn nqueens_spawn(
    __worker__: &mut velvet::VelvetWorker<__Frame__>,
    mut board: Vec<u8>,
    row: usize,
    size: usize,
) -> usize {
    let mut __checkpoint__ = __worker__.get_seq();
    let mut __count__ = 0;
    if row > THRESHOLD {
        return nqueens(&mut board, row, size);
    }
    if row >= size {
        return 1;
    }
    let mut solutions = 0;
    'try_new_row: for q in 0..size-1 { // save last iter!
        for i in 0..row {
            let p = board[i] as isize - q as isize;
            let d = (row - i) as isize;
            if p == 0 || p == d || p == -d {
                continue 'try_new_row;
            }
        }
        let mut new_board = board.clone();
        new_board[row] = q as u8;
        let __uid__ = __worker__.get_seq();
        __worker__.spawn(__Frame__::InputNqueensSpawn(__uid__, new_board, row + 1, size));
        __count__ += 1;
    }
    // do last iter directly
    let mut breaking = false;
    let q = size-1;
    for i in 0..row {
        let p = board[i] as isize - q as isize;
        let d = (row - i) as isize;
        if p == 0 || p == d || p == -d {
            breaking = true;
            break;
        }
    }
    if !breaking {
        let mut new_board = board.clone();
        new_board[row] = q as u8;
        solutions += nqueens_spawn(__worker__, new_board, row + 1, size);
    }

    while __count__ > 0 {
        let __SYNC__ = __worker__.sync(__checkpoint__ + __count__);
        let __SYNC_RES__ = match __SYNC__ {
            crate::__Frame__::InputNqueensSpawn(_, a0, a1, a2) => {
                nqueens_spawn(__worker__, a0, a1, a2)
            }
            crate::__Frame__::Stolen(ptr) => {
                let mut try_lock = ptr.try_lock();
                loop {
                    if let Ok(mut _value) = try_lock {
                        if let Some(crate::__Frame__::OutputNqueensSpawn(result)) = (*_value)
                            .take()
                        {
                            break result
                        } else {
                            panic!("WRONG STOLEN RESULT FRAME!");
                        }
                    } else {
                        __worker__.steal();
                        try_lock = ptr.try_lock();
                    }
                }
            }
            _ => panic!("WRONG FRAME POPPED!"),
        };
        solutions += __SYNC_RES__;
        __count__ -= 1;
    }
    solutions
}