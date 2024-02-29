use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::types::PyList;
use std::fs::File;
// use std::io::{self, BufRead, BufReader, stdin, Read};
use std::io::{self, BufRead, BufReader};
use std::env;
use std::process;
use serde::Serialize;
use serde_json;

#[derive(Clone, Debug, Serialize)]
struct Point {
    value: u32,
}

impl Point {
    fn new(value: u32) -> Self {
        Point { value }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ClusterGapInfo {
    span_length: f32, // Full span length
    num_elements: usize, // Number of elements, 0 for gaps
    centroid: f32, // Centroid value
    z_score: Option<f32>, // Z-score, to be calculated later
}


fn load_dataset(filename: &str) -> io::Result<Vec<Point>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut dataset = Vec::new();

    for line in reader.lines() {
        let value: u32 = line?.trim().parse().unwrap();
        dataset.push(Point::new(value));
    }

    dataset.sort_by_key(|p| p.value);
    Ok(dataset)
}

// Define the ClusterGapInfo struct as described above

fn calculate_densities_and_gaps(dataset: &[Point], factor: f32, min_cluster_size: usize) -> Vec<ClusterGapInfo> {
    let mut results: Vec<ClusterGapInfo> = Vec::new();
    if dataset.len() < 2 {
        return results;
    }

    let mean_distance = dataset.windows(2)
                               .map(|w| distance(&w[0], &w[1]) as f32)
                               .sum::<f32>() / (dataset.len() - 1) as f32;

    let cluster_threshold = 1.0 / factor * mean_distance;
    let gap_threshold = factor * mean_distance * 2.0;

    let mut current_cluster = Vec::new();
    for window in dataset.windows(2) {
        let gap_distance = distance(&window[0], &window[1]) as f32;
        if gap_distance <= cluster_threshold {
            current_cluster.push(window[1].clone());
        } else {
            // Before clearing the current_cluster, check if it meets the size requirement
            if !current_cluster.is_empty() && current_cluster.len() >= min_cluster_size {
                let cluster_info = create_cluster_info(&current_cluster);
                results.push(cluster_info);
            }
            current_cluster.clear();

            // Add a gap if the distance exceeds the gap threshold
            if gap_distance > gap_threshold {
                results.push(ClusterGapInfo {
                    span_length: gap_distance,
                    num_elements: 0,
                    centroid: (window[0].value as f32 + window[1].value as f32) / 2.0,
                    z_score: None,
                });
            }
        }
    }

    // Handle the last cluster if it meets the size requirement
    if !current_cluster.is_empty() && current_cluster.len() >= min_cluster_size {
        let cluster_info = create_cluster_info(&current_cluster);
        results.push(cluster_info);
    }

    results
}

// Additional helper function to create cluster information
fn create_cluster_info(cluster: &[Point]) -> ClusterGapInfo {
    let num_elements = cluster.len();
    let span_length = (cluster.last().unwrap().value as f32) - (cluster.first().unwrap().value as f32);
    let centroid = cluster.iter().map(|p| p.value as f32).sum::<f32>() / num_elements as f32;

    ClusterGapInfo {
        span_length,
        num_elements,
        centroid,
        z_score: None, // Placeholder, to be calculated later
    }
}

fn distance(p1: &Point, p2: &Point) -> u32 {
    if p1.value > p2.value { p1.value - p2.value } else { p2.value - p1.value }
}


fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut dataset: Vec<Point> = Vec::new();

    // Check if data is being piped into the program
    if atty::is(atty::Stream::Stdin) {
        // Not receiving piped input, expect filename as argument
        if args.len() < 4 {
            eprintln!("Usage: {} <filename> <factor> <min_cluster_size>", args[0]);
            eprintln!("Or pipe in a list of integers and provide <factor> <min_cluster_size>");
            process::exit(1);
        }

        let filename = &args[1];
        dataset = load_dataset(filename)?;
    } else {
        // Receiving piped input, read from stdin
        let stdin = io::stdin();
        let reader = stdin.lock();
        for line in reader.lines() {
            let value: u32 = line?.trim().parse().unwrap();
            dataset.push(Point::new(value));
        }
        dataset.sort_by_key(|p| p.value);
    }

    let factor: f32 = args[args.len() - 2].parse().expect("Factor must be a float");
    let min_cluster_size: usize = args[args.len() - 1].parse().expect("Min cluster size must be an integer");

    let mut cluster_gap_infos = calculate_densities_and_gaps(&dataset, factor, min_cluster_size);

    // Calculate mean distance for Z-score computation
    let total_distances: f32 = dataset.windows(2)
                                      .map(|w| (w[1].value as f32 - w[0].value as f32))
                                      .sum();
    let mean_distance = total_distances / (dataset.len() as f32 - 1.0);

    // Calculate Z-scores for clusters and gaps
    for info in cluster_gap_infos.iter_mut() {
        if info.num_elements == 0 {
            // Z-score for gaps
            info.z_score = Some((info.span_length - mean_distance) / mean_distance); // Simplified deviation measure
        } else {
            // Z-score for clusters, based on density deviation
            let density = info.num_elements as f32 / info.span_length;
            let expected_density = 1.0 / mean_distance; // Expected: one element per mean distance
            info.z_score = Some((density - expected_density) / expected_density); // Simplified deviation measure
        }
    }

    // Convert cluster_gap_infos to JSON
    let json = serde_json::to_string_pretty(&cluster_gap_infos).expect("Failed to serialize to JSON");

    // Output the JSON string
    println!("{}", json);

    Ok(())
}

#[pyfunction]
fn traktor(py: Python, int_list: &PyList, factor: f32, min_cluster_size: usize) -> PyResult<String> {
    // Convert Python list to Rust Vec<Point>
    let mut dataset: Vec<Point> = Vec::new();
    for py_any in int_list.into_iter() {
        let value: u32 = py_any.extract()?;
        dataset.push(Point::new(value));
    }
    dataset.sort_by_key(|p| p.value);

    // Proceed with your existing logic
    let mut cluster_gap_infos = calculate_densities_and_gaps(&dataset, factor, min_cluster_size);

    // Calculate mean distance for Z-score computation
    let total_distances: f32 = dataset.windows(2)
                                      .map(|w| (w[1].value as f32 - w[0].value as f32))
                                      .sum();
    let mean_distance = total_distances / (dataset.len() as f32 - 1.0);

    // Calculate Z-scores for clusters and gaps
    for info in cluster_gap_infos.iter_mut() {
        if info.num_elements == 0 {
            // Z-score for gaps
            info.z_score = Some((info.span_length - mean_distance) / mean_distance); // Simplified deviation measure
        } else {
            // Z-score for clusters, based on density deviation
            let density = info.num_elements as f32 / info.span_length;
            let expected_density = 1.0 / mean_distance; // Expected: one element per mean distance
            info.z_score = Some((density - expected_density) / expected_density); // Simplified deviation measure
        }
    }

    // Serialize to JSON and return
    let json = serde_json::to_string_pretty(&cluster_gap_infos)
        .expect("Failed to serialize to JSON");
        
    Ok(json)
}

#[pymodule]
fn lyagushka(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(traktor, m)?)?;
    Ok(())
}