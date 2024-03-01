# lyagushka

(Russian лягушка: frog)

Cluster and Gap Analysis Tool inspired by Fatum Project's 'Zhaba' algorithm (Russian 'жаба': toad) that finds attractor clusters in lists of integers.

This Rust command-line tool analyzes a dataset of integers to identify clusters of closely grouped points and significant gaps between these clusters. It calculates z-scores for each cluster or gap to measure their statistical significance relative to the dataset's mean distance. The analysis results, including clusters, gaps, and their z-scores, are output as a JSON string.

## Features

- **Cluster Identification**: Identifies groups of points that are closely spaced together based on a customizable threshold.
- **Gap Detection**: Detects significant gaps between clusters, providing insights into the dataset's distribution.
- **Z-Score Calculation**: Calculates z-scores for both clusters and gaps, offering a statistical measure of their deviation from the mean distance.
- **Flexible Input**: Accepts input data either from a file specified as a command-line argument or piped directly into stdin.
- **JSON Output**: Outputs the analysis results in a readable JSON format, making it easy to interpret or use in further processing.

## Usage

### From a File

To analyze a dataset from a file, provide the filename as an argument along with two additional parameters: the factor for adjusting clustering and gap detection thresholds, and the minimum cluster size.

```sh
cargo run -- filename.txt 0.5 2
```

### From Stdin

Alternatively, you can pipe a list of integers into the tool, followed by the factor and minimum cluster size.

```sh
echo "1\n2\n10\n20" | cargo run -- 0.5 2
```

#### Parameters

*  `filename.txt` (optional): A file containing a newline-separated list of integers to analyze. If not provided, the program expects input from stdin.
*  `factor`: A floating-point value used to fine-tune the sensitivity of cluster and gap detection. Lower values result in tighter clusters and wider gaps, while higher values do the opposite.
*  `min_cluster_size`: An integer specifying the minimum number of contiguous points required to be considered a cluster.

### Output

The tool outputs a JSON string that includes details about the identified clusters and gaps, along with their respective z-scores. Here's an example of the JSON output format:

```json

[
  {
    "span_length": 1.0,
    "num_elements": 2,
    "centroid": 1.5,
    "z_score": -1.23
  },
  {
    "span_length": 8.0,
    "num_elements": 0,
    "centroid": 6.0,
    "z_score": 2.45
  }
]
```