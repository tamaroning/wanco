import pandas as pd
import matplotlib.pyplot as plt
import argparse
import numpy as np


def load_and_aggregate(csv_path) -> pd.DataFrame:
    return pd.read_csv(csv_path)


def plot_migration_time(
    df_wasm: pd.DataFrame, df_criu: pd.DataFrame, output_file: str
) -> None:
    print("--- Plotting ---")

    # プログラム名の出現順序を保持
    wasm_programs = df_wasm["program"].unique()
    criu_programs = df_criu["program"].unique()
    # 両方のデータに存在するプログラムを元の出現順で保持
    # Wasmのプログラム順を優先し、CRIUにしか存在しないプログラムを後に追加
    programs = []
    for prog in wasm_programs:
        if prog not in programs:
            programs.append(prog)
    for prog in criu_programs:
        if prog not in programs:
            programs.append(prog)
    # 重複を削除（順序を保持）
    programs = list(dict.fromkeys(programs))

    # 合計時間に基づいてハズレ値を除外する関数
    def remove_outliers_by_total(checkpoint_times, restore_times):
        if (
            len(checkpoint_times) <= 3 or len(restore_times) <= 3
        ):  # データが少ない場合はそのまま返す
            return checkpoint_times, restore_times

        # 合計時間を計算
        total_times = [c + r for c, r in zip(checkpoint_times, restore_times)]

        # IQRベースでハズレ値を判定
        q1 = np.percentile(total_times, 25)
        q3 = np.percentile(total_times, 75)
        iqr = q3 - q1
        lower_bound = q1 - 1.5 * iqr
        upper_bound = q3 + 1.5 * iqr

        # ハズレ値ではないインデックスを特定
        valid_indices = [
            i
            for i, total in enumerate(total_times)
            if lower_bound <= total <= upper_bound
        ]

        # 有効なインデックスのデータのみを返す
        filtered_checkpoint = [checkpoint_times[i] for i in valid_indices]
        filtered_restore = [restore_times[i] for i in valid_indices]

        return filtered_checkpoint, filtered_restore

    # 各プログラムのチェックポイントと復元時間のデータを収集（合計時間でハズレ値を判定）
    wasm_data = {}
    criu_data = {}

    for program in programs:
        wasm_program_data = df_wasm[df_wasm["program"] == program]
        criu_program_data = df_criu[df_criu["program"] == program]

        # Wasmデータ
        checkpoint_times_wasm = (
            wasm_program_data["checkpoint_time"].tolist()
            if not wasm_program_data.empty
            else []
        )
        restore_times_wasm = (
            wasm_program_data["restore_time"].tolist()
            if not wasm_program_data.empty
            else []
        )

        # checkpointとrestoreの数が一致する場合のみ処理
        if (
            len(checkpoint_times_wasm) == len(restore_times_wasm)
            and len(checkpoint_times_wasm) > 0
        ):
            # 合計時間でハズレ値を除外
            filtered_checkpoint_wasm, filtered_restore_wasm = remove_outliers_by_total(
                checkpoint_times_wasm, restore_times_wasm
            )

            # 除外されたデータがある場合はメッセージを表示
            if len(filtered_checkpoint_wasm) < len(checkpoint_times_wasm):
                print(
                    f"Wasm {program}: {len(checkpoint_times_wasm) - len(filtered_checkpoint_wasm)} data points removed as outliers (by total time)"
                )
        else:
            filtered_checkpoint_wasm = checkpoint_times_wasm
            filtered_restore_wasm = restore_times_wasm

        wasm_data[program] = {
            "checkpoint_times": filtered_checkpoint_wasm,
            "restore_times": filtered_restore_wasm,
        }

        # CRIUデータ
        checkpoint_times_criu = (
            criu_program_data["checkpoint_time"].tolist()
            if not criu_program_data.empty
            else []
        )
        restore_times_criu = (
            criu_program_data["restore_time"].tolist()
            if not criu_program_data.empty
            else []
        )

        # checkpointとrestoreの数が一致する場合のみ処理
        if (
            len(checkpoint_times_criu) == len(restore_times_criu)
            and len(checkpoint_times_criu) > 0
        ):
            # 合計時間でハズレ値を除外
            filtered_checkpoint_criu, filtered_restore_criu = remove_outliers_by_total(
                checkpoint_times_criu, restore_times_criu
            )

            # 除外されたデータがある場合はメッセージを表示
            if len(filtered_checkpoint_criu) < len(checkpoint_times_criu):
                print(
                    f"CRIU {program}: {len(checkpoint_times_criu) - len(filtered_checkpoint_criu)} data points removed as outliers (by total time)"
                )
        else:
            filtered_checkpoint_criu = checkpoint_times_criu
            filtered_restore_criu = restore_times_criu

        criu_data[program] = {
            "checkpoint_times": filtered_checkpoint_criu,
            "restore_times": filtered_restore_criu,
        }

    # 各プログラムの平均値を計算（ハズレ値を除外したデータから）
    checkpointtime_wasm_means = [
        (
            np.mean(wasm_data[p]["checkpoint_times"])
            if wasm_data[p]["checkpoint_times"]
            else 0
        )
        for p in programs
    ]
    restoretime_wasm_means = [
        np.mean(wasm_data[p]["restore_times"]) if wasm_data[p]["restore_times"] else 0
        for p in programs
    ]
    checkpointtime_criu_means = [
        (
            np.mean(criu_data[p]["checkpoint_times"])
            if criu_data[p]["checkpoint_times"]
            else 0
        )
        for p in programs
    ]
    restoretime_criu_means = [
        np.mean(criu_data[p]["restore_times"]) if criu_data[p]["restore_times"] else 0
        for p in programs
    ]

    color_wasm_checkpoint = "royalblue"
    color_wasm_restore = "lightskyblue"
    color_criu_checkpoint = "darkorange"
    color_criu_restore = "moccasin"

    # グラフのセットアップ - 高解像度設定
    plt.figure(figsize=(12, 7), dpi=300)
    fig, ax = plt.subplots(figsize=(12, 7), dpi=300)

    # 棒グラフの幅と位置を設定
    bar_width = 0.35
    index = np.arange(len(programs))

    # Wasmの棒グラフ作成（checkpointとrestore）- 平均値
    bar1 = ax.bar(
        index - bar_width / 2,
        checkpointtime_wasm_means,
        bar_width,
        color=color_wasm_checkpoint,
        label="Wanco Checkpoint",
    )
    bar2 = ax.bar(
        index - bar_width / 2,
        restoretime_wasm_means,
        bar_width,
        bottom=checkpointtime_wasm_means,
        color=color_wasm_restore,
        label="Wanco Restore",
    )

    # CRIUの棒グラフ作成（checkpointとrestore）- 平均値
    bar3 = ax.bar(
        index + bar_width / 2,
        checkpointtime_criu_means,
        bar_width,
        color=color_criu_checkpoint,
        label="CRIU Checkpoint",
    )
    bar4 = ax.bar(
        index + bar_width / 2,
        restoretime_criu_means,
        bar_width,
        bottom=checkpointtime_criu_means,
        color=color_criu_restore,
        label="CRIU Restore",
    )

    # データの散らばりを表示するためのエラーバーとボックスプロット風の線を追加（線を細く）
    for i, program in enumerate(programs):
        # Wasm checkpoint - 散らばりを表示
        if wasm_data[program]["checkpoint_times"]:
            checkpoint_min = min(wasm_data[program]["checkpoint_times"])
            checkpoint_max = max(wasm_data[program]["checkpoint_times"])
            # 箱ひげ図風の線を追加 (checkpoint min-max範囲) - 線を細く
            ax.plot(
                [
                    index[i] - bar_width / 2 - bar_width * 0.3,
                    index[i] - bar_width / 2 + bar_width * 0.3,
                ],
                [checkpoint_min, checkpoint_min],
                color="black",
                linewidth=0.5,
            )
            ax.plot(
                [
                    index[i] - bar_width / 2 - bar_width * 0.3,
                    index[i] - bar_width / 2 + bar_width * 0.3,
                ],
                [checkpoint_max, checkpoint_max],
                color="black",
                linewidth=0.5,
            )
            ax.plot(
                [index[i] - bar_width / 2, index[i] - bar_width / 2],
                [checkpoint_min, checkpoint_max],
                color="black",
                linewidth=0.5,
            )

            # Wasm total (checkpoint+restore) - 散らばりを表示
            if wasm_data[program]["restore_times"]:
                total_times = [
                    c + r
                    for c, r in zip(
                        wasm_data[program]["checkpoint_times"],
                        wasm_data[program]["restore_times"],
                    )
                ]
                total_min = min(total_times)
                total_max = max(total_times)
                # 箱ひげ図風の線を追加 (total min-max範囲) - 線を細く
                ax.plot(
                    [
                        index[i] - bar_width / 2 - bar_width * 0.3,
                        index[i] - bar_width / 2 + bar_width * 0.3,
                    ],
                    [total_min, total_min],
                    color="black",
                    linewidth=0.5,
                )
                ax.plot(
                    [
                        index[i] - bar_width / 2 - bar_width * 0.3,
                        index[i] - bar_width / 2 + bar_width * 0.3,
                    ],
                    [total_max, total_max],
                    color="black",
                    linewidth=0.5,
                )
                ax.plot(
                    [index[i] - bar_width / 2, index[i] - bar_width / 2],
                    [total_min, total_max],
                    color="black",
                    linewidth=0.5,
                )

        # CRIU checkpoint - 散らばりを表示
        if criu_data[program]["checkpoint_times"]:
            checkpoint_min = min(criu_data[program]["checkpoint_times"])
            checkpoint_max = max(criu_data[program]["checkpoint_times"])
            # 箱ひげ図風の線を追加 (checkpoint min-max範囲) - 線を細く
            ax.plot(
                [
                    index[i] + bar_width / 2 - bar_width * 0.3,
                    index[i] + bar_width / 2 + bar_width * 0.3,
                ],
                [checkpoint_min, checkpoint_min],
                color="black",
                linewidth=0.5,
            )
            ax.plot(
                [
                    index[i] + bar_width / 2 - bar_width * 0.3,
                    index[i] + bar_width / 2 + bar_width * 0.3,
                ],
                [checkpoint_max, checkpoint_max],
                color="black",
                linewidth=0.5,
            )
            ax.plot(
                [index[i] + bar_width / 2, index[i] + bar_width / 2],
                [checkpoint_min, checkpoint_max],
                color="black",
                linewidth=0.5,
            )

            # CRIU total (checkpoint+restore) - 散らばりを表示
            if criu_data[program]["restore_times"]:
                total_times = [
                    c + r
                    for c, r in zip(
                        criu_data[program]["checkpoint_times"],
                        criu_data[program]["restore_times"],
                    )
                ]
                total_min = min(total_times)
                total_max = max(total_times)
                # 箱ひげ図風の線を追加 (total min-max範囲) - 線を細く
                ax.plot(
                    [
                        index[i] + bar_width / 2 - bar_width * 0.3,
                        index[i] + bar_width / 2 + bar_width * 0.3,
                    ],
                    [total_min, total_min],
                    color="black",
                    linewidth=0.5,
                )
                ax.plot(
                    [
                        index[i] + bar_width / 2 - bar_width * 0.3,
                        index[i] + bar_width / 2 + bar_width * 0.3,
                    ],
                    [total_max, total_max],
                    color="black",
                    linewidth=0.5,
                )
                ax.plot(
                    [index[i] + bar_width / 2, index[i] + bar_width / 2],
                    [total_min, total_max],
                    color="black",
                    linewidth=0.5,
                )

    # X軸のラベルとtickを設定
    ax.set_xlabel("Programs", fontsize=20)
    ax.set_xticks(index)
    ax.set_xticklabels(programs, rotation=45, ha="right", fontsize=20)

    plt.ylabel("Migration time [ms]", fontsize=20)
    plt.yticks(fontsize=20)
    plt.legend(fontsize=20)
    plt.tight_layout()

    # 高解像度で保存
    plt.savefig(output_file, dpi=300, bbox_inches="tight")
    plt.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("wasm", help="Path to Wasm CSV file")
    parser.add_argument("criu", help="Path to CRIU CSV file")
    args = parser.parse_args()

    df_wasm = load_and_aggregate(args.wasm)
    df_criu = load_and_aggregate(args.criu)

    output_file = "migration_time_comparison.png"

    plot_migration_time(df_wasm, df_criu, output_file)


if __name__ == "__main__":
    main()
