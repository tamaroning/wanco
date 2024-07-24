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

results = []

def measure_time(command):
    start_time = time.time()
    start_resources = resource.getrusage(resource.RUSAGE_CHILDREN)
    
    subprocess.run(command, shell=True, stdout=devnull, stderr=devnull)
    
    end_time = time.time()
    end_resources = resource.getrusage(resource.RUSAGE_CHILDREN)
    
    real = end_time - start_time
    usr = end_resources.ru_utime - start_resources.ru_utime
    sys = end_resources.ru_stime - start_resources.ru_stime
    return [real, usr, sys]

# main
if __name__ == "__main__":
    # suppress wasmtime new CLI warning
    os.environ["WASMTIME_NEW_CLI"] = "0"
    
    devnull = open(os.devnull, 'w')
    for t in test_cases:
        name = t["name"]
        src = t["src"]
        args = t["args"]
        opt_level = 2
        opt = f"-O{opt_level}"
        
        ##### native
        print(f"compile and execute native-{name}")
        # compile
        exe = os.path.join(benchdir, f"{name}.clang.native")
        cmd = [clang, opt,"-o", exe, os.path.join(benchdir, src)]
        subprocess.run(cmd)
        # execute
        cmd = [exe] + args
        native_res = measure_time(cmd)
        
        ##### compile c to wasm
        wasm = os.path.join(benchdir, f"{name}.clang.wasm")
        cmd = [wasi_clang, opt, "-o", wasm, os.path.join(benchdir, src)]
        subprocess.run(cmd)
        
        ##### wanco
        print(f"compile and execute wanco-{name}")
        # compile
        exe = os.path.join(benchdir, f"{name}.wanco.aot")
        cmd = [wanco, opt, "-o", exe, wasm]
        subprocess.run(cmd)
        # execute
        cmd = [exe, "--"] + args
        wanco_res = measure_time(cmd)
        
        ##### wasmtime
        print(f"compile and execute wasmtime-{name}")
        # compile
        cwasm = os.path.join(benchdir, f"{name}.wasmtime.cwasm")
        cmd = ["wasmtime", "compile", 
               # FIXME: why does this not work?
               #"-O", f"opt-level={str(opt_level)}", 
               "-o", cwasm, wasm]
        subprocess.run(cmd)
        # execute
        cmd = ["wasmtime", "--allow-precompiled", cwasm] + args
        wasmtime_res = measure_time(cmd)
        
        results.append({
            "name": name,
            # only use the first element (real) and discard usr and sys
            "native": native_res[0],
            "wanco": wanco_res[0],
            "wasmtime": wasmtime_res[0]
        })
    
    print(results)

