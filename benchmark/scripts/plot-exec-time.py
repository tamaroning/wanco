import json
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from collections import defaultdict
import sys


# JSONファイルを読み込む
def load_data(filename):
    with open(filename, "r") as f:
        data = json.load(f)
    return data


# データを整理してプロット用に準備
def prepare_plot_data(data):
    # プログラム名とランタイムごとにデータを整理
    plot_data = defaultdict(dict)

    for result in data["results"]:
        name = result["name"]
        runtime = result.get("runtime", "unknown")
        # runtimeの名前を正規化
        if runtime == "wanco":
            runtime = "Wanco"
        elif runtime == "wanco-cr":
            runtime = "Wanco w/ C/R"
        elif runtime == "wamr":
            runtime = "WAMR"
        elif runtime == "wasmedge":
            runtime = "WasmEdge"
        else:
            raise ValueError(f"Unknown runtime: {runtime}")

        # ratiosがある場合はそれを使用、ない場合は基準として1.0を設定
        if "ratios" in result:
            ratios = result["ratios"]
            # ランタイム名を判定（name中に含まれる情報から）
            if "w/ cr" in name:
                runtime_key = "with cr"
                program_name = name.replace(" w/ cr", "")
            else:
                runtime_key = runtime
                program_name = name
        else:
            raise ValueError(
                f"Missing 'ratios' in result for {name} with runtime {runtime}"
            )

        plot_data[program_name][runtime_key] = ratios

    return plot_data


# グループ化された箱ひげ図を作成
def create_grouped_box_plot(plot_data, filename="overhead.jpg"):
    # プログラムごとにサブプロットを作成
    n_programs = len(plot_data)
    fig, axes = plt.subplots(1, n_programs, figsize=(1.5 * n_programs, 6), sharey=True)

    # プログラムが1つの場合は配列にする
    if n_programs == 1:
        axes = [axes]

    colors = ["#ff7f0e", "#2ca02c", "#d62728", "#1f77b4"]

    # 全ランタイムを収集
    all_runtimes = set()
    for program_data in plot_data.values():
        all_runtimes.update(k for k in program_data.keys())
    all_runtimes = sorted(list(all_runtimes))

    print(f"利用可能なランタイム: {all_runtimes}")

    # フォントサイズ設定
    label_fontsize = 14
    tick_fontsize = 11
    title_fontsize = 14

    # 各プログラムについてサブプロットを作成
    for idx, (program, runtime_data) in enumerate(plot_data.items()):
        ax = axes[idx]

        runtimes = ["Wanco w/ C/R", "WAMR", "WasmEdge"]
        data_to_plot = [runtime_data[runtime] for runtime in runtimes]

        # 箱ひげ図を描画
        bp = ax.boxplot(
            data_to_plot,
            labels=runtimes,
            patch_artist=True,
            showfliers=True,
            meanline=True,
            widths=0.6,  # デフォルトは0.5、これを小さく
        )

        # 箱ひげ図の塗りつぶしを無効化（no fill）
        for box in bp["boxes"]:
            box.set_facecolor("none")
            box.set_alpha(1.0)

        # サブプロットの設定
        ax.set_title(f"{program}", fontsize=title_fontsize)
        ax.grid(True, alpha=0.3)

        # y軸ラベルは最初のサブプロットのみ
        if idx == 0:
            ax.set_ylabel("Ratio of Execution Time to Wanco wo/ C/R", fontsize=label_fontsize)

        # x軸のラベルを45度回転
        ax.tick_params(axis="x", labelsize=tick_fontsize, rotation=45)
        ax.tick_params(axis="y", labelsize=tick_fontsize)
        for tick in ax.get_xticklabels():
            tick.set_horizontalalignment("right")

    plt.tight_layout(pad=0.0, w_pad=0.2, h_pad=0.2)  # サブプロット間の余白を狭く
    plt.savefig(filename, dpi=300, bbox_inches="tight")
    plt.show()


def main():
    # JSONファイル名を指定: args[1]
    filename = sys.argv[1] if len(sys.argv) > 1 else "overhead.json"

    try:
        # データを読み込み
        data = load_data(filename)

        # プロット用データを準備
        plot_data = prepare_plot_data(data)

        print("利用可能なプログラム:")
        for program in plot_data.keys():
            print(f"  - {program}")
            for runtime, ratios in plot_data[program].items():
                print(f"    {runtime}: 平均 {np.mean(ratios):.3f}")

        print("グループ化された箱ひげ図を作成中...")
        create_grouped_box_plot(plot_data)

        print("完了！")

    except FileNotFoundError:
        print(f"ファイル '{filename}' が見つかりません。")
        print("ファイル名を確認してください。")
    except json.JSONDecodeError:
        print("JSONファイルの形式が正しくありません。")
    except Exception as e:
        print(f"エラーが発生しました: {e}")


if __name__ == "__main__":
    main()
