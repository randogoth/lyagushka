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

fn gap_distances(dataset: &[Point]) -> Vec<f32> {
    dataset.windows(2)
           .map(|pair| distance(&pair[0], &pair[1]) as f32)
           .collect()
}

fn mean(values: &[f32]) -> f32 {
    values.iter().sum::<f32>() / values.len() as f32
}

fn std_dev(values: &[f32], mean: f32) -> f32 {
    let variance = values.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32;
    variance.sqrt()
}

fn mark_voids(dataset: &[Point], mean_distance: f32, factor: f32, z_score_threshold: f32, std_dev_distance: f32) -> Vec<(f32, f32)> {
    let significant_gap_distance = 2.0 * factor * mean_distance;
    let mut voids = Vec::new();

    for i in 0..dataset.len() - 1 {
        let gap_distance = distance(&dataset[i], &dataset[i + 1]) as f32;
        if gap_distance >= significant_gap_distance {
            // Calculate the midpoint of the gap
            let midpoint = (dataset[i].value as f32 + dataset[i + 1].value as f32) / 2.0;

            // Calculate Z-score for the gap based on its deviation from the mean distance
            let z_score = (gap_distance - mean_distance) / std_dev_distance;

            if z_score > z_score_threshold {
                voids.push((midpoint, z_score));
            }
        }
    }

    voids
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

fn dbscan(dataset: &mut [Point], min_cluster_size: usize, factor: f32, z_score_threshold: f32) {
    let gaps = gap_distances(dataset); // Calculate gap distances
    let mean_gap_distance = mean(&gaps); // Calculate the mean of gap distances
    let std_dev_gap_distance = std_dev(&gaps, mean_gap_distance); // Calculate std dev of gap distances
    let max_distance = mean_gap_distance * (1.0 / factor);

    // Adjusted call to mark_voids to use std_dev_gap_distance
    let voids = mark_voids(dataset, mean_gap_distance, factor, z_score_threshold, std_dev_gap_distance);

    // Now you can use the voids information
    for (midpoint, z_score) in voids {
        println!("void Midpoint: {}, Z-Score: {}", midpoint, z_score);
    }

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
    let factor = 1.6; // Adjust as needed
    let z_score_threshold = 1.6; // Define your Z-score threshold here

    dbscan(&mut dataset, min_cluster_size, factor, z_score_threshold);

    // Calculate Z-scores for centroids of dense clusters
    let centroids_z_scores = calculate_centroids_z_scores(&dataset);

    // Iterate through the dataset to print voids
    for point in &dataset {
        if let Some(z_score) = point.z_score {
            if z_score < 0.0 { // Assuming negative Z-scores indicate voids
                println!("void: {}, Z-Score: {}", point.value, z_score);
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
