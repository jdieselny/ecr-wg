"""
Cognitive Query Dispatcher — The Orchestrator for the COSA Resolution Model.
Implements the 'Resolution Gate' logic (L1/L2) to minimize GPU (L3) waste.
"""
import sys
import json
from rituals.normalize_intent import normalize
from rituals.inference_router import route, get_task_shape, load_intent_table
from rituals.solve_with_broadcast import solve as solve_broadcast

def dispatch(vomit_prompt):
    print("\n" + "="*60)
    print("---[ COSA // COGNITIVE QUERY DISPATCHER ]---")
    print("="*60)
    
    # 1. LAYER-0: NORMALIZE INTENT
    intent_key = normalize(vomit_prompt)
    print(f"[INTENT_NORMALIZED: {intent_key}]")
    
    # 2. LAYER-1/2: RESOLUTION GATE
    intent_table = load_intent_table()
    task_shape = get_task_shape(intent_key, intent_table.get("LATTICE_REGISTRY", {}))
    
    if task_shape == "zero_shot_broadcast":
        print("\n[RESOLUTION_GATE: MATCH FOUND // BROADCAST TIER]")
        print("[BYPASSING GPU INFERENCE...]")
        resolution = solve_broadcast(intent_key, vomit_prompt)
        _display_resolution(resolution)
        return
        
    # 3. LAYER-3: INFERENCE ROUTING (If not zero-shot)
    print("\n[RESOLUTION_GATE: NO LOCAL MATCH // ROUTING TO INFERENCE]")
    recommendation = route(intent_key)
    
    print(f"\n[NEXT_STEP: Invoke execution on {recommendation['body']}]")
    print("[NOTE: L3 solvers not auto-triggered in this prototype.]")

def _display_resolution(cogobj):
    if "error" in cogobj:
        print(f"\n[ERROR: {cogobj['error']}]")
        return
        
    print("\n" + "-"*40)
    print(f"RESOLUTION (COGOBJ: {cogobj['id']})")
    print("-"*40)
    for line in cogobj['data']['resolution']:
        try:
            print(line)
        except UnicodeEncodeError:
            encoding = sys.stdout.encoding or 'utf-8'
            print(line.encode(encoding, errors='replace').decode(encoding))
    print("-"*40)
    print(f"METRICS: {cogobj['metrics']['total_token_burn']} tokens // {cogobj['metrics']['latency_seconds']}s")
    print(f"PROVENANCE: {cogobj['provenance']['model']} ({cogobj['provenance']['provenance_tier']})")
    print("-"*40)

if __name__ == "__main__":
    if len(sys.argv) > 1:
        dispatch(" ".join(sys.argv[1:]))
    else:
        prompt = input("Enter your query: ")
        dispatch(prompt)
