import argparse
import numpy as np
from common import *
import matplotlib.pyplot as plt


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
                "wamrc": os.path.getsize(wamrc_object),
                "wasmedge": os.path.getsize(wasmedge_object),
            }
        )

    # Plot and compare the code size of the different programs
    plt.figure(figsize=(12, 7), dpi=300)

    # 設定
    bar_width = 0.18  # 棒の幅を少し狭くする
    group_gap = 0.02  # グループ内の棒の間隔
    index = np.arange(len(sizes))
    programs_names = [program["name"] for program in sizes]

    # データ準備
    wanco_bars = [program["wanco"] for program in sizes]
    wanco_cr_bars = [program["wanco_cr"] for program in sizes]
    wamrc_bars = [program["wamrc"] for program in sizes]
    wasmedge_bars = [program["wasmedge"] for program in sizes]

    # バイト数をMiBに変換（より読みやすくするため）
    wanco_bars_mib = [size / (1024 * 1024) for size in wanco_bars]
    wanco_cr_bars_mib = [size / (1024 * 1024) for size in wanco_cr_bars]
    wamrc_bars_mib = [size / (1024 * 1024) for size in wamrc_bars]
    wasmedge_bars_mib = [size / (1024 * 1024) for size in wasmedge_bars]

    # 各棒の位置を計算（間隔を追加）
    pos_wamrc = index - (bar_width * 1.5 + group_gap)
    pos_wasmedge = index - (bar_width * 0.5 + group_gap / 3)
    pos_wanco = index + (bar_width * 0.5 + group_gap / 3)
    pos_wanco_cr = index + (bar_width * 1.5 + group_gap)

    # プログラムごとに、wamrc, wasmedge, wanco, wanco_crの順に棒を並べる
    bars_wamrc = plt.bar(
        pos_wamrc, wamrc_bars_mib, bar_width, label="wamrc", color="lightseagreen"
    )

    bars_wasmedge = plt.bar(
        pos_wasmedge,
        wasmedge_bars_mib,
        bar_width,
        label="wasmedge",
        color="#1f77b4",  # 青系統
    )

    bars_wanco = plt.bar(
        pos_wanco,
        wanco_bars_mib,
        bar_width,
        label="wanco",
        color="#ff7f0e",  # オレンジ系統
    )

    bars_wanco_cr = plt.bar(
        pos_wanco_cr, wanco_cr_bars_mib, bar_width, label="wanco_cr", color="hotpink"
    )

    # 比率分析（提供されたコード例に類似した分析）
    print("--- Code Size Comparison Analysis ---")
    # wancoとwamrcの比較
    ratios = [
        wanco_bars[i] / wamrc_bars[i] if wamrc_bars[i] != 0 else float("inf")
        for i in range(len(wanco_bars))
    ]

    valid_ratios = [r for r in ratios if r != float("inf") and not np.isnan(r)]
    if valid_ratios:
        max_ratio = max(valid_ratios)
        max_idx = ratios.index(max_ratio)
        print(
            f"Max ratio wanco/wamrc: {programs_names[max_idx]} ({round(max_ratio, 4)})"
        )
        print(
            f"Wanco size: {wanco_bars[max_idx] / 1024 / 1024} MiB = {wanco_bars[max_idx] / 1024} KiB"
        )
        print(
            f"Wamrc size: {wamrc_bars[max_idx] / 1024 / 1024} MiB = {wamrc_bars[max_idx] / 1024} KiB"
        )

        min_ratio = min(valid_ratios)
        min_idx = ratios.index(min_ratio)
        print(
            f"Min ratio wanco/wamrc: {programs_names[min_idx]} ({round(min_ratio, 4)})"
        )
        print(
            f"Wanco size: {wanco_bars[min_idx] / 1024 / 1024} MiB = {wanco_bars[min_idx] / 1024} KiB"
        )
        print(
            f"Wamrc size: {wamrc_bars[min_idx] / 1024 / 1024} MiB = {wamrc_bars[min_idx] / 1024} KiB"
        )

    # グラフの装飾
    plt.xlabel("Programs")
    plt.ylabel("Size [MiB]")
    plt.title("Code Size Comparison")
    plt.xticks(index, programs_names, rotation=45, ha="right")
    plt.legend()
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
                    fontsize=8,
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
