from pyagushka import lyagushka
from randonautentropy import rndo
import json

random_data = []

with open('random_values.txt', 'r') as file:
    for line in file:
        random_data.append(int(line.strip()))

anomalies = json.loads(lyagushka(random_data, 1.0, 5))

print(anomalies)