use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::wrap_pyfunction;
use serde::Serialize;
use serde_json::to_string_pretty;

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

fn anomaly_info(cluster: &[i32]) -> Anomaly {
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
        z_score: None, // Placeholder for actual Z-score calculation
    }
}


/// Calculates the densities (clusters) and significant gaps between points in a dataset.
///
/// This function iterates over a dataset of points, identifying clusters based on a distance threshold
/// (calculated from the mean distance between points and adjusted by a given factor) and identifying significant gaps
/// that exceed a certain threshold. Each cluster or significant gap identified is summarized in a `Anomaly` object.
///
/// # Arguments
/// * `dataset`: A slice of `Point` objects representing the dataset to be analyzed.
/// * `factor`: A multiplier used to define the thresholds for clustering and gap identification. 
///   A lower factor tightens the cluster threshold and widens the gap threshold, and vice versa.
/// * `min_cluster_size`: The minimum number of points required for a group of points to be considered a cluster.
///
/// # Returns
/// A vector of `Anomaly` objects, each representing either a cluster of points or a significant gap between points.
///
fn scan_anomalies(dataset: &[i32], factor: f32, min_cluster_size: usize) -> Vec<Anomaly> {
    
    // Return early if the dataset is too small to form any clusters or gaps.
    if dataset.len() < 2 { return Vec::new(); }

    // Calculate the mean distance between consecutive points in the dataset.
    let mean_distance: f32 = dataset.windows(2)
                                    .map(|w| (w[1] - w[0]) as f32)
                                    .sum::<f32>() / (dataset.len() - 1) as f32;

    // Define thresholds for clustering and gap identification based on the mean distance and factor.
    let cluster_threshold: f32 = mean_distance / factor;
    let gap_threshold: f32 = factor * mean_distance;

    let mut results: Vec<Anomaly> = Vec::new(); // Stores the resulting clusters and gaps.
    let mut current_cluster: Vec<i32> = Vec::new(); // Temporary storage for points in the current cluster.

    // Iterate through pairs of consecutive points to find clusters and significant gaps.
    for window in dataset.windows(2) {
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
                results.push(anomaly_info(&current_cluster));
                current_cluster.clear();
            }

            // Record the gap
            if gap_size > gap_threshold {
                results.push(Anomaly {
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
        results.push(anomaly_info(&current_cluster));
    }

    results
}

/// Analyzes a dataset of integers to identify clusters and gaps, then calculates Z-scores 
/// for each based on their deviation from mean metrics. The analysis aims to highlight 
/// significant clusters of closely grouped points and notable gaps between them, providing 
/// a statistical measure of their significance through Z-scores. The results, including 
/// clusters, gaps, and their Z-scores, are serialized into a JSON string.
///
/// # Arguments
/// * `_py` - The Python interpreter instance, used for Python-Rust interoperability. 
///     This argument is necessary for functions exposed to Python via PyO3 but is not 
///     directly used within the function.
/// * `int_list` - A Python list of integers representing the dataset to be analyzed. 
///     This list is converted into a Vec<Point> for internal processing.
/// * `factor` - A floating-point value used as a threshold factor to adjust the sensitivity 
///     of cluster and gap detection. This factor influences the identification of clusters 
///     by defining the minimum density or separation required.
/// * `min_cluster_size` - An integer specifying the minimum number of contiguous points 
///     required for a group of points to be considered a cluster. This parameter helps 
///     filter out noise by defining a threshold for the minimum cluster size.
///
/// # Returns
/// Returns a `PyResult<String>` containing a JSON-formatted string of the analysis results. 
/// The JSON string includes detailed information about each identified cluster and gap, 
/// such as their span length, number of elements (if applicable), centroid, and calculated 
/// Z-score. In case of an error during processing or serialization, a Python exception is 
/// returned.
///
#[pyfunction]
fn lyagushka(_py: Python, int_list: &PyList, factor: f32, min_cluster_size: usize) -> PyResult<String> {
    // Extract integers from a Python list and create a vector.
    let mut dataset: Vec<i32> = int_list.extract::<Vec<i32>>()?;
    
    // Sort the vector
    dataset.sort_unstable();

    // Calculate clusters and gaps from the dataset using predefined criteria.
    let mut anomalies: Vec<Anomaly> = scan_anomalies(&dataset, factor, min_cluster_size);

    // Calculate the mean density of clusters in the dataset for comparison.
    let mean_density: f32 = anomalies.iter()
        .filter(|info: &&Anomaly| info.num_elements > 0)
        .map(|info: &Anomaly| info.num_elements as f32 / info.span_length as f32)
        .sum::<f32>() / anomalies.iter().filter(|info: &&Anomaly| info.num_elements > 0).count() as f32;

    // Calculate the standard deviation of cluster densities to evaluate variation.
    let variance_density: f32 = anomalies.iter()
        .filter(|info: &&Anomaly| info.num_elements > 0)
        .map(|info: &Anomaly| info.num_elements as f32 / info.span_length as f32)
        .map(|density| (density - mean_density).powi(2))
        .sum::<f32>() / anomalies.iter().filter(|info: &&Anomaly| info.num_elements > 0).count() as f32;
    let std_dev_density = variance_density.sqrt();

    // Calculate mean span length
    let mean_span_length: f32 = anomalies.iter()
        .map(|info: &Anomaly| info.span_length as f32)
        .sum::<f32>() / anomalies.len() as f32;

    // Calculate variance
    let variance: f32 = anomalies.iter()
        .map(|info: &Anomaly| (info.span_length as f32 - mean_span_length).powi(2))
        .sum::<f32>() / anomalies.len() as f32;

    // Standard deviation is the square root of variance
    let std_dev_span_length: f32 = variance.sqrt();

    // Update Z-scores for both clusters and gaps based on their deviation from mean metrics.
    for info in anomalies.iter_mut() {
        if info.num_elements > 0 {
            // Calculate and update Z-score for clusters based on density deviation.
            let cluster_density: f32 = info.num_elements as f32 / info.span_length as f32;
            info.z_score = Some((cluster_density - mean_density) / std_dev_density);
        } else {
            // Calculate and update Z-score for gaps based on span length deviation.
            info.z_score = Some((info.span_length as f32 / std_dev_span_length) * -1.0);
        }
    }

    // Serialize the updated cluster and gap information, including Z-scores, to a JSON string.
    to_string_pretty(&anomalies)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("JSON Serialization Error: {}", e)))
}


#[pymodule]
fn pyagushka(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(lyagushka, m)?)?;
    Ok(())
}
