# lyagushka

(Russian лягушка [lʲɪˈɡuʂkə]: frog)

Lyagushka is a Rust command-line tool inspired by Fatum Project's ['Zhaba' algorithm](https://gist.github.com/randogoth/ab5ab9e8665303be176f16241e7b26b5) (Russian 'жаба': toad) and expands upon it for more versatility.

It is an algorithm that analyzes a one-dimensional dataset of integers to identify clusters of closely grouped "attractor" points and significant "void" gaps between these clusters. It calculates z-scores for each cluster or gap to measure their statistical significance relative to the dataset's mean density and distance between points. The analysis results, including attractors, voids, and their z-scores, are output as a JSON string.

## Building

With a Rust and Cargo environment set up, simply run:

```sh
$ cargo build --release
```

## Usage

### Parameters

*  `filename.txt` (optional): A file containing a newline-separated list of integers to analyze. If not provided, the program expects input from stdin.
*  `factor`: A floating-point value by which the mean density/span is multiplied to make up a threshold for attractor and void detection.
*  `min_cluster_size`: An integer specifying the minimum number of contiguous points required to be considered a cluster.

### Output

The tool outputs a JSON string that includes details about the identified attractors and voids, along with their respective z-scores. Here's an example of the JSON output format:

```json

[
  //...
  {
    "elements": [ 722, 722, 722, 725, 725, 726, 726, 726],
    "start": 722,
    "end": 726,
    "span_length": 4,
    "num_elements": 8,
    "centroid": 724.0,
    "z_score": 1.19528
  },
  {
    "elements": [],
    "start": 732,
    "end": 740,
    "span_length": 8,
    "num_elements": 0,
    "centroid": 736.0,
    "z_score": -1.13359
  },
  //...
]
```

### From a File

To analyze a dataset from a file, provide the filename as an argument, followed by the factor and minimum cluster size parameters
```sh
lyagushka random_values.txt 1.5 6
```
(= '*Attractor clusters need to have at least 6 numbers with 1.5 times the mean density, void gaps need to be at leat 1.5 times the mean gap size wide*')

### From Stdin

Alternatively, you can pipe a list of integers into the tool, followed by the factor and minimum cluster size.

```sh
cat random_values.txt | lyagushka 0.5 2
```

## Python Module

If you need lyagushka in a Python environment, check out the 'Python' branch of this repository