use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Clone, Debug)]
struct Point {
    value: u32,
    cluster_id: Option<usize>,
    z_score: Option<f32>, // Added field for Z-score
}

impl Point {
    fn new(value: u32) -> Self {
        Point {
            value,
            cluster_id: None,
            z_score: None, // Initialize Z-score as None
        }
    }
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

fn calculate_mean_distance(dataset: &[Point]) -> f32 {
    if dataset.len() < 2 { return 0.0; }
    let total_distance: f32 = dataset.windows(2)
        .map(|w| distance(&w[0], &w[1]) as f32)
        .sum();
    total_distance / (dataset.len() - 1) as f32
}

fn distance(p1: &Point, p2: &Point) -> u32 {
    if p1.value > p2.value {
        p1.value.wrapping_sub(p2.value)
    } else {
        p2.value.wrapping_sub(p1.value)
    }
}

fn mean(values: &[f32]) -> f32 {
    values.iter().sum::<f32>() / values.len() as f32
}

fn std_dev(values: &[f32], mean: f32) -> f32 {
    let variance = values.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32;
    variance.sqrt()
}

fn mark_repellers(dataset: &mut [Point], mean_distance: f32, std_dev_distance: f32) {
    let dataset_len = dataset.len();
    for i in 0..dataset_len {
        // Temporarily take out the point to avoid borrowing issues
        let point = std::mem::replace(&mut dataset[i], Point::new(0));
        let distances: Vec<f32> = (0..dataset_len)
            .filter(|&j| j != i) // Ensure we're not comparing the point to itself
            .map(|j| distance(&point, &dataset[j]) as f32)
            .collect();
        // Put the point back
        dataset[i] = point;

        if let Some(&min_distance) = distances.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
            let z_score = (min_distance - mean_distance) / std_dev_distance;
            if z_score.abs() > 1.0 { // Arbitrary Z-score threshold for repellers
                dataset[i].z_score = Some(-z_score);
            }
        }
    }
}


fn expand_cluster(dataset: &mut [Point], core_index: usize, cluster_id: usize, max_distance: f32) {
    let mut indices_to_visit = vec![core_index];
    while let Some(current_index) = indices_to_visit.pop() {
        if dataset[current_index].cluster_id.is_none() {
            dataset[current_index].cluster_id = Some(cluster_id);
            
            let new_neighbors: Vec<usize> = dataset.iter().enumerate()
                .filter(|&(idx, other_point)| {
                    other_point.cluster_id.is_none() &&
                    distance(&dataset[current_index], other_point) as f32 <= max_distance
                })
                .map(|(idx, _)| idx)
                .filter(|&idx| !indices_to_visit.contains(&idx))
                .collect();
            
            indices_to_visit.extend(new_neighbors);
        }
    }
}

fn dbscan(dataset: &mut [Point], min_cluster_size: usize, factor: f32) {
    let mean_distance = calculate_mean_distance(dataset);
    let distances: Vec<f32> = dataset.windows(2)
        .map(|w| distance(&w[0], &w[1]) as f32)
        .collect();
    let mean_val = mean(&distances);
    let std_dev_distance = std_dev(&distances, mean_val);
    let max_distance = mean_distance * factor;

    mark_repellers(dataset, mean_distance, std_dev_distance);

    let mut cluster_id = 0;
    for idx in 0..dataset.len() {
        if dataset[idx].cluster_id.is_none() && dataset[idx].z_score.is_none() {
            let mut neighbors = Vec::new();
            for (n_idx, other_point) in dataset.iter().enumerate() {
                if distance(&dataset[idx], other_point) as f32 <= max_distance {
                    neighbors.push(n_idx);
                }
            }

            if neighbors.len() >= min_cluster_size {
                cluster_id += 1;
                for &n_idx in &neighbors {
                    dataset[n_idx].cluster_id = Some(cluster_id);
                }
                expand_cluster(dataset, idx, cluster_id, max_distance);
            }
        }
    }
}

fn calculate_centroids_z_scores(dataset: &[Point]) -> Vec<(usize, f32)> {
    let mean_value = mean(&dataset.iter().map(|p| p.value as f32).collect::<Vec<f32>>());
    let std_dev_value = std_dev(&dataset.iter().map(|p| p.value as f32).collect::<Vec<f32>>(), mean_value);
    
    let mut centroids_z_scores: Vec<(usize, f32)> = Vec::new();

    let max_cluster_id = dataset.iter().filter_map(|p| p.cluster_id).max().unwrap_or(0);
    for cluster_id in 1..=max_cluster_id {
        let cluster_points: Vec<&Point> = dataset.iter().filter(|p| p.cluster_id == Some(cluster_id)).collect();
        if cluster_points.is_empty() {
            continue;
        }

        let centroid_value = cluster_points.iter().map(|p| p.value as f32).sum::<f32>() / cluster_points.len() as f32;
        let z_score = (centroid_value - mean_value) / std_dev_value;

        centroids_z_scores.push((cluster_id, z_score));
    }

    centroids_z_scores
}

fn main() -> io::Result<()> {
    let filename = "random_values.txt";
    let mut dataset = load_dataset(filename)?;

    let min_cluster_size = 7; // Adjust as needed
    let factor = 0.6; // Adjust as needed

    dbscan(&mut dataset, min_cluster_size, factor);

    // Calculate Z-scores for centroids of dense clusters
    let centroids_z_scores = calculate_centroids_z_scores(&dataset);

    // Iterate through the dataset to print repellers
    for point in &dataset {
        if let Some(z_score) = point.z_score {
            if z_score < 0.0 { // Assuming negative Z-scores indicate repellers
                println!("Repeller: {}, Z-Score: {}", point.value, z_score);
            }
        }
    }

    // Print information for dense cluster centroids
    for (cluster_id, z_score) in centroids_z_scores {
        let cluster_points: Vec<&Point> = dataset.iter().filter(|p| p.cluster_id == Some(cluster_id)).collect();
        if !cluster_points.is_empty() {
            // Find the value closest to the centroid
            let centroid_value = cluster_points.iter().map(|p| p.value as f32).sum::<f32>() / cluster_points.len() as f32;
            let closest_point = cluster_points.iter().min_by_key(|&&p| ((p.value as f32 - centroid_value).abs() * 1000.0) as u32).unwrap();
            println!("Attractor: {}, Z-Score: {}", closest_point.value, z_score);
        }
    }

    Ok(())
}
