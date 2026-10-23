import png
from sys import argv
import json

r = png.Reader(filename=argv[1])
(x, y, data, meta) = r.read()

print(meta)

data = list(b for row in data for b in row)

with open(argv[2], 'w') as out:
    out.write(json.dumps(data))
