import sys
sys.path.append("compiler/")

import os
from regolith import compose
from glob import glob

reg_files = glob("examples/*.reg")

os.makedirs("songs", exist_ok=True)

for reg in reg_files:
    out = os.path.join("songs", os.path.basename(reg).replace(".reg", ".mp3"))
    if os.path.exists(out):
        rtime = os.path.getmtime(reg)
        otime = os.path.getmtime(out)
        if rtime < otime:
            print(f"(built) {out}")
            continue
    print(f"Building: {reg} -> {out}")
    compose(reg, out)
