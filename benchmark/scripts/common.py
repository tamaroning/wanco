from dataclasses import dataclass
import subprocess
import os
import time
from typing import Any


def check_installed(cmd: str) -> bool:
    code = subprocess.run(
        [cmd, "--version"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
    )
    if code.returncode != 0:
        print(f"Error: {cmd} is not installed")
        return False
    else:
        return True


def get_bench_dir() -> str:
    cwd = os.getcwd()
    return cwd


def check_preconditions() -> bool:
    hyperfine_found = check_installed("hyperfine")
    if not hyperfine_found:
        return False
    return True


@dataclass
class Command:
    wanco: list[str]
    wanco_cr: list[str]
    native: list[str]
    wasmedge: list[str]
    wamr: list[str]


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

    def get_wasmedge_cmd(self) -> list[str]:
        return ["wasmedge", "run", "--dir=/:."] + self.command.wasmedge + self.args

    def get_wamr_cmd(self) -> list[str]:
        return ["iwasm", "--map-dir=/::."] + self.command.wamr + self.args


def wait_for_file_creation(file_path: str) -> None:
    for i in range(10):
        if os.path.exists(file_path):
            return
        time.sleep(0.1)


programs = [
    Program(
        "llama2.c",
        command=Command(
            wanco=["../wanco-artifacts/run.c.aot", "--"],
            wanco_cr=["../wanco-artifacts/run.c.cr.aot", "--"],
            native=["./llama2.c.exe"],
            wasmedge=["../wasmedge-artifacts/run.aot"],
            wamr=["../wamrc-artifacts/run.aot"],
        ),
        args=["model.bin", "-n", "256", "-s", "42"],
        workdir=os.path.join(get_bench_dir(), "llama2-c"),
    ),
    Program(
        name="nbody",
        command=Command(
            wanco=["./wanco-artifacts/nbody.c.aot", "--"],
            wanco_cr=["./wanco-artifacts/nbody.c.cr.aot", "--"],
            native=["./computer-lab-benchmark/nbody.c.exe"],
            wasmedge=["./wasmedge-artifacts/nbody.c.aot"],
            wamr=["./wamrc-artifacts/nbody.c.aot"],
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
            wasmedge=["./wasmedge-artifacts/binary-trees.c.aot"],
            wamr=["./wamrc-artifacts/binary-trees.c.aot"],
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
            wasmedge=["./wasmedge-artifacts/fannkuch-redux.c.aot"],
            wamr=["./wamrc-artifacts/fannkuch-redux.c.aot"],
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
            wasmedge=["./wasmedge-artifacts/mandelbrot.c.aot"],
            wamr=["./wamrc-artifacts/mandelbrot.c.aot"],
        ),
        args=["1000"],
        workdir=get_bench_dir(),
    ),
    # Program(
    #    name="nop",
    #    command=Command(
    #        wanco=["./wanco-artifacts/nop.c.aot", "--"],
    #        wanco_cr=["./wanco-artifacts/nop.c.cr.aot", "--"],
    #        native=["./computer-lab-benchmark/nop.c.exe"],
    #    ),
    #    args=[],
    #    workdir=get_bench_dir(),
    # ),
    Program(
        name="bc",
        command=Command(
            wanco=["./wanco-artifacts/bc.aot", "--"],
            wanco_cr=["./wanco-artifacts/bc.cr.aot", "--"],
            native=["./gapbs/br.exe"],
            wasmedge=["./wasmedge-artifacts/bc.aot"],
            wamr=["./wamrc-artifacts/bc.aot"],
        ),
        args=["-g", "18", "-n", "1"],
        workdir=get_bench_dir(),
    ),
    Program(
        name="bfs",
        command=Command(
            wanco=["./wanco-artifacts/bfs.aot", "--"],
            wanco_cr=["./wanco-artifacts/bfs.cr.aot", "--"],
            native=["./gapbs/bfs.exe"],
            wasmedge=["./wasmedge-artifacts/bfs.aot"],
            wamr=["./wamrc-artifacts/bfs.aot"],
        ),
        args=["-g", "18", "-n", "1"],
        workdir=get_bench_dir(),
    ),
    Program(
        name="cc",
        command=Command(
            wanco=["./wanco-artifacts/cc.aot", "--"],
            wanco_cr=["./wanco-artifacts/cc.cr.aot", "--"],
            native=["./gapbs/cc.exe"],
            wasmedge=["./wasmedge-artifacts/cc.aot"],
            wamr=["./wamrc-artifacts/cc.aot"],
        ),
        args=["-g", "18", "-n", "1"],
        workdir=get_bench_dir(),
    ),
    Program(
        name="cc_sv",
        command=Command(
            wanco=["./wanco-artifacts/cc_sv.aot", "--"],
            wanco_cr=["./wanco-artifacts/cc_sv.cr.aot", "--"],
            native=["./gapbs/cc_sv.exe"],
            wasmedge=["./wasmedge-artifacts/cc_sv.aot"],
            wamr=["./wamrc-artifacts/cc_sv.aot"],
        ),
        args=["-g", "18", "-n", "1"],
        workdir=get_bench_dir(),
    ),
    Program(
        name="pr",
        command=Command(
            wanco=["./wanco-artifacts/pr.aot", "--"],
            wanco_cr=["./wanco-artifacts/pr.cr.aot", "--"],
            native=["./gapbs/pr.exe"],
            wasmedge=["./wasmedge-artifacts/pr.aot"],
            wamr=["./wamrc-artifacts/pr.aot"],
        ),
        args=["-g", "18", "-n", "1"],
        workdir=get_bench_dir(),
    ),
    Program(
        name="pr_spmv",
        command=Command(
            wanco=["./wanco-artifacts/pr_spmv.aot", "--"],
            wanco_cr=["./wanco-artifacts/pr_spmv.cr.aot", "--"],
            native=["./gapbs/pr_spmv.exe"],
            wasmedge=["./wasmedge-artifacts/pr_spmv.aot"],
            wamr=["./wamrc-artifacts/pr_spmv.aot"],
        ),
        args=["-g", "18", "-n", "1"],
        workdir=get_bench_dir(),
    ),
    Program(
        name="sssp",
        command=Command(
            wanco=["./wanco-artifacts/sssp.aot", "--"],
            wanco_cr=["./wanco-artifacts/sssp.cr.aot", "--"],
            native=["./gapbs/sssp.exe"],
            wasmedge=["./wasmedge-artifacts/sssp.aot"],
            wamr=["./wamrc-artifacts/sssp.aot"],
        ),
        args=["-g", "18", "-n", "1"],
        workdir=get_bench_dir(),
    ),
    # Program(
    #    name="tc",
    #    command=Command(
    #        wanco=["./wanco-artifacts/tc.aot", "--"],
    #        wanco_cr=["./wanco-artifacts/tc.cr.aot", "--"],
    #        native=["./gapbs/tc.exe"],
    #    ),
    #    args=["-g", "18", "-n", "1"],
    #    workdir=get_bench_dir(),
    # ),
]


def get_elapsed_time_sec(name: str, overhead_json: Any, cr=False) -> float:
    """
    Get the elapsed time in seconds for a given program name from the overhead.json file.
    """
    runtime = "wanco"
    if cr:
        runtime = "wanco-cr"

    results = overhead_json["results"]
    for result in results:
        if result["name"] == name and result["runtime"] == runtime:
            elapsed_time_sec = result["median"]
            print(f"\tElapsed time: {elapsed_time_sec} s")
            return elapsed_time_sec

    raise Exception(f"Error: {name} not found in overhead.json")


def get_pid_by_name(name):
    try:
        result = subprocess.check_output(["pgrep", "-f", name])
        pids = result.decode().strip().split("\n")
        return [int(pid) for pid in pids]
    except subprocess.CalledProcessError:
        return []


def get_dir_size(path: str) -> int:
    total_size = 0
    for dirpath, dirnames, filenames in os.walk(path):
        for f in filenames:
            fp = os.path.join(dirpath, f)
            # skip if it is symbolic link
            if not os.path.islink(fp):
                total_size += os.path.getsize(fp)

    return total_size
