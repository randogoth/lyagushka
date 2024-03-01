use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::wrap_pyfunction;
use serde::Serialize;
use serde_json;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

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

fn calculate_densities_and_gaps(dataset: &[Point], factor: f32, min_cluster_size: usize) -> Vec<ClusterGapInfo> {
    if dataset.len() < 2 { return Vec::new(); }

    let mean_distance = dataset.windows(2)
                               .map(|w| (w[1].value - w[0].value) as f32)
                               .sum::<f32>() / (dataset.len() - 1) as f32;
    let cluster_threshold = mean_distance / factor;
    let gap_threshold = factor * mean_distance * 2.0;

    dataset.windows(2).fold(Vec::new(), |mut acc, window| {
        let gap_distance = (window[1].value - window[0].value) as f32;
        if gap_distance > gap_threshold && acc.last().map_or(true, |last: &ClusterGapInfo| last.num_elements >= min_cluster_size) {
            acc.push(ClusterGapInfo {
                span_length: gap_distance,
                num_elements: 0,
                centroid: (window[0].value + window[1].value) as f32 / 2.0,
                z_score: None,
            });
        }
        acc
    })
}

#[pyfunction]
fn lyagushka(_py: Python, int_list: &PyList, factor: f32, min_cluster_size: usize) -> PyResult<String> {
    let dataset: Vec<Point> = int_list.into_iter()
                                      .map(|py_any| py_any.extract::<u32>().map(Point::new))
                                      .collect::<PyResult<Vec<Point>>>()?;
    let cluster_gap_infos = calculate_densities_and_gaps(&dataset, factor, min_cluster_size);

    serde_json::to_string_pretty(&cluster_gap_infos)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("JSON Serialization Error: {}", e)))
}

#[pymodule]
fn lyagushka_module(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(lyagushka, m)?)?;
    Ok(())
}