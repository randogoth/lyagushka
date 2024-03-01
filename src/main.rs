use std::fs::File;
use std::io::{self, BufRead, BufReader, stdin};
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
    span_length: f32,
    num_elements: usize,
    centroid: f32,
    z_score: Option<f32>,
}

fn create_cluster_info(cluster: &[Point]) -> ClusterGapInfo {
    
    let num_elements = cluster.len();
    let span_length = (cluster.last().unwrap().value as f32) - (cluster.first().unwrap().value as f32);
    let centroid = cluster.iter().map(|p| p.value as f32).sum::<f32>() / num_elements as f32;

    ClusterGapInfo {
        span_length,
        num_elements,
        centroid,
        z_score: None, 
    }
}

/// Calculates the densities (clusters) and significant gaps between points in a dataset.
///
/// This function iterates over a dataset of points, identifying clusters based on a distance threshold
/// (calculated from the mean distance between points and adjusted by a given factor) and identifying significant gaps
/// that exceed a certain threshold. Each cluster or significant gap identified is summarized in a `ClusterGapInfo` object.
///
/// # Arguments
/// * `dataset`: A slice of `Point` objects representing the dataset to be analyzed.
/// * `factor`: A multiplier used to define the thresholds for clustering and gap identification. 
///   A lower factor tightens the cluster threshold and widens the gap threshold, and vice versa.
/// * `min_cluster_size`: The minimum number of points required for a group of points to be considered a cluster.
///
/// # Returns
/// A vector of `ClusterGapInfo` objects, each representing either a cluster of points or a significant gap between points.
///
fn calculate_densities_and_gaps(dataset: &[Point], factor: f32, min_cluster_size: usize) -> Vec<ClusterGapInfo> {
    
    // Return early if the dataset is too small to form any clusters or gaps.
    if dataset.len() < 2 { return Vec::new(); }

    // Calculate the mean distance between consecutive points in the dataset.
    let mean_distance = dataset.windows(2)
                               .map(|w| w[1].value as f32 - w[0].value as f32)
                               .sum::<f32>() / (dataset.len() - 1) as f32;

    // Define thresholds for clustering and gap identification based on the mean distance and factor.
    let cluster_threshold = mean_distance / factor;
    let gap_threshold = factor * mean_distance * 2.0;

    let mut results: Vec<ClusterGapInfo> = Vec::new(); // Stores the resulting clusters and gaps.
    let mut current_cluster: Vec<Point> = Vec::new(); // Temporary storage for points in the current cluster.

    // Iterate through pairs of consecutive points to find clusters and significant gaps.
    for window in dataset.windows(2) {
        let gap_distance = window[1].value as f32 - window[0].value as f32;

        // If the distance between points is within the cluster threshold, add to current cluster.
        if gap_distance <= cluster_threshold {
            if current_cluster.is_empty() {
                current_cluster.push(window[0].clone()); // Start a new cluster with the first point.
            }
            current_cluster.push(window[1].clone()); // Add the second point to the cluster.
        } else {
            // If the current cluster is large enough, finalize it and prepare for a new cluster.
            if !current_cluster.is_empty() && current_cluster.len() >= min_cluster_size {
                results.push(create_cluster_info(&current_cluster));
                current_cluster.clear();
            }

            // If the gap between points is significant, record it as a gap.
            if gap_distance > gap_threshold {
                results.push(ClusterGapInfo {
                    span_length: gap_distance,
                    num_elements: 0, // Indicating this is a gap, not a cluster.
                    centroid: (window[0].value as f32 + window[1].value as f32) / 2.0,
                    z_score: None, // Z-score will be calculated later if necessary.
                });
            }
        }
    }

    // Finalize the last cluster if it meets the size requirement.
    if !current_cluster.is_empty() && current_cluster.len() >= min_cluster_size {
        results.push(create_cluster_info(&current_cluster));
    }

    results
}

