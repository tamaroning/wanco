## Testbed

CPU: CPU: Intel Core i7-14700F
OS: Ubuntu 24.04.2 LTS
RAM: 32GB

## Ratio of execution time (w/ CR vs. w/o CR)

```
llama2.c w/ cr : Median ratio 1.564
nbody w/ cr : Median ratio 0.89
binary-trees w/ cr : Median ratio 1.834
fannkuch-redux w/ cr : Median ratio 1.404
mandelbrot w/ cr : Median ratio 1.321
bc w/ cr : Median ratio 1.246
bfs w/ cr : Median ratio 1.399
cc w/ cr : Median ratio 1.237
cc_sv w/ cr : Median ratio 1.21
pr w/ cr : Median ratio 1.421
pr_spmv w/ cr : Median ratio 1.359
sssp w/ cr : Median ratio 1.319
--------------------
Mean median ratio 1.35
Max ratio 1.834
Min ratio 0.89
```

## Checkpoint time (ms)

- Max ratio: nbody (5.05)
    - CRIU=5.28 => Wanco=1.05
- Min ratio: pr (1.12)
    - CRIU=43.48 => Wanco=38.89

## Restore time (ms)

- Max ratio: fannkuch-redux (41.09)
    - CRIU=3.94 => Wanco=0.10
- Min ratio: pr (0.54)
    - CRIU=14.64 => Wanco=26.86

## Snapshot size (bytes)

- Max ratio: fannkuch-redux (25.36)
    - CRIU=3331469.00 => Wanco=131392.00
- Min ratio: cc (1.06)
    - CRIU=75439921.50 => Wanco=71436198.00
