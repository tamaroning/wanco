import matplotlib.pyplot as plt
import numpy as np

#x86-64
#No C/R Average throughput: 863.534546 tok/s
#C/R with no opt Average throughput: 781.291609 tok/s (+ 9.523989% overhead)
#C/R with reduced migration points Average throughput: 809.123416 tok/s (+ 6.300978% overhead)
#WAMR Average throughput: 72.527367 tok/s (+ 91.601104% overhead)
#WAMR with reduced migration points Average throughput: 115.837009 tok/s (+ 86.585712% overhead)

# AArch64
#no-cr: 264.308469 tok/s (0% overhead)
#cr: 215.511597 tok/s (18.462091% overhead)
#cr-opt: 224.615343 tok/s (15.017727% overhead)
#cr-wamr: 19.667075 tok/s (92.559044% overhead)
#cr-opt-wamr: 33.223547 tok/s (87.430010% overhead)

# データ
labels = ['No C/R', 'C/R', 'C/R with opt', 'AOT_STACK']
x86_64_throughputs = [863.53/863.53, 781.29/863.53, 809.12/863.53, 72.53/863.53]
aa64_throughputs = [264.31/264.31, 215.51/264.31, 224.62/264.31, 19.67/264.31]

# グラフの設定
x = np.arange(len(labels))  # ラベルの位置
width = 0.4  # バーの幅

fig, ax1 = plt.subplots()

# スループットのバー
bars1 = ax1.bar(x - width/2, x86_64_throughputs, width, label='x86-64', color='#0072B2')
bars2 = ax1.bar(x + width/2, aa64_throughputs, width, label='AArch64', color='#D55E00')

# オーバーヘッドの棒グラフ
#ax2 = ax1.twinx()  # 2つ目のy軸を追加
#bars2 = ax2.bar(x + width/2, overheads, width, label='Overhead (%)', color='#E69F00')

# ラベルとタイトル
#ax1.set_xlabel('Condition')
ax1.set_ylabel('Throughput Ratio to No C/R')
#ax2.set_ylabel('Overhead (%)', color='salmon')
ax1.set_title('Average Throughput of llama2.c')
ax1.set_xticks(x)
ax1.set_xticklabels(labels)

ax1.set_ylim(0, 1)
#ax2.set_ylim(0, 100)

# 凡例
#fig.legend(loc="lower left", bbox_to_anchor=(0,0), bbox_transform=ax1.transAxes)
fig.legend(loc="upper right", bbox_to_anchor=(1,1), bbox_transform=ax1.transAxes)

# グリッドとレイアウト
ax1.grid(axis='y', linestyle='--', alpha=0.7)
fig.tight_layout()

# グラフの表示
plt.show()