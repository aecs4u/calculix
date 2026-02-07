#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if ! command -v gfortran >/dev/null 2>&1; then
  echo "gfortran is required to build ccx coverage binaries" >&2
  exit 1
fi

mkdir -p build/coverage

echo "[1/3] Cleaning previous artifacts"
find ccx_f -maxdepth 1 -type f \( -name '*.o' -o -name '*.gcno' -o -name '*.gcda' -o -name 'ccx_2.23' -o -name 'ccx_2.23.a' \) -delete
find cgx_c/src -maxdepth 1 -type f \( -name '*.o' -o -name '*.gcno' -o -name '*.gcda' -o -name 'cgx' \) -delete

echo "[2/3] Building ccx with coverage flags"
make -C ccx_f \
  CFLAGS='-O0 -g --coverage -fprofile-arcs -ftest-coverage -I ../../../SPOOLES.2.2 -DARCH="Linux" -DSPOOLES -DARPACK -DMATRIXSTORAGE -DNETWORKOUT' \
  FFLAGS='-O0 -g --coverage -fprofile-arcs -ftest-coverage -cpp' \
  ccx_2.23

echo "[3/3] Building cgx with coverage flags"
make -C cgx_c/src \
  CFLAGS='-O0 -g --coverage -fprofile-arcs -ftest-coverage -Wno-narrowing -DSEMINIT -I./ -I/usr/include -I/usr/include/GL -I../../libSNL/src -I../../glut-3.5/src -I/usr/X11/include' \
  LFLAGS='--coverage -L/usr/lib64 -lGL -lGLU -L/usr/X11R6/lib64 -lX11 -lXi -lXmu -lXext -lXt -lSM -lICE -lm -lpthread -lrt' \
  cgx

echo "Coverage binaries ready:"
echo "  - $ROOT_DIR/ccx_f/ccx_2.23"
echo "  - $ROOT_DIR/cgx_c/src/cgx"
