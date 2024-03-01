# pyagushka - a Python module for lyagushka

(Russian лягушка [lʲɪˈɡuʂkə]: frog)

Pyagushka is a Python module based on the Rust algorithm lyagushka that is inspired by Fatum Project's ['Zhaba' algorithm](https://gist.github.com/randogoth/ab5ab9e8665303be176f16241e7b26b5) (Russian 'жаба': toad) and expands upon it for more versatility.

It is an algorithm that analyzes a one-dimensional dataset of integers to identify clusters of closely grouped "attractor" points and significant "void" gaps between these clusters. It calculates z-scores for each cluster or gap to measure their statistical significance relative to the dataset's mean density and distance between points. The analysis results, including attractors, voids, and their z-scores, are output as a JSON string.

## Building

With a Rust/Cargo and Python3/Pip environment set up, run:

```sh
$ pip install maturin
$ maturin build --release
$ pip install target/wheels/pyagushka-1.0.0-*.whl
```

## Usage

### Parameters

*  `dataset`: list of integers representing the dataset to be analyzed.
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

### Example

To analyze a dataset from a file, provide the filename as an argument, followed by the factor and minimum cluster size parameters
```Python
from pyagushka import lyagushka

dataset = []
with open('random_values.txt', 'r') as file:
    for line in file:
        random_data.append(int(line.strip()))

analysis_results = json.loads(lyagushka(dataset, 4.0, 7))

print(analysis_result)
```
(= '*Attractor clusters need to have at least 7 numbers with 4.0 times the mean density, void gaps need to be at leat 4.0 times the mean gap size wide*')

## CLI

If you need lyagushka as a command line tool, check out the 'main' branch of this repository