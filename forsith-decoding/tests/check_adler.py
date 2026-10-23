
import struct
import zlib
import sys

with open(sys.argv[1], "rb") as f:
    assert f.read(8) == b"\x89PNG\r\n\x1a\n"

    idat = bytearray()

    while True:
        length_bytes = f.read(4)
        if not length_bytes:
            break

        length = struct.unpack(">I", length_bytes)[0]
        chunk_type = f.read(4)
        data = f.read(length)
        f.read(4)  # Skip PNG chunk CRC

        if chunk_type == b"IDAT":
            idat.extend(data)
        elif chunk_type == b"IEND":
            break

# The final 4 bytes of the zlib stream contain the big-endian Adler-32.
stored_adler = struct.unpack(">I", idat[-4:])[0]

# Decompress the zlib stream, including validation of its Adler-32.
decompressed = zlib.decompress(idat)
calculated_adler = zlib.adler32(decompressed) & 0xFFFFFFFF

print(f"Stored Adler-32:     {stored_adler:08X}")
print(f"Calculated Adler-32: {calculated_adler:08X}")
print(f"Status: {'OK' if stored_adler == calculated_adler else 'MISMATCH'}")
print(f"Decompressed bytes: {len(decompressed)}")

