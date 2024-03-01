from pyagushka import lyagushka
from randonautentropy import rndo
import json
import matplotlib.pyplot as plt
import numpy as np
from scipy.interpolate import interp1d

def generate_random_data(size=1024, max_value=100):

    random_data = []
    max_value_bytes = (max_value.bit_length() + 7) // 8
    max_int_for_bytes = 2**(max_value_bytes * 8) - 1
    min_bytes_needed =  max_value_bytes * size
    mod_cutoff = max_int_for_bytes - (max_int_for_bytes % max_value) - 1

    # Populate the 'random_data' array
    while len(random_data) < size:
        hex_data = rndo.get(length=min_bytes_needed)
        hex_chunks = list((hex_data[0+i:2 * max_value_bytes+i] for i in range(0, len(hex_data), 2 * max_value_bytes)))
        for i in hex_chunks:
            num = int(i, 16)
            if num <= mod_cutoff and len(random_data) < size:
                random_data.append( num % (max_value + 1) )

    return random_data

# load the random test data
dataset = []
# with open('random_values.txt', 'r') as file:
#     for line in file:
#         random_data.append(int(line.strip()))

dataset = generate_random_data(1024, 1024)
dataset.sort()

# calculate the anomalies in the data
analysis_results = json.loads(lyagushka(dataset, 3.0, 7))

# Initialize plot
plt.figure(figsize=(10, 6))

# Color palette for clusters and gaps
colors = plt.cm.jet(np.linspace(0, 1, len(analysis_results)))

# Plot dataset points and assign colors based on cluster membership
for i, result in enumerate(analysis_results):
    if result['num_elements'] > 0:  # It's a cluster
        points_in_cluster = [point for point in dataset if 
                             result['centroid'] - result['span_length'] / 2 <= point <= 
                             result['centroid'] + result['span_length'] / 2]
        for point in points_in_cluster:
            plt.plot(point, 0, 'o', color=colors[i])  # Plot points in cluster with the same color

    # Plot a line segment for the cluster/gap Z-score in the same color
    start = result['centroid'] - result['span_length'] / 2
    end = result['centroid'] + result['span_length'] / 2
    z_score = result['z_score'] if result['z_score'] is not None else 0
    plt.plot([start, end], [z_score, z_score], color=colors[i], linewidth=2)

# Enhancements for visualization
plt.xlabel('Integer Value')
plt.ylabel('Z-Score')
plt.title('Cluster and Gap Analysis')
plt.grid(True)

plt.show()