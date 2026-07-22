import struct, os, zlib

d = os.path.join(os.path.dirname(__file__), 'icons')
os.makedirs(d, exist_ok=True)

def create_png(path, size):
    rgba = bytearray()
    center_x, center_y = size // 2, size // 2
    radius = size // 2 - 1
    inner = size // 4
    for y in range(size):
        for x in range(size):
            dx, dy = x - center_x, y - center_y
            dist = (dx*dx + dy*dy) ** 0.5
            if dist <= inner:
                rgba.extend([212, 175, 55, 255])
            elif dist <= radius:
                rgba.extend([76, 29, 149, 255])
            else:
                rgba.extend([0, 0, 0, 0])
    sig = b'\x89PNG\r\n\x1a\n'
    def chunk(ctype, data):
        c = ctype + data
        crc = zlib.crc32(c) & 0xffffffff
        return struct.pack('>I', len(data)) + c + struct.pack('>I', crc)
    ihdr = struct.pack('>IIBBBBB', size, size, 8, 6, 0, 0, 0)
    raw = b''
    for y in range(size):
        raw += b'\x00' + bytes(rgba[y*size*4:(y+1)*size*4])
    compressed = zlib.compress(raw)
    with open(path, 'wb') as f:
        f.write(sig + chunk(b'IHDR', ihdr) + chunk(b'IDAT', compressed) + chunk(b'IEND', b''))

def create_ico(path, png_path):
    with open(png_path, 'rb') as f:
        png_data = f.read()
    ico_header = struct.pack('<HHH', 0, 1, 1)
    ico_entry = struct.pack('<BBBBHHII', 0, 0, 0, 0, 1, 32, len(png_data), 22)
    with open(path, 'wb') as f:
        f.write(ico_header + ico_entry + png_data)

# Create all sizes
sizes = [(32, '32x32.png'), (128, '128x128.png'), (256, '128x128@2x.png')]
for sz, name in sizes:
    create_png(os.path.join(d, name), sz)

create_ico(os.path.join(d, 'icon.ico'), os.path.join(d, '128x128.png'))
create_ico(os.path.join(d, 'icon.icns'), os.path.join(d, '128x128.png'))

print('All icons created')
