import pandas as pd
import matplotlib.pyplot as plt
import argparse


def load_and_aggregate(csv_path) -> pd.DataFrame:
    df = pd.read_csv(csv_path)
    return df.groupby("program", sort=False).median()


def plot_comparison(
    df_wasm: pd.DataFrame, df_criu: pd.DataFrame, column: str, output_file: str
) -> None:
    print("--- Plotting", column, "---")

    programs = df_wasm.index

    y_label = column.replace("_", " ").capitalize()

    values_wasm = [
        df_wasm.loc[p, column] if p in df_wasm.index else 0 for p in programs
    ]
    values_criu = [
        df_criu.loc[p, column] if p in df_criu.index else 0 for p in programs
    ]

    # print analysis result to stdout
    ratios = [
        values_criu[i] / values_wasm[i] if values_criu[i] != 0 else float("inf")
        for i in range(len(values_wasm))
    ]
    # 最大の比を与えるプログラム
    max_ratio_index = ratios.index(max(ratios))
    max_ratio_program = programs[max_ratio_index]
    max_ratio_value = ratios[max_ratio_index]
    max_ratio_wanco = values_wasm[max_ratio_index]
    max_ratio_criu = values_criu[max_ratio_index]
    print(f"Max ratio: {max_ratio_program} ({max_ratio_value:.2f})")
    print(f"\tCRIU={max_ratio_criu:.2f} => Wanco={max_ratio_wanco:.2f}")
    # 最小の比を与えるプログラム
    min_ratio_index = ratios.index(min(ratios))
    min_ratio_program = programs[min_ratio_index]
    min_ratio_value = ratios[min_ratio_index]
    min_ratio_wanco = values_wasm[min_ratio_index]
    min_ratio_criu = values_criu[min_ratio_index]
    print(f"Min ratio: {min_ratio_program} ({min_ratio_value:.2f})")
    print(f"\tCRIU={min_ratio_criu:.2f} => Wanco={min_ratio_wanco:.2f}")

    color_wasm = "blue"
    color_criu = "orange"

    x = range(len(programs))
    width = 0.35

    if "time" in y_label:
        y_label += " [ms]"
        color_wasm = "#1f77b4"
        color_criu = "#ff7f0e"
    elif "size" in y_label:
        y_label += " [MiB]"
        values_wasm = [v / 1024 / 1024 for v in values_wasm]
        values_criu = [v / 1024 / 1024 for v in values_criu]
        color_wasm = "lightseagreen"
        color_criu = "hotpink"

    plt.figure(figsize=(10, 6))
    plt.bar(
        [i - width / 2 for i in x],
        values_wasm,
        width=width,
        label="Wanco",
        color=color_wasm,
    )
    plt.bar(
        [i + width / 2 for i in x],
        values_criu,
        width=width,
        label="CRIU",
        color=color_criu,
    )
    plt.xticks(ticks=x, labels=programs, rotation=45, ha="right")
    plt.title(f'Comparison of {column.replace("_", " ").capitalize()}')
    plt.ylabel(y_label)
    plt.legend()
    plt.tight_layout()
    plt.savefig(output_file)
    plt.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("wasm", help="Path to Wasm CSV file")
    parser.add_argument("criu", help="Path to CRIU CSV file")
    args = parser.parse_args()

    df_wasm = load_and_aggregate(args.wasm)
    df_criu = load_and_aggregate(args.criu)

    comparisons = {
        "checkpoint_time": "checkpoint-time-wasm-criu.png",
        "restore_time": "restore-time-wasm-criu.png",
        "snapshot_size": "snapshot-size-wasm-criu.png",
    }

    for column, output_file in comparisons.items():
        plot_comparison(df_wasm, df_criu, column, output_file)


if __name__ == "__main__":
    main()
