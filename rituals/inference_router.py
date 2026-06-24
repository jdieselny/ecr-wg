"""Inference Router — Analyzes historical performance data to recommend the optimal L0/L3 public LLM for a given task shape."""
import json
import os

def load_datums(path="routing/datums.jsonl"):
    datums = []
    if os.path.exists(path):
        with open(path, 'r', encoding='utf-8') as f:
            for line in f:
                try:
                    datums.append(json.loads(line))
                except:
                    continue
    return datums

def load_intent_table(path="rituals/intent_table.json"):
    if os.path.exists(path):
        with open(path, 'r', encoding='utf-8') as f:
            return json.load(f)
    return {}

def get_task_shape(intent_key, registry):
    # Recursively find the task_shape for a canonical_key
    if isinstance(registry, dict):
        if registry.get("canonical_key") == intent_key:
            return registry.get("task_shape")
        for v in registry.values():
            shape = get_task_shape(intent_key, v)
            if shape:
                return shape
    return None

def route(intent_key):
    print(f"\n---[ COSA // INFERENCE ROUTER ]---")
    print(f"[Analyzing L0/L3 Route for: {intent_key}]")
    
    intent_table = load_intent_table()
    task_shape = get_task_shape(intent_key, intent_table.get("LATTICE_REGISTRY", {}))
    
    if not task_shape:
        task_shape = "general_query"
        print(f"[WARN: INTENT NOT IN LATTICE. DEFAULTING TO SHAPE: {task_shape}]")
    else:
        print(f"[TASK_SHAPE: {task_shape}]")
        
    datums = load_datums()
    stats = {} # body: {pass: 0, fail: 0, total: 0, notes: []}
    
    for d in datums:
        if d.get("task_shape") == task_shape:
            body = d.get("body")
            outcome = d.get("outcome", "unknown")
            
            if body not in stats:
                stats[body] = {"pass": 0, "fail": 0, "total": 0, "notes": []}
                
            stats[body]["total"] += 1
            if "pass" in outcome:
                stats[body]["pass"] += 1
            elif "fail" in outcome:
                stats[body]["fail"] += 1
                
            if d.get("notes"):
                stats[body]["notes"].append(d["notes"])
                
    if not stats:
        print("[NO PERFORMANCE DATA FOR THIS SHAPE. RECOMMENDING BALANCED DEFAULT: llama3:8b]")
        return {"body": "llama3:8b", "reason": "No historical datums for this shape."}
        
    # Rank by Success Rate
    ranked = []
    for body, s in stats.items():
        rate = s["pass"] / s["total"] if s["total"] > 0 else 0
        ranked.append({"body": body, "rate": rate, "total": s["total"], "notes": s["notes"]})
        
    ranked.sort(key=lambda x: (x["rate"], x["total"]), reverse=True)
    best = ranked[0]
    
    print(f"\n[STAMPED RECOMMENDATION]")
    print(f"MODEL: {best['body']}")
    print(f"CONFIDENCE: {int(best['rate'] * 100)}% (based on {best['total']} datums)")
    
    if best['notes']:
        print(f"OPERATOR NOTES:")
        for note in best['notes'][-2:]: # Last 2 notes
            print(f"- {note}")
            
    return best

if __name__ == "__main__":
    import sys
    key = sys.argv[1] if len(sys.argv) > 1 else "UPHOLSTERY::REPAIR::STITCHING"
    route(key)
