#!/usr/bin/env python3
"""Photo-like fixtures: smooth gradients plus noise so codecs do real work."""
import os, sys
import numpy as np
from PIL import Image

out = os.path.join(os.path.dirname(__file__), "fixtures")
os.makedirs(out, exist_ok=True)
rng = np.random.default_rng(1)

def photo(w, h):
    y, x = np.mgrid[0:h, 0:w].astype(np.float32)
    r = 128 + 100 * np.sin(x / w * 6.3) * np.cos(y / h * 4.1)
    g = 128 + 100 * np.sin((x + y) / (w + h) * 9.0)
    b = 128 + 100 * np.cos(x / w * 3.0 + y / h * 5.0)
    img = np.stack([r, g, b], -1) + rng.normal(0, 12, (h, w, 3))
    return Image.fromarray(np.clip(img, 0, 255).astype(np.uint8))

for name, (w, h) in {"1mp": (1280, 800), "4mp": (2400, 1600), "12mp": (4000, 3000)}.items():
    photo(w, h).save(f"{out}/{name}.jpg", quality=90)
photo(2400, 1600).save(f"{out}/4mp.png", compress_level=6)
photo(2400, 1600).save(f"{out}/4mp.webp", quality=90)
for f in sorted(os.listdir(out)):
    print(f, os.path.getsize(f"{out}/{f}"))
