import argparse
import json
import re
from typing import Any


def transform_path(line: str) -> str:
    match = re.search(r"([^/\s]+)\.c\.aot", line)
    if match:
        return f"{match.group(1)}"

    match2 = re.search(r"([^/\s]+)\.c\.cr\.aot", line)
    if match2:
        return f"{match2.group(1)} w/ cr"

    return line  # 変換できない場合はそのまま


def process_json(json: Any):
    results = json["results"]
    # rewrite command field
    for result in results:
        result["command"] = transform_path(result["command"])

    last_mean = 0
    for result in results:
        if "w/ cr" not in result["command"]:
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
