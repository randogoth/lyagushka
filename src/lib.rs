use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::wrap_pyfunction;
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

/// A Python-exposed function that analyzes a list of numerical values to identify clusters and significant gaps,
/// calculates z-scores for each identified cluster/gap, and returns the analysis results as a JSON string.
///
/// This function takes a list of integers (representing a dataset), a factor to adjust clustering and gap detection thresholds,
/// and a minimum cluster size. It calculates the mean distance and standard deviation across the dataset,
/// identifies clusters and significant gaps based on these metrics, calculates z-scores for each cluster/gap,
/// and returns a JSON string representing the analysis results.
///
/// # Arguments
/// * `_py`: The Python interpreter, used for Python-Rust interactions. Not directly used in the function body.
/// * `int_list`: A Python list of integers representing the dataset to be analyzed.
/// * `factor`: A floating-point value used to adjust the sensitivity of cluster and gap detection.
///   Lower values result in tighter clustering and wider gaps, while higher values do the opposite.
/// * `min_cluster_size`: The minimum number of contiguous points required to be considered a cluster.
///
/// # Returns
/// A `PyResult<String>` which is either:
/// * Ok containing a JSON-formatted string of the analysis results, including clusters and gaps with their z-scores.
/// * Err containing a Python exception if an error occurs during processing or JSON serialization.
///
#[pyfunction]
fn lyagushka(_py: Python, int_list: &PyList, factor: f32, min_cluster_size: usize) -> PyResult<String> {
    
    // Convert the Python list of integers into a Rust Vec of Point structs.
    let dataset: Vec<Point> = int_list.into_iter()
                                      .map(|py_any| py_any.extract::<u32>().map(Point::new))
                                      .collect::<PyResult<Vec<Point>>>()?;

    // Analyze the dataset to identify clusters and significant gaps.
    let mut cluster_gap_infos = calculate_densities_and_gaps(&dataset, factor, min_cluster_size);

    // Calculate the mean distance between consecutive points in the dataset.
    let mean_distance: f32 = dataset.windows(2)
                                    .map(|w| w[1].value as f32 - w[0].value as f32)
                                    .sum::<f32>() / (dataset.len() - 1) as f32;

    // Calculate the standard deviation of distances between consecutive points.
    let std_deviation: f32 = (dataset.windows(2)
                                      .map(|w| w[1].value as f32 - w[0].value as f32 - mean_distance)
                                      .map(|d| d * d)
                                      .sum::<f32>() / (dataset.len() - 1) as f32)
                                      .sqrt();

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

    // Serialize the analysis results into a JSON string and return it.
    serde_json::to_string_pretty(&cluster_gap_infos)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("JSON Serialization Error: {}", e)))
}


#[pymodule]
fn lyagushka_module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(lyagushka, m)?)?;
    Ok(())
}
