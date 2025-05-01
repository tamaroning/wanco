import argparse
import json
import re
from typing import Any


def transform_path(line: str) -> str:
    match = re.search(r"[A-Za-z0-9|\.]+\.aot", line)
    if match:
        line = match.group(0)

    line = line.replace(".c.", ".")

    match = re.search(r"([^/\s]+)\.cr\.aot", line)
    if match:
        return f"{match.group(1)} w/ cr"

    match = re.search(r"([^/\s]+)\.aot", line)
    if match:
        return f"{match.group(1)}"

    return line


def process_json(json: Any):
    results = json["results"]
    # create name field from command field
    for result in results:
        result["name"] = transform_path(result["command"])
        if "run" in result["name"]:
            result["name"] = result["name"].replace("run", "llama2.c")
        print("Found", result["name"])

    # add ratios field
    last_mean = 0
    for result in results:
        if "w/ cr" not in result["name"]:
            last_mean = result["mean"]

        ratios = []
        for time in result["times"]:
            ratios.append(time / last_mean)

        result["ratios"] = ratios


def main():
    parser = argparse.ArgumentParser(description="Convert paths in a file.")
    parser.add_argument("input_file", help="Path to the input file")
    parser.add_argument("--output", required=True, help="Path to the output file")
    args = parser.parse_args()

    # read json
    with open(args.input_file, encoding="utf-8") as f:
        obj = json.load(f)
        process_json(obj)

        # write json
        with open(args.output, "w", encoding="utf-8") as f:
            json.dump(obj, f)


if __name__ == "__main__":
    main()
