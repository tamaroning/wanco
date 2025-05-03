import argparse
import time
from typing import Any, Dict, Tuple
from common import *
import pandas as pd
import json

NUM_RUNS = 10

data: Dict[str, list[float | int | str]] = {
    # program name
    "program": [],
    # checkpoint time in microseconds
    "checkpoint_time": [],
    # restore time in microseconds
    "restore_time": [],
    # snapshot file size in bytes
    "snapshot_size": [],
}


def add_row_to_csv(
    program: str,
    chkpt_time: float,
    restore_time: float,
    snapshot_size: int,
) -> None:
    data["program"].append(program)
    data["checkpoint_time"].append(chkpt_time)
    data["restore_time"].append(restore_time)
    data["snapshot_size"].append(snapshot_size)


def save_csv_file(filename: str) -> None:
    df = pd.DataFrame(data)
    df.to_csv(filename, index=False)


def measure_once(
    program: Program, half_elapsed_time_ms: float
) -> Tuple[float, float, int]:
    """
    Measure the checkpoint time, restore time, and snoshot of a program once.
    Raise an exception if checkpoint or restore does not succeed.
    """

    command = program.get_wanco_cr_cmd()
    exe_name = command[0].split("/")[-1]

    chkpt_time: float = 0
    restore_time: float = 0
    file_size: int = 0

    if os.path.exists("checkpoint.pb"):
        os.remove("checkpoint.pb")
    if os.path.exists("restore-time.txt"):
        os.remove("restore-time.txt")
    if os.path.exists("chkpt-time.txt"):
        os.remove("chkpt-time.txt")

    proc = subprocess.Popen(
        command,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        cwd=program.workdir,
    )
    time.sleep(half_elapsed_time_ms / 1000)
    subprocess.run(["pkill", "-10", "-f", exe_name], cwd=program.workdir)

    # wait for the process to finish
    stat = proc.wait()
    if stat != 0:
        raise Exception("Error: process failed")

    chkpt_time_path = os.path.join(program.workdir, "chkpt-time.txt")
    wait_for_file_creation(chkpt_time_path)
    try:
        f = open(chkpt_time_path, "r")
        chkpt_time = float(f.read().strip())
    except FileNotFoundError:
        raise Exception("Error: chkpt-time.txt not found")

    snapshot_path = os.path.join(program.workdir, "checkpoint.pb")
    wait_for_file_creation(snapshot_path)
    if os.path.exists(snapshot_path):
        file_size = os.path.getsize(snapshot_path)

    proc2 = subprocess.Popen(
        [command[0], "--restore", "checkpoint.pb"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        cwd=program.workdir,
    )
    stat2 = proc2.wait(timeout=5)
    if stat2 != 0:
        raise Exception("Error: restore failed")

    restore_time_path = os.path.join(program.workdir, "restore-time.txt")
    wait_for_file_creation(restore_time_path)
    try:
        f = open(restore_time_path, "r")
        restore_time = float(f.read().strip())
    except FileNotFoundError:
        raise Exception("Error: restore-time.txt not found")

    return chkpt_time, restore_time, file_size


def measure_checkpoint_time(program: Program, elapsed_time_sec: float) -> None:
    half_elapsed_time_ms: float = elapsed_time_sec * 1000 / 2

    print(f"Program: {program.name}")
    print(f"\tHalf elapsed time: {half_elapsed_time_ms} ms")

    print("\t", end="")
    rest = NUM_RUNS
    while rest > 0:
        try:
            chkpt_time, restore_time, file_size = measure_once(
                program, half_elapsed_time_ms
            )
            add_row_to_csv(program.name, chkpt_time, restore_time, file_size)
        except Exception as e:
            print(f"\tError: {e}, retrying...")
            continue

        print(".", end="", flush=True)
        rest -= 1

    print("done")


def get_elapsed_time_sec(name: str, overhead_json: Any) -> float:
    name = name + " w/ cr"
    results = overhead_json["results"]
    for result in results:
        if result["name"] == name:
            elapsed_time_sec = result["median"]
            print(f"\tElapsed time: {elapsed_time_sec} s")
            return elapsed_time_sec

    raise Exception(f"Error: {name} not found in overhead.json")


def measure(programs: list[Program], args: Any) -> None:
    # Load overhead.json
    overhead_json: Any = None
    with open(args.filename, "r") as f:
        overhead_json = json.load(f)

    for program in programs:
        print(f"{program.name}")
        elapsed_time_sec = get_elapsed_time_sec(program.name, overhead_json)
        measure_checkpoint_time(program, elapsed_time_sec)


def main():
    if not check_preconditions():
        exit(1)

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "-o", "--output", help="Save CSV to the given filename.", default="chkpt-restore.csv"
    )
    parser.add_argument("filename", help="overhead.json")
    args = parser.parse_args()

    measure(programs, args)
    save_csv_file(args.output)


if __name__ == "__main__":
    main()
