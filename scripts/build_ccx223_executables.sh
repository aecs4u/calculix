#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Build CalculiX 2.23 legacy executables from `calculix_migration_tooling/ccx_2.23/src`.

Usage:
  scripts/build_ccx223_executables.sh [options]

Options:
  --variant <name>     Build variant: default|mt|i8|mfront|all (default: default)
  --src-dir <path>     Path to ccx_2.23/src directory
  --out-dir <path>     Output directory for built executables
  --jobs <n>           Parallel jobs for make (default: auto)
  --clean              Attempt clean before build
  -h, --help           Show this help

Examples:
  scripts/build_ccx223_executables.sh --variant default
  scripts/build_ccx223_executables.sh --variant all --jobs 8
  scripts/build_ccx223_executables.sh --variant i8 --src-dir /opt/CalculiX/ccx_2.23/src

Notes:
  - These builds use the original Makefiles in ccx_2.23/src:
      Makefile, Makefile_MT, Makefile_i8, Makefile_MFront
  - External dependencies (SPOOLES, ARPACK, PaStiX, etc.) must match each Makefile.
EOF
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
src_dir="${repo_root}/calculix_migration_tooling/ccx_2.23/src"
out_dir="${repo_root}/dist/ccx_2.23/bin"
variant="default"
clean=0

auto_jobs="$(getconf _NPROCESSORS_ONLN 2>/dev/null || true)"
jobs="${auto_jobs:-4}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --variant)
      variant="${2:-}"
      shift 2
      ;;
    --src-dir)
      src_dir="${2:-}"
      shift 2
      ;;
    --out-dir)
      out_dir="${2:-}"
      shift 2
      ;;
    --jobs)
      jobs="${2:-}"
      shift 2
      ;;
    --clean)
      clean=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ ! -d "${src_dir}" ]]; then
  echo "ccx source directory not found: ${src_dir}" >&2
  exit 1
fi

if ! [[ "${jobs}" =~ ^[0-9]+$ ]] || [[ "${jobs}" -lt 1 ]]; then
  echo "--jobs must be a positive integer (got: ${jobs})" >&2
  exit 2
fi

build_one() {
  local makefile="$1"
  local target="$2"

  echo "==> Building ${target} using ${makefile}"
  (
    cd "${src_dir}"
    if [[ "${clean}" -eq 1 ]]; then
      make -f "${makefile}" clean >/dev/null 2>&1 || true
    fi
    make -f "${makefile}" -j"${jobs}" "${target}"
  )

  mkdir -p "${out_dir}"
  install -m 755 "${src_dir}/${target}" "${out_dir}/${target}"
  echo "==> Installed ${target} to ${out_dir}/${target}"
}

case "${variant}" in
  default)
    build_one "Makefile" "ccx_2.23"
    ;;
  mt)
    build_one "Makefile_MT" "ccx_2.23_MT"
    ;;
  i8)
    build_one "Makefile_i8" "ccx_2.23_i8"
    ;;
  mfront)
    build_one "Makefile_MFront" "ccx_2.23_helfer"
    ;;
  all)
    build_one "Makefile" "ccx_2.23"
    build_one "Makefile_MT" "ccx_2.23_MT"
    build_one "Makefile_i8" "ccx_2.23_i8"
    build_one "Makefile_MFront" "ccx_2.23_helfer"
    ;;
  *)
    echo "Invalid --variant '${variant}'. Use: default|mt|i8|mfront|all" >&2
    exit 2
    ;;
esac

echo "Done."
