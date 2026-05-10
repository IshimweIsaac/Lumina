import sys
import os
import time
import json
import statistics

# Add lumina_py to path
PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
sys.path.append(os.path.join(PROJECT_ROOT, "crates", "lumina_ffi"))

from lumina_py import LuminaRuntime

def generate_source(n_entities: int) -> str:
    lines = [
        "entity Device {",
        "  temp: Number",
        "  isOnline: Boolean",
        "  isOverheating := temp > 80",
        "  status := if isOverheating then \"danger\" else if not isOnline then \"offline\" else \"ok\"",
        "}"
    ]
    for i in range(1, n_entities + 1):
        name = f"dev{i}"
        lines.append(f"let {name} = Device {{ temp: {20 + (i % 60)}, isOnline: true }}")
        lines.append(f"rule \"{name}_alert\"")
        lines.append(f"when {name}.isOverheating becomes true {{")
        lines.append(f"  show \"CRITICAL: {name} overheating!\"")
        lines.append("}")
    
    return "\n".join(lines)

def run_stress_test(n_entities: int):
    print(f"--- Stress Test: {n_entities} Entities ---")
    
    # 1. Generation
    start_gen = time.perf_counter()
    source = generate_source(n_entities)
    end_gen = time.perf_counter()
    print(f"Generation: {end_gen - start_gen:.4f}s")
    
    # 2. Initialization (Parse/Analyze)
    try:
        start_init = time.perf_counter()
        rt = LuminaRuntime.from_source(source)
        end_init = time.perf_counter()
        print(f"Initialization: {end_init - start_init:.4f}s")
    except Exception as e:
        print(f"Failed to initialize: {e}")
        return

    # 3. Baseline Tick
    tick_times = []
    for _ in range(10):
        start_tick = time.perf_counter()
        rt.tick()
        tick_times.append(time.perf_counter() - start_tick)
    
    avg_baseline = statistics.mean(tick_times) * 1000
    print(f"Avg Baseline Tick: {avg_baseline:.2f}ms")

    # 4. Burst Load (Updating 10% of entities)
    n_updates = n_entities // 10
    print(f"Applying {n_updates} events...")
    
    start_burst = time.perf_counter()
    for i in range(1, n_updates + 1):
        # Every 5th update triggers an overheating alert
        temp = 90 if i % 5 == 0 else 40
        rt.apply_event(f"dev{i}", "temp", temp)
    
    # Process all changes
    rt.tick()
    end_burst = time.perf_counter()
    
    burst_total = end_burst - start_burst
    print(f"Burst Load (Events + Tick): {burst_total:.4f}s")
    print(f"Throughput: {n_updates / burst_total:.2f} events/sec")

    # 5. Verification
    messages = rt.get_messages()
    expected_alerts = n_updates // 5
    print(f"Alerts triggered: {len(messages)} (Expected ~{expected_alerts})")
    
    print("-" * 40)
    return {
        "n_entities": n_entities,
        "init_time": end_init - start_init,
        "avg_baseline_ms": avg_baseline,
        "throughput": n_updates / burst_total,
        "alerts": len(messages)
    }

if __name__ == "__main__":
    if len(sys.argv) > 1:
        sizes = [int(x) for x in sys.argv[1:]]
    else:
        sizes = [1000, 5000, 10000]
    
    results = []
    for size in sizes:
        res = run_stress_test(size)
        if res:
            results.append(res)
    
    # Summary Table
    print("\nSUMMARY")
    print(f"{'Entities':<10} | {'Init (s)':<10} | {'Tick (ms)':<10} | {'EPS':<10}")
    print("-" * 45)
    for r in results:
        print(f"{r['n_entities']:<10} | {r['init_time']:<10.3f} | {r['avg_baseline_ms']:<10.2f} | {r['throughput']:<10.1f}")
