from PIL import ImageGrab
n = 0
for x_start in range(1715, 3307, 203):
    for y_start in range(442, 821, 54):
        x_end = x_start + 62
        y_end = y_start + 36

        img = ImageGrab.grab(bbox=(x_start, y_start, x_end, y_end))
        img.save(f"templates/{n}.png")

        n += 1