# lyagushka

(Russian лягушка [lʲɪˈɡuʂkə]: frog)

Lyagushka is a Rust command-line tool inspired by Fatum Project's 'Zhaba' algorithm (Russian 'жаба': toad) that analyzes a one-dimensional dataset of integers to identify clusters of closely grouped "attractor" points and significant "void" gaps between these clusters. It calculates z-scores for each cluster or gap to measure their statistical significance relative to the dataset's mean density and distance between points. The analysis results, including attractors, voids, and their z-scores, are output as a JSON string.

## Building

```sh
$ cargo build
```

## Usage

### Parameters

*  `filename.txt` (optional): A file containing a newline-separated list of integers to analyze. If not provided, the program expects input from stdin.
*  `factor`: A floating-point value by which the mean density/span is multiplied to make up a threshold for attractor and void detection.
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

### From a File

To analyze a dataset from a file, provide the filename as an argument, followed by the factor and minimum cluster size parameters
```sh
lyagushka filename.txt 1.5 6
```
(= '*Attractor clusters need to have at least 6 numbers with 1.5 times the mean density, void gaps need to be at leat 1.5 times the mean gap size wide*')

### From Stdin

Alternatively, you can pipe a list of integers into the tool, followed by the factor and minimum cluster size.

```sh
echo "1\n2\n10\n20" | lyagushka 0.5 2
```

