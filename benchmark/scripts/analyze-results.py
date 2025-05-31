import argparse
import json
from typing import Any


def analyze_ratio(results: list[Any], runtime: str):
    print(f"-------- {runtime} ------------")
    median_ratios = []
    for result in results:
        if runtime == result["runtime"]:
            name = result["name"]
            # calulate mean
            ratios = result["ratios"]
            median = sorted(ratios)[len(ratios) // 2]
            print(name, ": Median ratio", round(median, 3))
            median_ratios.append(median)

    print("--------------------")
    # average mean ratios, max, and min
    mean = sum(median_ratios) / len(median_ratios)
    max_ratio = max(median_ratios)
    min_ratio = min(median_ratios)
    print("Mean median ratio", round(mean, 3))
    print("Max ratio", round(max_ratio, 3))
    print("Min ratio", round(min_ratio, 3))
    print("--------------------")


def process_json(json: Any):
    results = json["results"]

    analyze_ratio(results, "wanco-cr")
    analyze_ratio(results, "wasmedge")
    analyze_ratio(results, "wamr")


def main():
    parser = argparse.ArgumentParser(description="Convert paths in a file.")
    parser.add_argument("input_file", help="Path to overhead.json")
    args = parser.parse_args()

    # read json
    with open(args.input_file, encoding="utf-8") as f:
        obj = json.load(f)
        process_json(obj)


if __name__ == "__main__":
    main()
