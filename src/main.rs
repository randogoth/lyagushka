use std::fs::File;
use std::io::{self, BufRead, BufReader, stdin};
use std::env;
use std::process;
use serde::Serialize;
use serde_json;

#[derive(Debug, Clone, Serialize)]
struct Anomaly {
    elements: Vec<i32>,
    start: i32,
    end: i32,
    span_length: i32,
    num_elements: usize,
    centroid: f32,
    z_score: Option<f32>,
}

impl Anomaly {

    pub fn new(cluster: &[i32]) -> Self {
        let num_elements: usize = cluster.len();
        let start: i32 = *cluster.first().expect("Cluster has no start");
        let end: i32 = *cluster.last().expect("Cluster has no end");
        let span_length: i32 = end - start;
        let centroid: f32 = start as f32 + span_length as f32 / 2.0;

        Anomaly {
            elements: cluster.to_vec(),
            start,
            end,
            span_length,
            num_elements,
            centroid,
            z_score: None,
        }
    }
}

struct Lyagushka {
    dataset: Vec<i32>,
    anomalies: Vec<Anomaly>,
}

impl Lyagushka {

    pub fn new(dataset: Vec<i32>) -> Self {
        Lyagushka {
            dataset,
            anomalies: vec![]
        }
    }

    fn scan_anomalies(&mut self, factor: f32, min_cluster_size: usize) {
    
        // Calculate the mean distance between consecutive points in the dataset.
        let mean_distance: f32 = self.dataset.windows(2)
                                        .map(|w| (w[1] - w[0]) as f32)
                                        .sum::<f32>() / (self.dataset.len() - 1) as f32;
    
        // Define thresholds for clustering and gap identification based on the mean distance and factor.
        let cluster_threshold: f32 = mean_distance / factor;
        let gap_threshold: f32 = factor * mean_distance;
    
        let mut current_cluster: Vec<i32> = Vec::new(); // Temporary storage for points in the current cluster.
    
        // Iterate through pairs of consecutive points to find clusters and significant gaps.
        for window in self.dataset.windows(2) {
            let gap_size: f32 = (window[1] - window[0]) as f32;
    
            if gap_size <= cluster_threshold {
                // Add points to the current cluster
                if current_cluster.is_empty() {
                    current_cluster.push(window[0]); // Start a new cluster with the first point
                }
                current_cluster.push(window[1]); // Add the second point to the cluster
            } else {
                // End the current cluster and start a new gap
                if !current_cluster.is_empty() && current_cluster.len() >= min_cluster_size {
                    self.anomalies.push(Anomaly::new(&current_cluster));
                    current_cluster.clear();
                }
    
                // Record the gap
                if gap_size > gap_threshold {
                    self.anomalies.push(Anomaly {
                        elements: Vec::new(), // No elements in a gap
                        start: window[0],
                        end: window[1],
                        span_length: gap_size as i32,
                        num_elements: 0,
                        centroid: (window[0] as f32 + window[1] as f32) / 2.0,
                        z_score: None,
                    });
                }
            }
        }
    
        // Finalize the last cluster if applicable
        if !current_cluster.is_empty() && current_cluster.len() >= min_cluster_size {
            self.anomalies.push(Anomaly::new(&current_cluster));
        }
    
    }

    pub fn search(&mut self, factor: f32, min_cluster_size: usize) -> String {

        // Sort the vector
        self.dataset.sort_unstable();
    
        // Calculate clusters and gaps from the dataset using predefined criteria.
        self.scan_anomalies(factor, min_cluster_size);
    
        // Calculate the mean density of clusters in the dataset for comparison.
        let mean_density: f32 = self.anomalies.iter()
            .filter(|info: &&Anomaly| info.num_elements > 0)
            .map(|info: &Anomaly| info.num_elements as f32 / info.span_length as f32)
            .sum::<f32>() / self.anomalies.iter().filter(|info: &&Anomaly| info.num_elements > 0).count() as f32;
    
        // Calculate the standard deviation of cluster densities to evaluate variation.
        let variance_density: f32 = self.anomalies.iter()
            .filter(|info: &&Anomaly| info.num_elements > 0)
            .map(|info: &Anomaly| info.num_elements as f32 / info.span_length as f32)
            .map(|density: f32| (density - mean_density).powi(2))
            .sum::<f32>() / self.anomalies.iter().filter(|info: &&Anomaly| info.num_elements > 0).count() as f32;
        let std_dev_density: f32 = variance_density.sqrt();
    
        // Calculate mean span length
        let mean_span_length: f32 = self.anomalies.iter()
            .map(|info: &Anomaly| info.span_length as f32)
            .sum::<f32>() / self.anomalies.len() as f32;
    
        // Calculate variance
        let variance: f32 = self.anomalies.iter()
            .map(|info: &Anomaly| (info.span_length as f32 - mean_span_length).powi(2))
            .sum::<f32>() / self.anomalies.len() as f32;
    
        // Standard deviation is the square root of variance
        let std_dev_span_length: f32 = variance.sqrt();
    
        // Update Z-scores for both clusters and gaps based on their deviation from mean metrics.
        for info in self.anomalies.iter_mut() {
            if info.num_elements > 0 {
                // Calculate and update Z-score for clusters based on density deviation.
                let cluster_density: f32 = info.num_elements as f32 / info.span_length as f32;
                info.z_score = Some((cluster_density - mean_density) / std_dev_density);
            } else {
                // Calculate and update Z-score for gaps based on span length deviation.
                info.z_score = Some((info.span_length as f32 / std_dev_span_length) * -1.0);
            }
        }
    
        serde_json::to_string_pretty(&self.anomalies).unwrap_or_else(|_| "Failed to serialize data".to_string())
    }
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
    let dataset: Vec<i32> = if atty::is(atty::Stream::Stdin) {
        if args.len() != 4 {
            eprintln!("Usage: {} <filename> <factor> <min_cluster_size>", args[0]);
            process::exit(1);
        }
        let filename = &args[1];
        let file = File::open(filename)?;
        BufReader::new(file).lines().filter_map(Result::ok)
            .filter_map(|line| line.trim().parse::<i32>().ok()) // Directly parse to i32
            .collect()
    } else {
        stdin().lock().lines().filter_map(Result::ok)
            .filter_map(|line| line.trim().parse::<i32>().ok()) // Directly parse to i32
            .collect()
    };

    let factor: f32 = args[args.len() - 2].parse().expect("Factor must be a float");
    let min_cluster_size: usize = args[args.len() - 1].parse().expect("Min cluster size must be an integer");

    // Analysis and output
    let mut zhaba = Lyagushka::new(dataset);
    println!("{}", zhaba.search(factor, min_cluster_size));

    Ok(())
}