#!/bin/python3
import os
import subprocess
from dataclasses import dataclass
import argparse
from typing import Any

NUM_RUNS = 30


def check_installed(cmd: str) -> bool:
    code = subprocess.run(
        [cmd, "--version"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
    )
    if code.returncode != 0:
        print(f"Error: {cmd} is not installed")
        return False
    else:
        return True


def check_preconditions() -> bool:
    hyperfine_found = check_installed("hyperfine")
    if not hyperfine_found:
        return False
    return True


def get_bench_dir() -> str:
    cwd = os.getcwd()
    return cwd


@dataclass
class Command:
    wanco: list[str]
    wanco_cr: list[str]
    native: list[str]


@dataclass
class Program:
    name: str
    command: Command
    args: list[str]
    workdir: str

    def get_wanco_cmd(self) -> list[str]:
        return self.command.wanco + self.args

    def get_wanco_cr_cmd(self) -> list[str]:
        return self.command.wanco_cr + self.args

    def get_native_cmd(self) -> list[str]:
        return self.command.native + self.args


programs = [
    Program(
        "llama2.c",
        command=Command(
            wanco=["../wanco-artifacts/run.c.aot", "--"],
            wanco_cr=["../wanco-artifacts/run.c.cr.aot", "--"],
            native=["./llama2.c.exe"],
        ),
        args=["model.bin", "-n", "256", "-i", "'Once upon a time'"],
        workdir=os.path.join(get_bench_dir(), "llama2-c"),
    ),
    Program(
        name="nbody",
        command=Command(
            wanco=["./wanco-artifacts/nbody.c.aot", "--"],
            wanco_cr=["./wanco-artifacts/nbody.c.cr.aot", "--"],
            native=["./computer-lab-benchmark/nbody.c.exe"],
        ),
        args=["1000000"],
        workdir=get_bench_dir(),
    ),
    Program(
        name="binary-trees",
        command=Command(
            wanco=["./wanco-artifacts/binary-trees.c.aot", "--"],
            wanco_cr=["./wanco-artifacts/binary-trees.c.cr.aot", "--"],
            native=["./computer-lab-benchmark/binary-trees.c.exe"],
        ),
        args=["18"],
        workdir=get_bench_dir(),
    ),
    Program(
        name="fannkuch-redux",
        command=Command(
            wanco=["./wanco-artifacts/fannkuch-redux.c.aot", "--"],
            wanco_cr=["./wanco-artifacts/fannkuch-redux.c.cr.aot", "--"],
            native=["./computer-lab-benchmark/fannkuch-redux.c.exe"],
        ),
        args=["11"],
        workdir=get_bench_dir(),
    ),
    Program(
        name="mandelbrot",
        command=Command(
            wanco=["./wanco-artifacts/mandelbrot.c.aot", "--"],
            wanco_cr=["./wanco-artifacts/mandelbrot.c.cr.aot", "--"],
            native=["./computer-lab-benchmark/mandelbrot.c.exe"],
        ),
        args=["1000"],
        workdir=get_bench_dir(),
    ),
    Program(
        name="nop",
        command=Command(
            wanco=["./wanco-artifacts/nop.c.aot", "--"],
            wanco_cr=["./wanco-artifacts/nop.c.cr.aot", "--"],
            native=["./computer-lab-benchmark/nop.c.exe"],
        ),
        args=[],
        workdir=get_bench_dir(),
    ),
]


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
