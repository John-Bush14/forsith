
import struct
import zlib
import sys

with open(sys.argv[1], "rb") as f:
    assert f.read(8) == b"\x89PNG\r\n\x1a\n"

    while True:
        length_bytes = f.read(4)
        if not length_bytes:
            break

        length = struct.unpack(">I", length_bytes)[0]
        chunk_type = f.read(4)
        data = f.read(length)
        stored_crc = struct.unpack(">I", f.read(4))[0]

        calculated_crc = zlib.crc32(chunk_type + data)

        status = "OK" if stored_crc == calculated_crc else "MISMATCH"

        print(
            f"{chunk_type.decode('ascii'):4} "
            f"len={length:8} "
            f"stored={stored_crc:08X} "
            f"calculated={calculated_crc:08X} "
            f"{status}"
        )

        if chunk_type == b"IEND":
            break
