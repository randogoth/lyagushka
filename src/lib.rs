use pyo3::prelude::*;
use serde::Serialize;

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

#[pyclass]
struct Lyagushka {
    dataset: Vec<i32>,
    anomalies: Vec<Anomaly>,
}

#[pymethods]
impl Lyagushka {
    
    #[new]
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

#[pymodule]
fn pyagushka(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Lyagushka>()?;
    Ok(())
}
