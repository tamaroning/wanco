import argparse
import json

import matplotlib.pyplot as plt

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("file", help="JSON file with benchmark results")
parser.add_argument("--title", help="Plot Title")
parser.add_argument("--sort-by", choices=["median"], help="Sort method")
parser.add_argument("-o", "--output", help="Save image to the given filename.")

args = parser.parse_args()

with open(args.file, encoding="utf-8") as f:
    results = json.load(f)["results"]

    labels_ = [b["name"] for b in results]
    ratios_ = [b["ratios"] for b in results]

    labels = []
    ratios = []
    for label, ratio in zip(labels_, ratios_):
        if " w/ cr" in label:
            labels.append(label.replace(" w/ cr", ""))
            ratios.append(ratio)

if args.sort_by == "median":
    medians = [b["median"] for b in results]
    indices = sorted(range(len(labels)), key=lambda k: medians[k])

    labels = []
    ratios = []
    for i in indices:
        if " w/ cr" not in labels[i]:
            labels.append(labels[i].replace(" w/ cr", ""))
            ratios.append(ratios[i])


plt.figure(figsize=(10, 6), constrained_layout=True)
boxplot = plt.boxplot(ratios, vert=True, patch_artist=True)
cmap = plt.get_cmap("rainbow")
colors = [cmap(val / len(ratios)) for val in range(len(ratios))]

for patch, color in zip(boxplot["boxes"], colors):
    patch.set_facecolor(color)

if args.title:
    plt.title(args.title)
plt.legend(handles=boxplot["boxes"], labels=labels, loc="best", fontsize="medium")
plt.ylabel("Execution time [ratio]")
#plt.ylim(0.75, 1.5)
plt.xticks(list(range(1, len(labels) + 1)), labels, rotation=45)
if args.output:
    plt.savefig(args.output)
else:
    plt.show()
