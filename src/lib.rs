use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::wrap_pyfunction;
use serde::Serialize;
use serde_json::to_string_pretty;

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
    // Extract integers from a Python list and create a vector of Point structs.
    let dataset: Vec<Point> = int_list.extract::<Vec<u32>>()?
                                      .into_iter()
                                      .map(Point::new)
                                      .collect();

    // Calculate clusters and gaps from the dataset using predefined criteria.
    let mut cluster_gap_infos = calculate_densities_and_gaps(&dataset, factor, min_cluster_size);

    // Calculate the mean density of clusters in the dataset for comparison.
    let mean_density: f32 = cluster_gap_infos.iter()
                                             .filter(|info| info.num_elements > 0)
                                             .map(|info| info.num_elements as f32 / info.span_length)
                                             .sum::<f32>() / cluster_gap_infos.iter().filter(|info| info.num_elements > 0).count() as f32;

    // Calculate the standard deviation of cluster densities to evaluate variation.
    let variance_density: f32 = cluster_gap_infos.iter()
                                                 .filter(|info| info.num_elements > 0)
                                                 .map(|info| info.num_elements as f32 / info.span_length)
                                                 .map(|density| (density - mean_density).powi(2))
                                                 .sum::<f32>() / cluster_gap_infos.iter().filter(|info| info.num_elements > 0).count() as f32;
    let std_dev_density = variance_density.sqrt();

    // Calculate the average span of all clusters and gaps to assess gap significance.
    let average_span: f32 = cluster_gap_infos.iter().map(|info| info.span_length).sum::<f32>() / cluster_gap_infos.len() as f32;

    // Update Z-scores for both clusters and gaps based on their deviation from mean metrics.
    for info in &mut cluster_gap_infos {
        if info.num_elements > 0 {
            // Calculate and update Z-score for clusters based on density deviation.
            let cluster_density = info.num_elements as f32 / info.span_length;
            info.z_score = Some((cluster_density - mean_density) / std_dev_density);
        } else {
            // Calculate and update Z-score for gaps based on span length deviation.
            info.z_score = Some((info.span_length - average_span) / std_dev_density);
        }
    }

    // Serialize the updated cluster and gap information, including Z-scores, to a JSON string.
    to_string_pretty(&cluster_gap_infos)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("JSON Serialization Error: {}", e)))
}

#[pymodule]
fn pyagushka(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(lyagushka, m)?)?;
    Ok(())
}
