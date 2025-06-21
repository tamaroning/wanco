import pandas as pd
import matplotlib.pyplot as plt
import argparse
import numpy as np


def load_data(csv_path) -> pd.DataFrame:
    """生のデータを読み込む（集計せずに全データを保持）"""
    return pd.read_csv(csv_path)


def remove_outliers_by_column(df, column):
    """特定の列に基づいてハズレ値を除外する"""
    if len(df) <= 3:  # データが少ない場合はそのまま返す
        return df

    # IQRベースでハズレ値を判定
    q1 = df[column].quantile(0.25)
    q3 = df[column].quantile(0.75)
    iqr = q3 - q1
    lower_bound = q1 - 1.5 * iqr
    upper_bound = q3 + 1.5 * iqr

    # ハズレ値の個数
    num_outliers = len(df[(df[column] < lower_bound) | (df[column] > upper_bound)])
    print(
        f"Outliers in {column}: {num_outliers} ({(num_outliers / len(df)) * 100:.2f}%)"
    )

    # ハズレ値を除外
    return df[(df[column] >= lower_bound) & (df[column] <= upper_bound)]


def plot_comparison(
    df_wasm_raw: pd.DataFrame, df_criu_raw: pd.DataFrame, column: str, output_file: str
) -> None:
    print("--- Plotting", column, "---")

    # プログラム名の出現順序を保持
    wasm_programs = df_wasm_raw["program"].unique()
    criu_programs = df_criu_raw["program"].unique()

    # 出現順序を保持したプログラムリスト作成
    programs = []
    for prog in wasm_programs:
        if prog not in programs:
            programs.append(prog)

    for prog in criu_programs:
        if prog not in programs:
            programs.append(prog)

    # ハズレ値を除外したデータフレーム作成
    wasm_data = {}
    criu_data = {}

    for program in programs:
        # プログラムごとのデータ取得
        wasm_program_data = df_wasm_raw[df_wasm_raw["program"] == program]
        criu_program_data = df_criu_raw[df_criu_raw["program"] == program]

        # ハズレ値除外
        if len(wasm_program_data) > 0:
            wasm_program_data = remove_outliers_by_column(wasm_program_data, column)

        if len(criu_program_data) > 0:
            criu_program_data = remove_outliers_by_column(criu_program_data, column)

        # 各プログラムのデータ保存
        wasm_data[program] = (
            wasm_program_data[column].tolist() if len(wasm_program_data) > 0 else []
        )
        criu_data[program] = (
            criu_program_data[column].tolist() if len(criu_program_data) > 0 else []
        )

    # 平均値と標準偏差を計算
    values_wasm = [np.mean(wasm_data[p]) if wasm_data[p] else 0 for p in programs]
    stds_wasm = [np.std(wasm_data[p]) if wasm_data[p] else 0 for p in programs]
    values_criu = [np.mean(criu_data[p]) if criu_data[p] else 0 for p in programs]
    stds_criu = [np.std(criu_data[p]) if criu_data[p] else 0 for p in programs]

    # 比率分析
    ratios = [
        (
            values_criu[i] / values_wasm[i]
            if values_wasm[i] != 0 and values_criu[i] != 0
            else float("inf")
        )
        for i in range(len(values_wasm))
    ]
    # 最大の比を与えるプログラム
    valid_ratios = [r for r in ratios if r != float("inf") and not np.isnan(r)]
    if valid_ratios:
        max_ratio = max(valid_ratios)
        max_ratio_index = ratios.index(max_ratio)
        max_ratio_program = programs[max_ratio_index]
        max_ratio_value = ratios[max_ratio_index]
        max_ratio_wasm = values_wasm[max_ratio_index]
        max_ratio_criu = values_criu[max_ratio_index]
        print(f"Max ratio: {max_ratio_program} ({round(max_ratio_value, 4)})")
        print(f"\tCRIU={round(max_ratio_criu, 4)} => Wasm={round(max_ratio_wasm,4)}")

        # 最小の比を与えるプログラム
        min_ratio = min(valid_ratios)
        min_ratio_index = ratios.index(min_ratio)
        min_ratio_program = programs[min_ratio_index]
        min_ratio_value = ratios[min_ratio_index]
        min_ratio_wasm = values_wasm[min_ratio_index]
        min_ratio_criu = values_criu[min_ratio_index]
        print(f"Min ratio: {min_ratio_program} ({round(min_ratio_value,4)})")
        print(f"\tCRIU={round(min_ratio_criu,4)} => Wasm={round(min_ratio_wasm,4)}")

    # グラフ設定
    y_label = column.replace("_", " ").capitalize()

    color_wasm = "blue"
    color_criu = "orange"

    x = range(len(programs))
    width = 0.35

    if "time" in y_label:
        y_label += " [ms]"
        color_wasm = "#1f77b4"  # 青系統
        color_criu = "#ff7f0e"  # オレンジ系統
    elif "size" in y_label:
        y_label += " [MiB]"
        values_wasm = [v / 1024 / 1024 for v in values_wasm]
        stds_wasm = [s / 1024 / 1024 for s in stds_wasm]
        values_criu = [v / 1024 / 1024 for v in values_criu]
        stds_criu = [s / 1024 / 1024 for s in stds_criu]
        color_wasm = "#1f77b4"
        color_criu = "#ff7f0e"

    # プロット作成（高解像度設定）
    plt.figure(figsize=(12, 7), dpi=300)

    # 平均値＋標準偏差のエラー線を使用した棒グラフ作成
    bars_wasm = plt.bar(
        [i - width / 2 for i in x],
        values_wasm,
        width=width,
        yerr=stds_wasm,
        capsize=3,
        label="Wanco",
        color=color_wasm,
        alpha=0.8,
    )
    bars_criu = plt.bar(
        [i + width / 2 for i in x],
        values_criu,
        width=width,
        yerr=stds_criu,
        capsize=3,
        label="CRIU",
        color=color_criu,
        alpha=0.8,
    )

    # 箱ひげ図風の線（min/max線）は削除

    plt.xticks(ticks=x, labels=programs, rotation=45, ha="right", fontsize=20)
    plt.ylabel(y_label, fontsize=20)
    plt.yticks(fontsize=20)
    plt.legend(fontsize=20)
    plt.tight_layout()
    plt.savefig(output_file, dpi=300, bbox_inches="tight")
    plt.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("wasm", help="Path to Wasm CSV file")
    parser.add_argument("criu", help="Path to CRIU CSV file")
    args = parser.parse_args()

    # 生データを読み込む（集計しない）
    df_wasm_raw = load_data(args.wasm)
    df_criu_raw = load_data(args.criu)

    comparisons = {
        "checkpoint_time": "checkpoint-time-wasm-criu.png",
        "restore_time": "restore-time-wasm-criu.png",
        "snapshot_size": "snapshot-size-wasm-criu.png",
    }

    for column, output_file in comparisons.items():
        plot_comparison(df_wasm_raw, df_criu_raw, column, output_file)


if __name__ == "__main__":
    main()
