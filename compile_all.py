#! /usr/bin/env python3

import sys

import os
from glob import glob

reg_files = glob("examples/*.md")

os.makedirs("songs", exist_ok=True)

for reg in reg_files:
    print(reg)
    ret = os.system(f"cargo run -- {reg}")
    if ret:
        break
    # out = os.path.join("songs", os.path.basename(reg).replace(".reg", ".mp3"))
    # if os.path.exists(out):
    #     print(out)
    #     rtime = os.path.getmtime(reg)
    #     otime = os.path.getmtime(out)
    #     if rtime < otime:
    #         print(f"(built) {out}")
    #         continue
    # print(f"Building: {reg} -> {out}")
    # compose(reg, out)
