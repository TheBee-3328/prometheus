# B6 — ORCH ORDER
Best slice still ~−3KB after 80+ trials (gate −15KB). Finish the **current** hyp batch only.
Then **REJECT**: write B6_REJECT.md, restore WS off / main-equivalent, commit+push agent/b6-ws, stop.
Do **not** start another large sweep. Orch will run enwik9 on main best next.
