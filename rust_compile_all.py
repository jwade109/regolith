import os
from glob import glob

reg_files = glob("examples/*.reg")

os.makedirs("rust_songs", exist_ok=True)

for reg in reg_files:
    out = os.path.join("rust_songs", os.path.basename(reg).replace(".reg", ".mp3"))
    # if os.path.exists(out):
    #     rtime = os.path.getmtime(reg)
    #     otime = os.path.getmtime(out)
    #     if rtime < otime:
    #         print(f"(built) {out}")
    #         continue
    print(f"Building: {reg} -> {out}")
    if os.system(f"cargo run --bin regolith -- {reg} {out}"):
        break
