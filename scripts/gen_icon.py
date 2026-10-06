"""在 CI 中生成 DeskSort 应用图标（1024x1024 PNG）。
用法：python scripts/gen_icon.py
"""
import os

from PIL import Image, ImageDraw


def main():
    os.makedirs("src-tauri/icons", exist_ok=True)
    s = 1024
    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    base = Image.new("RGBA", (s, s), (0, 0, 0, 0))

    grad = Image.new("RGBA", (s, s))
    gd = ImageDraw.Draw(grad)
    for y in range(s):
        t = y / s
        r = int(0x5B + (0x7C - 0x5B) * t)
        g = int(0x8C + (0x5B - 0x8C) * t)
        gd.line([(0, y), (s, y)], fill=(r, g, 0xFF, 255))
    mask = Image.new("L", (s, s), 0)
    ImageDraw.Draw(mask).rounded_rectangle([16, 16, s - 16, s - 16], radius=220, fill=255)
    base.paste(grad, (0, 0), mask)

    d = ImageDraw.Draw(base)
    white = (255, 255, 255, 235)
    sw, sh = 300, 300
    d.rounded_rectangle([180, 190, 180 + sw, 190 + sh], radius=70, fill=white)
    d.rounded_rectangle([560, 190, 560 + sw, 190 + sh], radius=70, fill=white)
    d.rounded_rectangle([180, 560, 180 + sw, 560 + sh], radius=70, fill=white)
    for i in range(3):
        y = 560 + i * 115
        d.rounded_rectangle([560, y, 560 + sw, y + 85], radius=42, fill=white)

    out = Image.alpha_composite(img, base)
    out.save("src-tauri/icons/icon.png")
    print("图标已生成：src-tauri/icons/icon.png")


if __name__ == "__main__":
    main()
