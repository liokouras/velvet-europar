use std::{env, path::Path, fs};
mod bh_par;
mod bh_seq;
pub(crate) mod par_tree;
pub(crate) mod seq_tree;
pub(crate) mod par_body;
pub(crate) mod seq_body;
pub(crate) mod quad;
use bh_par::par_main;
use bh_seq::seq_main;
#[cfg(not(feature = "test_direct_rec"))]
include!(concat!(env!("OUT_DIR"), "/velvet_app.rs"));

pub(crate) static G:f64 = 6.67e-11;
pub(crate) static THETA: f64 = 0.5;
pub(crate) static LEAF_CAP: usize = parse_leafcap();
const fn parse_leafcap() -> usize {
    if let Some(string) = option_env!("LEAF_CAP") {
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
        4
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        println!("usage for barnes hut: cargo run [cargo_options] [velvet|seq|rayon] [input_file] [output_file] [num_iters]");
        println!("example: cargo run --release velvet ./inputs.txt ./outputs.txt 100");
        return;
    }
    let app = &args[1];
    let input_file = &args[2];
    let output_file = &args[3];
    let num_iterations: u32 = args[4].parse().expect("Invalid number of iterations");


    // check if output_file contains a directory path and create it if necessary
    if let Some(output_dir) = Path::new(output_file).parent() {
        if !output_dir.exists() {
            if let Err(e) = fs::create_dir_all(output_dir) {
                println!("Failed to create output directory: {}", e);
                return;
            }
        }
    }

    if app.eq("velvet") {
        #[cfg(not(feature = "test_direct_rec"))]
        {
            use bh_par::velvet_main;
            velvet_main(input_file, output_file, num_iterations);
        }
    } else if app.eq("rayon_treeiter") {
        #[cfg(feature = "rayon")]
        {
            if args.len() < 6 {
                println!("must provide number of threads for Rayon");
                println!("usage: cargo run [cargo_options] [velvet|seq|rayon] [input_file] [output_file] [num_iters] [number_of_threads]");
                println!("example: cargo run --release rayon ./inputs.txt ./outputs.txt 100");
                return;
            }
            use bh_par::rayon_main;

            let num_threads: usize = args[5].parse().unwrap();
            rayon_main(input_file, output_file, num_iterations, 1, num_threads);
        }
        #[cfg(not(feature = "rayon"))]
        println!("COMPILE WITH RAYON!");
    } else if app.eq("rayon_pariter") {
        #[cfg(feature = "rayon")]
        {
            if args.len() < 6 {
                println!("must provide number of threads for Rayon");
                println!("usage: cargo run [cargo_options] [velvet|seq|rayon] [input_file] [output_file] [num_iters] [number_of_threads]");
                println!("example: cargo run --release rayon ./inputs.txt ./outputs.txt 100");
                return;
            }
            use bh_par::rayon_main;

            let num_threads: usize = args[5].parse().unwrap();
            rayon_main(input_file, output_file, num_iterations, 5, num_threads);
        }
        #[cfg(not(feature = "rayon"))]
        println!("COMPILE WITH RAYON!");     
    } else if app.eq("rayon_iterative") {
        #[cfg(feature = "rayon")]
        {
            if args.len() < 6 {
                println!("must provide number of threads for Rayon");
                println!("usage: cargo run [cargo_options] [velvet|seq|rayon] [input_file] [output_file] [num_iters] [number_of_threads]");
                println!("example: cargo run --release rayon ./inputs.txt ./outputs.txt 100");
                return;
            }
            use bh_par::rayon_iterative;

            let num_threads: usize = args[5].parse().unwrap();
            rayon_iterative(input_file, output_file, num_iterations, 6, num_threads);
        }
        #[cfg(not(feature = "rayon"))]
        println!("COMPILE WITH RAYON!");   
    } else if app.eq("par_seq") {
        par_main(input_file, output_file, num_iterations);
    } else if app.eq("test_direct") {
        #[cfg(feature = "test_direct_rec")]
        {
            if args.len() < 6 {
                println!("must provide number of workers!");
                return;
            }
            use bh_par::test_direct_recursion;
            let num_workers: usize = args[5].parse().unwrap();
            test_direct_recursion(input_file, output_file, num_iterations, num_workers);
        }
        #[cfg(not(feature = "test_direct_rec"))]
        println!("Compile with feature \"test_direct_rec\"");
    } else if app.eq("seq") {
        seq_main(input_file, output_file, num_iterations);
    } else {
        eprintln!("could not recognize app: {}", app);
    }
}
#[cfg(feature = "test_direct_rec")]
pub(crate) enum __Frame__ {
    Stolen(std::sync::Arc<std::sync::Mutex<Option<__Frame__>>>),
    InputTraverseSpawn(usize, std::sync::Arc<crate::par_tree::BHTree>, std::sync::Arc<crate::par_tree::BHTree>),
}
#[cfg(feature = "test_direct_rec")]
impl velvet::Identifiable for __Frame__ {
    fn get_id(&self) -> usize {
        if let __Frame__::InputTraverseSpawn(uid, ..) = self {
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
                __Frame__::InputTraverseSpawn(_, a0, a1) => {
                    a0.traverse_spawn(worker, a1);
                    *lock = None;
                }
                _ => panic!("WRONG STOLEN WORK FRAME!"),
            }
            return;
        }
        n = (n + 1) % len;
    }
}