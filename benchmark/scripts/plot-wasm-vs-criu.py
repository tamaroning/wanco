import pandas as pd
import matplotlib.pyplot as plt
import argparse


def load_and_aggregate(csv_path) -> pd.DataFrame:
    df = pd.read_csv(csv_path)
    return df.groupby("program", sort=False).median()


def plot_comparison(
    df_a: pd.DataFrame, df_b: pd.DataFrame, column: str, output_file: str
) -> None:
    programs = df_a.index

    values_a = [df_a.loc[p, column] if p in df_a.index else 0 for p in programs]
    values_b = [df_b.loc[p, column] if p in df_b.index else 0 for p in programs]

    x = range(len(programs))
    width = 0.35

    plt.figure(figsize=(10, 6))
    plt.bar(
        [i - width / 2 for i in x],
        values_a,
        width=width,
        label="Wanco",
        color="skyblue",
    )
    plt.bar(
        [i + width / 2 for i in x], values_b, width=width, label="CRIU", color="salmon"
    )

    plt.xticks(ticks=x, labels=programs, rotation=45, ha="right")
    plt.ylabel(column.replace("_", " ").capitalize())
    plt.title(f'Comparison of {column.replace("_", " ").capitalize()}')
    plt.legend()
    plt.tight_layout()
    plt.savefig(output_file)
    plt.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("wasm", help="Path to Wasm CSV file")
    parser.add_argument("criu", help="Path to CRIU CSV file")
    args = parser.parse_args()

    df_a = load_and_aggregate(args.wasm)
    df_b = load_and_aggregate(args.criu)

    comparisons = {
        "checkpoint_time": "checkpoint-time-wasm-criu.png",
        "restore_time": "restore-time-wasm-criu.png",
        "snapshot_size": "snapshot-size-wasm-criu.png",
    }

    for column, output_file in comparisons.items():
        plot_comparison(df_a, df_b, column, output_file)


if __name__ == "__main__":
    main()
