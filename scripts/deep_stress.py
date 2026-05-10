import sys
import os
import time
import json
import statistics
import traceback

# Add lumina_py to path
PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
sys.path.append(os.path.join(PROJECT_ROOT, "crates", "lumina_ffi"))

try:
    from lumina_py import LuminaRuntime
except ImportError:
    print("Error: lumina_py not found. Ensure crates/lumina_ffi is built.")
    sys.exit(1)

class LuminaStressTester:
    def __init__(self):
        self.results = {}

    def run_test(self, name, source, actions=None, iterations=1):
        print(f"\n>>> Running Test: {name}")
        try:
            start_init = time.perf_counter()
            rt = LuminaRuntime.from_source(source)
            init_time = time.perf_counter() - start_init
            print(f"  [PASS] Initialization: {init_time:.4f}s")

            tick_times = []
            for i in range(iterations):
                if actions:
                    actions(rt, i)
                start_tick = time.perf_counter()
                rt.tick()
                tick_times.append(time.perf_counter() - start_tick)

            avg_tick = statistics.mean(tick_times) if tick_times else 0
            print(f"  [PASS] Avg Tick: {avg_tick*1000:.2f}ms")
            
            return {
                "status": "PASS",
                "init_time": init_time,
                "avg_tick_ms": avg_tick * 1000,
                "output": rt.get_messages()
            }
        except Exception as e:
            print(f"  [FAIL] {name}: {e}")
            # traceback.print_exc()
            return {
                "status": "FAIL",
                "error": str(e)
            }

    def test_depth(self, depth):
        """Tests dependency propagation depth."""
        lines = ["entity Node { val: Number \n derived: Number }"]
        lines.append("let n0 = Node { val: 0 }")
        for i in range(1, depth + 1):
            lines.append(f"entity Level{i} {{ prev_val: Number \n val := prev_val + 1 }}")
            prev = f"n{i-1}.val" if i > 1 else "n0.val"
            lines.append(f"let n{i} = Level{i} {{ prev_val: {prev} }}")
        
        source = "\n".join(lines)
        return self.run_test(f"Depth_{depth}", source)

    def test_wide_fleet(self, count):
        """Tests many instances of a single entity."""
        lines = [
            "entity Device { temp: Number \n isHot := temp > 80 }",
            "aggregate HotStats over Device { totalHot := count(isHot) }"
        ]
        for i in range(count):
            lines.append(f"let d{i} = Device {{ temp: 25 }}")
        
        def actions(rt, i):
            rt.apply_event(f"d{i % count}", "temp", 90 if i % 2 == 0 else 25)

        source = "\n".join(lines)
        return self.run_test(f"WideFleet_{count}", source, actions, iterations=10)

    def test_rule_storm(self, rule_count):
        """Tests many rules firing at once."""
        lines = ["entity Trigger { active: Boolean }", "let t = Trigger { active: false }"]
        for i in range(rule_count):
            # Using Entity name 'Trigger' instead of parameter 't' due to binding issue
            lines.append(f"rule r{i} when Trigger.active becomes true {{ show \"fire {i}\" }}")
        
        def actions(rt, i):
            rt.apply_event("t", "active", i % 2 == 0)

        source = "\n".join(lines)
        return self.run_test(f"RuleStorm_{rule_count}", source, actions, iterations=4)

    def test_circular_dependency(self):
        """Tests if the engine catches or crashes on circularity."""
        # Note: Lumina has a MAX_DEPTH of 100. Circular dependencies should hit this.
        source = """
        entity Node { val: Number \n target: ref Node \n target_val := target.val }
        let n1 = Node { val: 1, target: "n1" }
        let n2 = Node { val: 2, target: "n2" }
        rule Circ when Node.target_val > 5 { update Node.val to Node.val + 1 }
        """
        def actions(rt, i):
            if i == 0:
                rt.apply_event("n1", "target", "n2")
                rt.apply_event("n2", "target", "n1")
                rt.apply_event("n1", "val", 6)
        
        return self.run_test("CircularDependency", source, actions, iterations=1)

    def test_prev_churn(self, iterations):
        """Tests 'prev()' stability under rapid updates."""
        source = """
        entity Counter { 
          val: Number
          diff := val - prev(val) 
        }
        let c = Counter { val: 0 }
        rule CheckDiff when Counter.diff != 0 { show "Diff is {Counter.val}" }
        """
        def actions(rt, i):
            rt.apply_event("c", "val", i + 1)
            
        return self.run_test("PrevChurn", source, actions, iterations=iterations)

    def test_semantic_minefield(self):
        """Tests edge cases and keyword combinations."""
        source = """
        entity Mixed {
          s: Secret
          t: Timestamp
          l: Number[]
          is_ready: Boolean
        }
        
        fn compute(x: Number) -> Number { x * 2 }
        
        let m = Mixed { 
          s: "init", 
          t: now(), 
          l: [1, 2, 3],
          is_ready: true 
        }
        
        rule Complex
          when Mixed.is_ready becomes true
          and len(Mixed.l) > 2 {
            show "Complex rule fired"
            alert severity: "info", message: "Everything looks good"
        }
        """
        return self.run_test("SemanticMinefield", source)



if __name__ == "__main__":
    tester = LuminaStressTester()
    results = {}
    
    # 1. Depth Test (Lumina MAX_DEPTH is 100)
    results["depth_50"] = tester.test_depth(50)
    results["depth_101"] = tester.test_depth(101) # Should fail or cap
    
    # 2. Scale Test
    results["wide_1000"] = tester.test_wide_fleet(1000)
    results["wide_5000"] = tester.test_wide_fleet(5000)
    
    # 3. Churn Test
    results["rule_storm_500"] = tester.test_rule_storm(500)
    results["prev_churn_100"] = tester.test_prev_churn(100)
    
    # 4. Stability
    results["circular"] = tester.test_circular_dependency()
    results["minefield"] = tester.test_semantic_minefield()

    # Save summary
    with open("stress_results.json", "w") as f:
        # Convert non-serializable parts if needed
        json.dump(results, f, indent=2, default=lambda x: str(x))
    
    print("\n" + "="*40)
    print("STRESS TEST SUMMARY")
    print("="*40)
    for name, res in results.items():
        status = res.get("status", "UNKNOWN")
        info = f"Init: {res.get('init_time',0):.3f}s, Tick: {res.get('avg_tick_ms',0):.2f}ms" if status == "PASS" else f"Error: {res.get('error','')}"
        print(f"{name:<20} | {status:<5} | {info}")
