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


# 全プログラムを1つのグラフにまとめた棒グラフを作成（間隔調整版）
def create_combined_bar_plot(plot_data, filename="overhead.jpg"):
    fig, ax = plt.subplots(figsize=(16, 8))  # 幅を少し広げる

    programs = list(plot_data.keys())
    runtimes = ["Wanco w/ C/R", "WAMR", "WasmEdge"]

    # カラーパレット
    colors = {"Wanco w/ C/R": "#1f77b4", "WAMR": "#ff7f0e", "WasmEdge": "#d62728"}

    # 棒の幅と位置の設定（間隔を広げる）
    bar_width = 0.2  # 棒の幅を少し細く
    group_spacing = 0.0  # グループ間の間隔を調整
    x = np.arange(len(programs)) * (1 + group_spacing)  # プログラム間の間隔を広げる

    # 各ランタイムごとに棒グラフを描画
    for i, runtime in enumerate(runtimes):
        means = []
        stds = []

        for program in programs:
            if runtime in plot_data[program]:
                data = plot_data[program][runtime]
                means.append(np.mean(data))
                stds.append(np.std(data))
            else:
                means.append(0)
                stds.append(0)

        # 各ランタイムの棒の位置を調整（ランタイム間に間隔を作る）
        runtime_spacing = 0.01  # ランタイム間の間隔
        total_width = len(runtimes) * bar_width + (len(runtimes) - 1) * runtime_spacing
        start_offset = -total_width / 2 + bar_width / 2
        runtime_offset = start_offset + i * (bar_width + runtime_spacing)

        bars = ax.bar(
            x + runtime_offset,
            means,
            bar_width,
            yerr=stds,
            capsize=3,
            label=runtime,
            color=colors[runtime],
            alpha=0.8,
        )

    # グラフの設定
    # ax.set_xlabel("Programs", fontsize=14)
    ax.set_ylabel("Ratio of Execution Time to Wanco wo/ C/R", fontsize=14)
    ax.set_xticks(x)  # 中央の位置にラベルを配置
    ax.set_xticklabels(programs, rotation=45, ha="right", fontsize=14)
    ax.legend(loc="upper left", fontsize=14)
    ax.grid(True, alpha=0.3, axis="y")

    # y軸の範囲を調整
    all_means = []
    all_stds = []
    for program in programs:
        for runtime in runtimes:
            if runtime in plot_data[program]:
                data = plot_data[program][runtime]
                all_means.append(np.mean(data))
                all_stds.append(np.std(data))

    if all_means:
        y_max = max(all_means) + max(all_stds) + 0.2
        ax.set_ylim(0, y_max)

    # x軸の範囲を調整して余白を確保
    ax.set_xlim(-0.5, x[-1] + 0.5)

    plt.tight_layout()
    plt.savefig(filename, dpi=300)
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
                print(
                    f"    {runtime}: 平均 {np.mean(ratios):.3f} ± {np.std(ratios):.3f}"
                )

        print("棒グラフを作成中...")
        create_combined_bar_plot(plot_data)

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
