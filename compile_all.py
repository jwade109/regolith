#! /usr/bin/env python3

import sys

import os
from glob import glob

reg_files = glob("examples/*.md")

os.makedirs("songs", exist_ok=True)

if os.system("cargo build"):
    exit()

for reg in reg_files:
    print(reg)
    ret = os.system(f"./target/debug/regolith build/ --path {reg}")
    print("\n")