/// Analyzes a dataset of points to identify clusters and significant gaps, calculates z-scores for each,
/// and serializes the results to a JSON string.
///
/// This function takes a vector of `Point` structs, a factor for adjusting clustering and gap detection thresholds,
/// and a minimum cluster size. It performs an analysis to identify clusters of points that are closely grouped
/// together and significant gaps between these clusters. For each cluster or gap, it calculates a z-score that
/// indicates how far the centroid or span length deviates from the mean distance of the dataset. The results
/// of this analysis are then serialized into a JSON string.
///
/// # Arguments
/// * `dataset` - A vector of `Point` structs representing the dataset to be analyzed.
/// * `factor` - A floating-point value used to adjust the sensitivity of cluster and gap detection. Lower values
///   result in tighter clustering and wider gaps, while higher values do the opposite.
/// * `min_cluster_size` - The minimum number of contiguous points required to be considered a cluster.
///
/// # Returns
/// Returns a `String` containing the JSON-serialized analysis results, including clusters and gaps with their z-scores.
///
fn lyagushka(dataset: Vec<Point>, factor: f32, min_cluster_size: usize) -> String {

    // Analyze the dataset to identify clusters and significant gaps.
    let mut cluster_gap_infos = calculate_densities_and_gaps(&dataset, factor, min_cluster_size);

    // Calculate the mean distance between consecutive points in the dataset.
    let mean_distance: f32 = if dataset.len() > 1 {
        dataset.windows(2)
            .map(|w| w[1].value as f32 - w[0].value as f32)
            .sum::<f32>() / (dataset.len() - 1) as f32
    } else {
        0.0
    };

    // Calculate the standard deviation of distances between consecutive points.
    let std_deviation: f32 = if dataset.len() > 1 {
        (dataset.windows(2)
            .map(|w| w[1].value as f32 - w[0].value as f32 - mean_distance)
            .map(|d| d.powi(2))
            .sum::<f32>() / (dataset.len() - 1) as f32)
        .sqrt()
    } else {
        0.0
    };

    // Calculate and assign z-scores for each cluster/gap based on their centroid or span length.
    for info in cluster_gap_infos.iter_mut() {
        info.z_score = Some(if info.num_elements > 0 {
            // For clusters, use the centroid for z-score calculation.
            (info.centroid - mean_distance) / std_deviation
        } else {
            // For gaps, use the span length for z-score calculation.
            (info.span_length - mean_distance) / std_deviation
        });
    }

    serde_json::to_string_pretty(&cluster_gap_infos).unwrap_or_else(|_| "Failed to serialize data".to_string())
}

/// The entry point for the command-line tool that reads a dataset of integers from either a file or stdin,
/// performs cluster and gap analysis using specified parameters, and prints the results as a JSON string.
///
/// This tool expects either a filename as an argument or a list of integers piped into stdin. It also requires
/// two additional command-line arguments: a factor for adjusting clustering and gap detection thresholds,
/// and a minimum cluster size. The tool reads the dataset, performs the analysis by identifying clusters
/// and significant gaps, calculates z-scores for each, and prints the JSON-serialized results to stdout.
///
/// # Usage
/// To read from a file:
/// ```
/// cargo run -- filename.txt 0.5 2
/// ```
///
/// To read from stdin:
/// ```
/// echo "1\n2\n10\n20" | cargo run -- 0.5 2
/// ```
///
/// # Arguments
/// - A filename (if not receiving piped input) to read the dataset from.
/// - `factor`: A floating-point value used to adjust the sensitivity of cluster and gap detection.
/// - `min_cluster_size`: The minimum number of contiguous points required to be considered a cluster.
///
/// # Exit Codes
/// - `0`: Success.
/// - `1`: Incorrect usage or failure to parse the input data.
///
/// # Errors
/// This tool will exit with an error if the required arguments are not provided, if the specified file cannot be opened,
/// or if the input data cannot be parsed into integers.
///
/// # Note
/// This function does not return a value but directly exits the process in case of failure.
///
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Input handling
    let dataset: Vec<Point> = if atty::is(atty::Stream::Stdin) {
        if args.len() != 4 {
            eprintln!("Usage: {} <filename> <factor> <min_cluster_size>", args[0]);
            process::exit(1);
        }
        let filename = &args[1];
        let file = File::open(filename)?;
        BufReader::new(file).lines().filter_map(Result::ok)
            .filter_map(|line| line.trim().parse::<u32>().ok())
            .map(Point::new)
            .collect()
    } else {
        stdin().lock().lines().filter_map(Result::ok)
            .filter_map(|line| line.trim().parse::<u32>().ok())
            .map(Point::new)
            .collect()
    };

    let factor: f32 = args[args.len() - 2].parse().expect("Factor must be a float");
    let min_cluster_size: usize = args[args.len() - 1].parse().expect("Min cluster size must be an integer");

    // Analysis and output
    println!("{}", lyagushka(dataset, factor, min_cluster_size));

    Ok(())
}