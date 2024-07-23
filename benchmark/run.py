#!/usr/bin/env python3
import os
import subprocess
import time
import resource

benchdir = os.path.dirname(os.path.abspath(__file__))
wanco = os.path.join(benchdir, '../target/release/wanco')

wasi_sdk = os.environ.get('WASI_SDK')
wasi_clang = os.path.join(wasi_sdk, 'bin/clang')

clang = "clang"

test_cases = [
    {
        "name": "fib",
        "src": "fib.c",
        "args": []
    }
]

def measure_time(command):
    start_time = time.time()
    start_resources = resource.getrusage(resource.RUSAGE_CHILDREN)
    
    subprocess.run(command, shell=True, stdout=devnull, stderr=devnull)
    
    end_time = time.time()
    end_resources = resource.getrusage(resource.RUSAGE_CHILDREN)
    
    real = end_time - start_time
    usr = end_resources.ru_utime - start_resources.ru_utime
    sys = end_resources.ru_stime - start_resources.ru_stime
    return real, usr, sys

# main
if __name__ == "__main__":
    devnull = open(os.devnull, 'w')
    for t in test_cases:
        name = t["name"]
        src = t["src"]
        args = t["args"]
        opt = "-O3"
        
        print(f"compile and execute native-{name}")
        
        # compile native
        exe = os.path.join(benchdir, f"{name}.clang.native")
        cmd = [clang, opt, "-o", exe, os.path.join(benchdir, src)]
        subprocess.run(cmd)
        
        # execute native
        cmd = [exe] + args
        real, usr, sys = measure_time(cmd)
        
        print(f"native-{name}: real={real:.3f}s usr={usr:.3f}s sys={sys:.3f}s")

        print(f"compile and execute wanco-{name}")
        
        # compile to wasm
        wasm = os.path.join(benchdir, f"{name}.clang.wasm")
        cmd = [wasi_clang, opt, "-o", wasm, os.path.join(benchdir, src)]
        subprocess.run(cmd)
        exe = os.path.join(benchdir, f"{name}.wanco.aot")
        cmd = [wanco, opt, "-o", exe, wasm]
        subprocess.run(cmd)
        
        # execute native
        cmd = [exe, "--llvm-layout", "--"] + args
        real, usr, sys = measure_time(cmd)
        
        print(f"wanco-{name}: real={real:.3f}s usr={usr:.3f}s sys={sys:.3f}s")
        

