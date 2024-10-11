import matplotlib.pyplot as plt
import numpy as np

#No C/R Average throughput: 863.534546 tok/s
#C/R with no opt Average throughput: 781.291609 tok/s (+ 9.523989% overhead)
#C/R with reduced migration points Average throughput: 809.123416 tok/s (+ 6.300978% overhead)
#WAMR Average throughput: 72.527367 tok/s (+ 91.601104% overhead)
#WAMR with reduced migration points Average throughput: 115.837009 tok/s (+ 86.585712% overhead)

# データ
labels = ['No C/R', 'C/R', 'C/R with opt', 'AOT_STACK', 'AOT_STACK with opt']
throughputs = [863.53, 781.29, 809.12, 72.53, 115.84]
overheads = [0, 9.52, 6.30, 91.60, 86.59]

# グラフの設定
x = np.arange(len(labels))  # ラベルの位置
width = 0.4  # バーの幅

fig, ax1 = plt.subplots()

# スループットのバー
bars1 = ax1.bar(x - width/2, throughputs, width, label='Average Throughput (tok/s)', color='#0072B2')

# オーバーヘッドの棒グラフ
ax2 = ax1.twinx()  # 2つ目のy軸を追加
bars2 = ax2.bar(x + width/2, overheads, width, label='Overhead (%)', color='#E69F00')

# ラベルとタイトル
#ax1.set_xlabel('Condition')
ax1.set_ylabel('Throughput (tok/s)', color='skyblue')
ax2.set_ylabel('Overhead (%)', color='salmon')
ax1.set_title('Average Throughput of llama2.c')
ax1.set_xticks(x)
ax1.set_xticklabels(labels)

ax1.set_ylim(0, 1000)
ax2.set_ylim(0, 100)

# 凡例
fig.legend(loc="lower left", bbox_to_anchor=(0,0), bbox_transform=ax1.transAxes)
#fig.legend(loc="upper right", bbox_to_anchor=(1,0.9), bbox_transform=ax1.transAxes)

# グリッドとレイアウト
ax1.grid(axis='y', linestyle='--', alpha=0.7)
fig.tight_layout()

# グラフの表示
plt.show()