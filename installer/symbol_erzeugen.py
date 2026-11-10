"""Erzeugt die Programm-Symbole (PNG und ICO) aus der Geometrie von branding/simplegrid-master.svg.

Aufruf (im Repository):  python installer/symbol_erzeugen.py
Braucht nur Pillow. Gezeichnet wird 8-fach vergrößert und dann verkleinert (glatte Kanten).
"""
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw

WURZEL = Path(__file__).resolve().parent.parent
ZIEL = WURZEL / "assets" / "app-icon"
S = 8  # Überabtastung


def symbol(groesse: int) -> Image.Image:
    """Das Symbol quadratisch (viewBox 512×452, senkrecht mittig in 512×512), transparent."""
    n = 512 * S
    dy = 30  # wie favicon.svg: translate(0 30)

    def p(x, y):
        return (x * S, (y + dy) * S)

    # Verlauf #18C58A → #078653 diagonal.
    a, b = (0x18, 0xC5, 0x8A), (0x07, 0x86, 0x53)
    verlauf = Image.new("RGBA", (n, n))
    px = verlauf.load()
    schritt = 4
    for y in range(0, n, schritt):
        for x in range(0, n, schritt):
            t = min(1.0, max(0.0, (x + y) / (2 * n)))
            c = tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3)) + (255,)
            for yy in range(y, min(y + schritt, n)):
                for xx in range(x, min(x + schritt, n)):
                    px[xx, yy] = c

    # Blatt: abgerundetes Rechteck ohne die Ecke oben rechts.
    maske = Image.new("L", (n, n), 0)
    d = ImageDraw.Draw(maske)
    d.rounded_rectangle([p(81, 38), p(452, 414)], radius=24 * S, fill=255)
    ecke = Image.new("L", (n, n), 0)
    ImageDraw.Draw(ecke).polygon([p(335, 38), p(460, 38), p(460, 156), p(452, 156)], fill=255)
    maske = ImageChops.subtract(maske, ecke)

    bild = Image.new("RGBA", (n, n), (0, 0, 0, 0))
    bild.paste(verlauf, (0, 0), maske)
    zeichnen = ImageDraw.Draw(bild)
    # Eselsohr.
    zeichnen.polygon([p(335, 38), p(335, 106), p(342, 120), p(354, 125), p(452, 125), p(452, 156)], fill=(0x6E, 0xE7, 0xB7, 255))
    # 3×3 weiße Felder.
    for zeile in range(3):
        for spalte in range(3):
            x = 139 + spalte * 90
            y = 166 + zeile * 64
            zeichnen.rounded_rectangle([p(x, y), p(x + 78, y + 52)], radius=3 * S, fill=(255, 255, 255, 255))
    return bild.resize((groesse, groesse), Image.LANCZOS)


def main():
    gross = symbol(1024)
    gross.save(ZIEL / "simplegrid-1024.png")
    symbol(512).save(ZIEL / "simplegrid-macos-512.png")
    for g in (16, 24, 32, 48, 64, 128, 256, 512):
        ordner = ZIEL / "hicolor" / f"{g}x{g}" / "apps"
        ordner.mkdir(parents=True, exist_ok=True)
        symbol(g).save(ordner / "simplegrid.png")
    gross.save(ZIEL / "simplegrid.ico", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    print("Symbole geschrieben nach", ZIEL)


if __name__ == "__main__":
    main()
