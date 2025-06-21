import argparse
import numpy as np
from common import *
import matplotlib.pyplot as plt


def print_ratio_analysis(
    programs_names: list[str],
    a_name: str,
    a_sizes: list[int],
    b_name: str,
    b_sizes: list[int],
) -> None:
    print(f"--- {a_name}/{b_name} Ratio Analysis ---")
    # a/bの分析
    # 最高、最低、平均比率を計算
    ratios = [
        a_sizes[i] / b_sizes[i] if b_sizes[i] != 0 else float("inf")
        for i in range(len(a_sizes))
    ]
    valid_ratios = [r for r in ratios if r != float("inf") and not np.isnan(r)]
    if valid_ratios:
        max_ratio = max(valid_ratios)
        max_idx = ratios.index(max_ratio)
        print(
            f"Max ratio {a_name}/{b_name}: {programs_names[max_idx]} ({round(max_ratio, 4)})"
        )
        print(
            f"{a_name} size: {a_sizes[max_idx] / 1024 / 1024} MiB = {a_sizes[max_idx] / 1024} KiB"
        )
        print(
            f"{b_name} size: {b_sizes[max_idx] / 1024 / 1024} MiB = {b_sizes[max_idx] / 1024} KiB"
        )

        min_ratio = min(valid_ratios)
        min_idx = ratios.index(min_ratio)
        print(
            f"Min ratio {a_name}/{b_name}: {programs_names[min_idx]} ({round(min_ratio, 4)})"
        )
        print(
            f"{a_name} size: {a_sizes[min_idx] / 1024 / 1024} MiB = {a_sizes[min_idx] / 1024} KiB"
        )
        print(
            f"{b_name} size: {b_sizes[min_idx] / 1024 / 1024} MiB = {b_sizes[min_idx] / 1024} KiB"
        )

        average_ratio = sum(valid_ratios) / len(valid_ratios)
        print(f"Average ratio {a_name}/{b_name}: {round(average_ratio, 4)}")

    print()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-o",
        "--output-file",
        help="Path to output file",
        default="code-size-comparison.png",
    )

    args = parser.parse_args()

    sizes = []
    for program in programs:
        wanco_object = os.path.join(
            program.workdir, program.get_wanco_cmd()[0].replace(".aot", ".o")
        )
        wanco_cr_object = os.path.join(
            program.workdir, program.get_wanco_cr_cmd()[0].replace(".aot", ".o")
        )
        wanco_asyncify_object = os.path.join(
            program.workdir, program.get_wanco_cmd()[0].replace(".aot", ".asyncify.o")
        )
        wamrc_object = wanco_object.replace(
            "wanco-artifacts", "wamrc-artifacts"
        ).replace(".o", ".aot")
        wasmedge_object = wanco_object.replace(
            "wanco-artifacts", "wasmedge-artifacts"
        ).replace(".o", ".aot")
        sizes.append(
            {
                "name": program.name,
                "wanco": os.path.getsize(wanco_object),
                "wanco_cr": os.path.getsize(wanco_cr_object),
                "wanco_asyncify": os.path.getsize(wanco_asyncify_object),
                "wamrc": os.path.getsize(wamrc_object),
                "wasmedge": os.path.getsize(wasmedge_object),
            }
        )

    # Plot and compare the code size of the different programs
    plt.figure(figsize=(12, 7), dpi=300)

    # 設定
    bar_width = 0.15  # 棒の幅をさらに狭くする（5つの棒を入れるため）
    group_gap = 0.01  # グループ内の棒の間隔
    index = np.arange(len(sizes))
    programs_names = [program["name"] for program in sizes]

    # データ準備
    wanco_bars = [program["wanco"] for program in sizes]
    wanco_cr_bars = [program["wanco_cr"] for program in sizes]
    wanco_asyncify_bars = [program["wanco_asyncify"] for program in sizes]
    wamrc_bars = [program["wamrc"] for program in sizes]
    wasmedge_bars = [program["wasmedge"] for program in sizes]

    # バイト数をMiBに変換（より読みやすくするため）
    wanco_bars_mib = [size / (1024 * 1024) for size in wanco_bars]
    wanco_cr_bars_mib = [size / (1024 * 1024) for size in wanco_cr_bars]
    wanco_asyncify_bars_mib = [size / (1024 * 1024) for size in wanco_asyncify_bars]
    wamrc_bars_mib = [size / (1024 * 1024) for size in wamrc_bars]
    wasmedge_bars_mib = [size / (1024 * 1024) for size in wasmedge_bars]

    # 各棒の位置を計算（間隔を追加）
    pos_wanco = index - (bar_width * 2 + group_gap * 2)
    pos_wanco_cr = index - (bar_width + group_gap)
    pos_wanco_asyncify = index
    pos_wamrc = index + (bar_width + group_gap)
    pos_wasmedge = index + (bar_width * 2 + group_gap * 2)

    # 各プログラムごとに平均値と標準偏差を計算（今回は1サンプルなのでstd=0になるが、将来拡張のために記述）
    # もし複数サンプルがある場合はここでリスト化してnp.mean/np.stdを使う
    # 今回は各バー1サンプルなのでエラー線は0
    wanco_stds = [0 for _ in wanco_bars_mib]
    wanco_cr_stds = [0 for _ in wanco_cr_bars_mib]
    wanco_asyncify_stds = [0 for _ in wanco_asyncify_bars_mib]
    wamrc_stds = [0 for _ in wamrc_bars_mib]
    wasmedge_stds = [0 for _ in wasmedge_bars_mib]

    # 各棒グラフをエラー線付きで描画
    bars_wanco = plt.bar(
        pos_wanco,
        wanco_bars_mib,
        bar_width,
        yerr=wanco_stds,
        capsize=3,
        label="Wanco",
        color="#1f77b4",  # 青
        alpha=0.8,
    )

    bars_wanco_cr = plt.bar(
        pos_wanco_cr,
        wanco_cr_bars_mib,
        bar_width,
        yerr=wanco_cr_stds,
        capsize=3,
        label="Wanco w/ C/R",
        color="#9467bd",  # 紫
        alpha=0.8,
    )

    bars_wanco_asyncify = plt.bar(
        pos_wanco_asyncify,
        wanco_asyncify_bars_mib,
        bar_width,
        yerr=wanco_asyncify_stds,
        capsize=3,
        label="Wanco w/ asyncify",
        color="#17becf",  # シアン
        alpha=0.8,
    )

    bars_wamrc = plt.bar(
        pos_wamrc,
        wamrc_bars_mib,
        bar_width,
        yerr=wamrc_stds,
        capsize=3,
        label="WAMR",
        color="#ff7f0e",  # オレンジ
        alpha=0.8,
    )

    bars_wasmedge = plt.bar(
        pos_wasmedge,
        wasmedge_bars_mib,
        bar_width,
        yerr=wasmedge_stds,
        capsize=3,
        label="WasmEdge",
        color="#d62728",  # 赤
        alpha=0.8,
    )

    # 比率分析（提供されたコード例に類似した分析）
    print("--- Code Size Comparison Analysis ---")

    # wamr/wancoの比較
    print_ratio_analysis(
        programs_names,
        "wamrc",
        wamrc_bars,
        "wanco",
        wanco_bars,
    )
    # wasmedge/wancoの比較
    print_ratio_analysis(
        programs_names,
        "wasmedge",
        wasmedge_bars,
        "wanco",
        wanco_bars,
    )
    # wanco_cr/wancoの比較
    print_ratio_analysis(
        programs_names,
        "wanco_cr",
        wanco_cr_bars,
        "wanco",
        wanco_bars,
    )
    # wanco_asyncify/wancoの比較
    print_ratio_analysis(
        programs_names,
        "wanco_asyncify",
        wanco_asyncify_bars,
        "wanco",
        wanco_bars,
    )

    FONT_SIZE = 20

    # グラフの装飾
    plt.ylabel("Code size [MiB]", fontsize=FONT_SIZE)
    plt.xticks(index, programs_names, rotation=45, ha="right", fontsize=FONT_SIZE)
    plt.yticks(fontsize=FONT_SIZE)
    plt.legend(fontsize=FONT_SIZE)
    plt.grid(axis="y", linestyle="--", alpha=0.7)

    # 各バーに値を表示（オプション）
    def add_labels(bars):
        for bar in bars:
            height = bar.get_height()
            if height > 0:
                plt.text(
                    bar.get_x() + bar.get_width() / 2.0,
                    height + 0.02,
                    f"{height:.2f}",
                    ha="center",
                    va="bottom",
                    fontsize=FONT_SIZE,
                    rotation=90,
                )

    # バーの値表示が必要な場合、コメント解除
    # add_labels(bars_wamrc)
    # add_labels(bars_wasmedge)
    # add_labels(bars_wanco)
    # add_labels(bars_wanco_cr)

    plt.tight_layout()
    plt.savefig(args.output_file, dpi=300, bbox_inches="tight")
    print(f"Plot saved to {args.output_file}")


if __name__ == "__main__":
    main()
