import argparse
from os import path
import time
from typing import Any, Dict, Tuple
from common import *
import pandas as pd
import json

NUM_RUNS = 1

CHECKPOINT_DIR = "checkpoint"

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

    command = program.get_wanco_cmd()
    exe_name = command[0].split("/")[-1]

    checkpoint_time_ms: float = 0
    restore_time_ms: float = 0
    snapshot_size: int = 0

    # --- start the execution ---

    snapshot_dir_path = CHECKPOINT_DIR
    if os.path.exists(snapshot_dir_path):
        os.system(f"rm -rf {snapshot_dir_path}")
    os.makedirs(snapshot_dir_path)

    proc = subprocess.Popen(
        command,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        cwd=program.workdir,
    )
    time.sleep(half_elapsed_time_ms / 1000)
    # get the pid of the process
    pids = get_pid_by_name(exe_name)
    if len(pids) == 0:
        raise Exception(f"Error: process {exe_name} not found")
    dump_proc = subprocess.run(
        [
            "criu",
            "dump",
            "--shell-job",
            "-t",
            str(pids[0]),
            "--file-locks",
            "-D",
            CHECKPOINT_DIR,
        ],
    )

    # wait for the process to finish
    stat = proc.wait()
    if stat != 0:
        # It is ok because the process is killed
        pass
    if dump_proc.returncode != 0:
        raise Exception("Error: criu dump failed")

    # --- Get the checkpoint status ---

    subproc_decode = subprocess.run(
        ["crit", "decode", "-i", path.join(snapshot_dir_path, "stats-dump")],
        capture_output=True,
        text=True,
    )
    if subproc_decode.returncode != 0:
        raise Exception("Error: criu decode failed")

    criu_stat_json: Any = json.loads(subproc_decode.stdout)
    # CRIU outputs us.
    # https://github.com/checkpoint-restore/criu/issues/702
    freezing_time_ms = int(criu_stat_json["entries"][0]["dump"]["freezing_time"]) / 1000
    memdump_time_ms = int(criu_stat_json["entries"][0]["dump"]["memdump_time"]) / 1000
    memwrite_time_ms = int(criu_stat_json["entries"][0]["dump"]["memwrite_time"]) / 1000
    checkpoint_time_ms = freezing_time_ms + memdump_time_ms + memwrite_time_ms

    # get the snpashot folder size
    snapshot_size = get_dir_size(snapshot_dir_path)

    # --- Restore the process ---

    subproc_restore = subprocess.run(
        ["criu", "restore", "--shell-job", "-D", snapshot_dir_path],
    )
    if subproc_restore.returncode != 0:
        raise Exception("Error: criu restore failed")

    # restore_time=$(crit decode -i checkpoint/stats-restore | jq '.entries[0].restore.restore_time')
    stats_restore = subprocess.run(
        ["crit", "decode", "-i", path.join(snapshot_dir_path, "stats-restore")],
        capture_output=True,
        text=True,
    )
    if stats_restore.returncode != 0:
        raise Exception("Error: criu decode failed to get stas-restore")

    criu_restore_stat_json: Any = json.loads(stats_restore.stdout)
    restore_time_ms = (
        int(criu_restore_stat_json["entries"][0]["restore"]["restore_time"]) / 1000
    )

    return checkpoint_time_ms, restore_time_ms, snapshot_size


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


def measure(programs: list[Program], args: Any) -> None:
    # Load overhead.json
    overhead_json: Any = None
    with open(args.filename, "r") as f:
        overhead_json = json.load(f)

    for program in programs:
        print(f"{program.name}")
        elapsed_time_sec = get_elapsed_time_sec(program.name, overhead_json, cr=True)
        measure_checkpoint_time(program, elapsed_time_sec)


def main():
    if not check_preconditions():
        exit(1)

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "-o",
        "--output",
        help="Save CSV to the given filename.",
        default="chkpt-restore-criu.csv",
    )
    parser.add_argument("filename", help="overhead.json")
    args = parser.parse_args()

    measure(programs, args)
    save_csv_file(args.output)


if __name__ == "__main__":
    main()
