#!/usr/bin/env python3
"""Micro-search around ORDER2_RESCALE=110 winner. Isolated worktree safe."""
from __future__ import annotations
import csv, json, pathlib, subprocess, sys, time
ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tune"))
import search as S  # noqa: E402

BASE = {
    "ORDER2_RESCALE": 110,
    "BUF_BITS": 25,
    "HASH_BITS": 26,
    "HASH_ORDER": 6,
    "MIN_MIX_LEN": 6,
    "MATCH_CONF": [32700, 32700, 32700, 32700, 32700],
    "MIX_WEIGHT": [100, 160, 200, 230, 245],
}

def cands():
    out = []
    # Fine rescale
    for r in range(100, 121):
        q = S.copy_params(BASE); q["ORDER2_RESCALE"] = r
        out.append((f"micro:res{r}", q))
    # Mix with res=110
    for m in [
        [100, 160, 200, 230, 245],
        [100, 160, 200, 230, 250],
        [100, 160, 200, 235, 245],
        [100, 165, 200, 230, 245],
        [95, 155, 195, 225, 245],
        [105, 165, 205, 235, 248],
        [110, 170, 210, 235, 248],
        [90, 150, 200, 230, 245],
        [100, 160, 210, 230, 245],
        [100, 150, 190, 230, 245],
        [80, 160, 200, 230, 245],
        [120, 160, 200, 230, 245],
        [100, 160, 200, 220, 240],
        [100, 160, 200, 240, 250],
    ]:
        q = S.copy_params(BASE); q["MIX_WEIGHT"] = m
        out.append((f"micro:mix{'-'.join(map(str,m))}", q))
    # Conf short with res=110
    for c0, c1 in [(32700,32700),(32000,32700),(31000,32700),(30000,32000),
                   (28000,31000),(26000,30000),(24000,28000),(32500,32700),
                   (32700,32500),(31500,32500)]:
        q = S.copy_params(BASE); q["MATCH_CONF"] = [c0,c1,32700,32700,32700]
        out.append((f"micro:conf{c0}-{c1}", q))
    # structure
    for ho,mml in [(6,5),(6,6),(6,7),(5,5),(5,6),(7,6)]:
        q = S.copy_params(BASE); q["HASH_ORDER"]=ho; q["MIN_MIX_LEN"]=mml
        out.append((f"micro:ho{ho}_m{mml}", q))
    # joint res x slight mix
    for r in (108,110,112,114):
        for m0 in (95,100,105,110):
            q = S.copy_params(BASE); q["ORDER2_RESCALE"]=r
            q["MIX_WEIGHT"]=[m0,160,200,230,245]
            out.append((f"micro:j_r{r}_m0{m0}", q))
    # BUF/HASH near
    for b,h in [(25,26),(26,25),(26,26),(25,25)]:
        q = S.copy_params(BASE); q["BUF_BITS"]=b; q["HASH_BITS"]=h
        if S.ram_ok(q):
            out.append((f"micro:b{b}_h{h}", q))
    seen=set(); uniq=[]
    for n,p in out:
        k=json.dumps(p,sort_keys=True)
        if k in seen: continue
        seen.add(k); uniq.append((n,p))
    return uniq

def main():
    minutes=float(sys.argv[1]) if len(sys.argv)>1 else 14.0
    S.ensure_header(True)
    t0=time.time(); deadline=t0+minutes*60
    best_p=S.copy_params(BASE); S.apply_params(best_p)
    best_size=S.eval_slice(); trial=1
    S.log_trial(trial,t0,best_size,best_size,best_p,"micro:baseline_res110")
    S.BEST_JSON.write_text(json.dumps(best_p,indent=2))
    print(f"baseline slice={best_size}", flush=True)
    evaluated={json.dumps(best_p,sort_keys=True)}
    for note,cand in cands():
        if time.time()>=deadline: break
        k=json.dumps(cand,sort_keys=True)
        if k in evaluated: continue
        evaluated.add(k)
        S.apply_params(cand)
        try:
            size=S.eval_slice()
        except subprocess.CalledProcessError as e:
            print(f"fail {e}", flush=True); continue
        trial+=1
        S.log_trial(trial,t0,size,best_size,cand,note)
        mark=""
        if size<best_size:
            best_size=size; best_p=S.copy_params(cand)
            S.BEST_JSON.write_text(json.dumps(best_p,indent=2)); mark=" *NEW BEST*"
        print(f"t{trial} {size} (best {best_size}) {note}{mark}", flush=True)
    S.apply_params(best_p)
    print(f"DONE trials={trial} best_slice={best_size}", flush=True)
    print(json.dumps(best_p), flush=True)

if __name__=="__main__":
    main()
