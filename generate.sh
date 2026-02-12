#!/usr/bin/env bash
# =============================================================================
# generate.sh — Genera imágenes para redes sociales con rrss-cli-rs
# =============================================================================
#
# Wrapper para la nueva CLI en Rust.
# Usa ./rrss (que apunta al binario compilado) para compilar.
#
# Uso:
#   ./generate.sh                      # Genera todo (implica --all)
#   ./generate.sh content/quote.typ    # Genera archivo específico
#   ./generate.sh --all                # Genera todo explícitamente
#

set -e

# Asegurar que el binario de Rust esté construido
if [ ! -f "./rrss-cli-rs/target/release/rrss-cli-rs" ]; then
    echo "Construyendo CLI de Rust..."
    (cd rrss-cli-rs && cargo build --release)
fi

CMD_ARGS=()

# Mapear variables de entorno a argumentos de CLI
if [ ! -z "${PPI}" ]; then
    CMD_ARGS+=("--ppi" "${PPI}")
fi

if [ ! -z "${OUTPUT_DIR}" ]; then
    CMD_ARGS+=("--output-dir" "${OUTPUT_DIR}")
fi

# Agregar argumentos pasados
CMD_ARGS+=("$@")

# Default a --all si no se pasan argumentos de archivo/opción
HAS_ARGS=0
for arg in "$@"; do
    if [[ "$arg" != -* ]]; then
        HAS_ARGS=1
    fi
    if [[ "$arg" == "--all" ]]; then
        HAS_ARGS=1
    fi
done

if [ $HAS_ARGS -eq 0 ]; then
    CMD_ARGS+=("--all")
fi

# Ejecutar mediante el wrapper ./rrss
echo "Running: ./rrss compile ${CMD_ARGS[@]}"
./rrss compile "${CMD_ARGS[@]}"
