#!/bin/python3
import os
import subprocess
from dataclasses import dataclass
import argparse
from typing import Any
from common import *

NUM_RUNS = 30

def measure(programs: list[Program], args: Any) -> None:
    hyperfine_cmd: list[str] = [
        "hyperfine",
        "--export-csv",
        "result.csv",
        "--export-json",
        args.output,
        # "--show-output",
        "--warmup",
        "1",
        "--runs",
        f"{NUM_RUNS}",
    ]

    for program in programs:
        cmd: list[str] = ["cd", program.workdir, ";"]
        cmd.extend(program.get_wanco_cmd())
        hyperfine_cmd.append(" ".join(cmd))

        cmd2: list[str] = ["cd", program.workdir, ";"]
        cmd2.extend(program.get_wanco_cr_cmd())
        hyperfine_cmd.append(" ".join(cmd2))

    stat = subprocess.Popen(hyperfine_cmd, cwd=get_bench_dir())
    stat.wait()
    if stat.returncode != 0:
        print("Error: hyperfine failed")
        exit(1)


def main():
    if not check_preconditions():
        exit(1)

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "-o", "--output", help="Save JSON to the given filename.", default="result.json"
    )
    args = parser.parse_args()

    measure(programs, args)


if __name__ == "__main__":
    main()
