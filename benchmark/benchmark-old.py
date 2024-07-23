#!/usr/bin/env python3
import matplotlib as mpl
import matplotlib.pyplot as plt

data = [
    {
        "name": "none",
        "data": [
            [30, 0.008],
            [40, 1.778],
            [42, 4.689],
            [45, 19.792]
        ]
    },
    {
        "name": "C",
        "data": [
            [30, 0.009],
            [40, 1.946],
            [42, 5.076],
            [45, 21.460]
        ]
    },
    {
        "name": "R",
        "data": [
            [30, 0.009],
            [40, 2.054],
            [42, 5.452],
            [45, 22.861],
        ]
    },
    {
        "name": "C+R",
        "data": [
            [30, 0.016],
            [40, 2.419],
            [42, 6.294],
            [45, 26.838],
        ]
    }
]


def plot(data):
    for d in data:
        x = [i[0] for i in d["data"]]
        y = [i[1] for i in d["data"]]
        plt.plot(x, y, label=d["name"])
    plt.xlabel("N (fibonacci number)")
    plt.ylabel("CPU Time (s)")
    plt.title("Benchmark")
    plt.legend()
    plt.show()


plot(data)
