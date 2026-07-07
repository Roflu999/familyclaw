#!/usr/bin/env python3
"""Generate placeholder icons for Tauri bundle."""
from PIL import Image, ImageDraw, ImageFont
import os

ICONS_DIR = "src-tauri/icons"
os.makedirs(ICONS_DIR, exist_ok=True)

# Create a 1024x1024 base icon
size = 1024
img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
draw = ImageDraw.Draw(img)

# Background circle
cx, cy = size // 2, size // 2
radius = size // 2 - 40
draw.ellipse([cx - radius, cy - radius, cx + radius, cy + radius], fill=(59, 130, 246, 255))

# Inner lighter circle
inner_r = radius - 60
draw.ellipse([cx - inner_r, cy - inner_r, cx + inner_r, cy + inner_r], fill=(96, 165, 250, 255))

# Claw/shell symbol (simple triangle/arrow shape)
points = [
    (cx, cy - inner_r + 120),
    (cx - inner_r + 200, cy + inner_r - 120),
    (cx, cy + inner_r - 200),
    (cx + inner_r - 200, cy + inner_r - 120),
]
draw.polygon(points, fill=(255, 255, 255, 255))

# Save base
base_path = os.path.join(ICONS_DIR, "icon.png")
img.save(base_path)
print(f"Saved {base_path}")

# Generate standard sizes
sizes = [32, 128, 256, 512]
for s in sizes:
    resized = img.resize((s, s), Image.LANCZOS)
    if s == 128:
        # Retina
        resized2x = img.resize((256, 256), Image.LANCZOS)
        resized2x.save(os.path.join(ICONS_DIR, "128x128@2x.png"))
    resized.save(os.path.join(ICONS_DIR, f"{s}x{s}.png"))
    print(f"Saved {s}x{s}.png")

# ICO for Windows
img.save(os.path.join(ICONS_DIR, "icon.ico"), sizes=[(32, 32), (64, 64), (128, 128), (256, 256)])
print(f"Saved icon.ico")

# ICNS for macOS (simplified - just copy PNG)
img.save(os.path.join(ICONS_DIR, "icon.icns"))
print(f"Saved icon.icns (PNG fallback)")

print("\nDone! Run this script from the project root:")
print("  python3 scripts/generate-icons.py")
