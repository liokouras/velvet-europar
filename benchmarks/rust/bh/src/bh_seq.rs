use std::{fs::File, io::{self, BufRead, Write}, time::{Duration, Instant}};
use super::{seq_body::Body, seq_tree::BHTree, quad::Quadrant};

// read input and make bodies
fn read_input_seq(input_file: &str) -> io::Result<(Vec<Body>, f64)> {
    let file = File::open(input_file)?;
    let mut lines = io::BufReader::new(file).lines();

    let n: usize = lines.next().unwrap()?.parse().expect("Invalid number of particles");
    let radius: f64 = lines.next().unwrap()?.parse().expect("Invalid radius of universe");

    let mut bodies: Vec<Body> = Vec::with_capacity(n);
    for _ in 0..n {
        if let Some(Ok(line)) = lines.next() {
            let mut iter = line.split_whitespace();
            let id = iter.next().unwrap().parse().expect("Invalid id");
            let px = iter.next().unwrap().parse().expect("Invalid px");
            let py = iter.next().unwrap().parse().expect("Invalid py");
            let vx = iter.next().unwrap().parse().expect("Invalid vx");
            let vy = iter.next().unwrap().parse().expect("Invalid vy");
            let mass = iter.next().unwrap().parse().expect("Invalid mass");
            bodies.push(Body::new(id, mass, px, py, vx, vy));
        }
    }

    Ok((bodies, radius))
}

// write output
fn _write_output_seq(output_file: &str, radius: f64, bodies: Vec<Body>) -> io::Result<()>  {
    let mut writer = File::create(output_file)?;
    writeln!(writer, "{}", radius)?;
    for body in bodies {
        writeln!(writer, "{} {} {}", body.id(), body.px(), body.py())?;
    }
    Ok(())
}

pub(crate) fn seq_main(input_file: &str, _output_file: &str, num_iterations: u32) {
    let dt: f64 = 0.1; // time quantum

    let (mut bodies, radius) = read_input_seq(input_file).unwrap();

    // run simulation
    let mut duration_tree = Duration::default();
    let mut duration_update_forces = Duration::default();
    let mut duration_update_bodies = Duration::default();
    let start = Instant::now();
    let quad = Quadrant::new(0., 0., radius * 2.);
    for _ in 0..num_iterations {
        // build the Barnes-Hut tree
        let start_tree = Instant::now();
        let mut tree = BHTree::new(quad);
        for body in &bodies {
            if body.inside(&quad) {
                tree.insert((body.id(), body.mass(), body.px(), body.py()));
            }
        }
        duration_tree += start_tree.elapsed();

        // update the forces, positions, velocities, and accelerations
        let start_update_forces = Instant::now();
        tree.traverse_update_force(&tree, &mut bodies);
        duration_update_forces += start_update_forces.elapsed();

        let start_update_bodies = Instant::now();
        for body in &mut bodies {
            body.update(dt);
        }
        duration_update_bodies += start_update_bodies.elapsed();
    }
    let duration = start.elapsed();
    println!("0,1,0,0,{},{},{},{}", duration.as_secs_f32(), duration_tree.as_secs_f32(), duration_update_forces.as_secs_f32(), duration_update_bodies.as_secs_f32());

    // let output_filename = _output_file.replace(".txt", "_seq.txt");
    // _write_output_seq(&output_filename, radius, bodies).unwrap();
}