#!/usr/bin/env bash
# =============================================================================
# generate.sh — Genera imágenes para redes sociales con Typst
# =============================================================================
#
# Uso:
#   ./generate.sh                      # Genera todos los contenidos de content/
#   ./generate.sh content/quote.typ    # Genera solo quote
#   ./generate.sh --all                # Genera todo (igual que sin argumentos)
#   ./generate.sh --help               # Muestra ayuda
#
# Opciones de entorno:
#   PPI=300 ./generate.sh              # Cambiar resolución (default: 144)
#   OUTPUT_DIR=mi_carpeta ./generate.sh # Cambiar carpeta de salida
# =============================================================================

set -euo pipefail

# ─── Configuración ────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONTENT_DIR="${SCRIPT_DIR}/content"
OUTPUT_DIR="${OUTPUT_DIR:-${SCRIPT_DIR}/output}"
PPI="${PPI:-144}"

# Colores para la terminal
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No color

# ─── Funciones ────────────────────────────────────────────────────────────────

show_help() {
  echo -e "${BOLD}${CYAN}typst-rrss${NC} — Generador de imágenes para redes sociales"
  echo ""
  echo -e "${BOLD}Uso:${NC}"
  echo "  ./generate.sh                      Genera todos los contenidos"
  echo "  ./generate.sh content/quote.typ    Genera un archivo específico"
  echo "  ./generate.sh --all                Genera todo"
  echo "  ./generate.sh --help               Muestra esta ayuda"
  echo ""
  echo -e "${BOLD}Variables de entorno:${NC}"
  echo "  PPI=300 ./generate.sh              Resolución (default: 144)"
  echo "  OUTPUT_DIR=carpeta ./generate.sh    Carpeta de salida (default: output/)"
  echo ""
  echo -e "${BOLD}Plataformas soportadas:${NC}"
  echo "  Instagram Post   1080×1080   (1:1)"
  echo "  Instagram Story  1080×1920   (9:16)"
  echo "  Facebook Post    1200×630    (1.91:1)"
  echo "  Twitter/X Post   1600×900    (16:9)"
  echo "  LinkedIn Post    1200×627    (~1.91:1)"
  echo "  OG Image         1200×630    (1.91:1)"
}

compile_file() {
  local input_file="$1"
  local basename=$(basename "$input_file" .typ)
  local output_file="${OUTPUT_DIR}/${basename}.png"

  echo -ne "  ${BLUE}⟩${NC} ${basename}..."

  if typst compile --root "$SCRIPT_DIR" --ppi "$PPI" "$input_file" "$output_file" 2>/dev/null; then
    local size=$(du -h "$output_file" | cut -f1)
    echo -e " ${GREEN}✓${NC} (${size})"
    return 0
  else
    echo -e " ${RED}✗ Error${NC}"
    # Mostrar error detallado
    typst compile --root "$SCRIPT_DIR" --ppi "$PPI" "$input_file" "$output_file" 2>&1 | head -5 | sed 's/^/    /'
    return 1
  fi
}

# ─── Main ─────────────────────────────────────────────────────────────────────

main() {
  # Mostrar ayuda
  if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    show_help
    exit 0
  fi

  echo ""
  echo -e "${BOLD}${CYAN}┌─────────────────────────────────────────┐${NC}"
  echo -e "${BOLD}${CYAN}│  typst-rrss · Generador de imágenes     │${NC}"
  echo -e "${BOLD}${CYAN}└─────────────────────────────────────────┘${NC}"
  echo ""

  # Crear carpeta de salida
  mkdir -p "$OUTPUT_DIR"

  # Determinar archivos a procesar
  local files=()

  if [[ $# -eq 0 || "${1:-}" == "--all" ]]; then
    # Todos los archivos .typ en content/
    while IFS= read -r -d '' f; do
      files+=("$f")
    done < <(find "$CONTENT_DIR" -name '*.typ' -print0 | sort -z)
  else
    # Archivos específicos
    for f in "$@"; do
      if [[ -f "$f" ]]; then
        files+=("$f")
      elif [[ -f "${SCRIPT_DIR}/$f" ]]; then
        files+=("${SCRIPT_DIR}/$f")
      else
        echo -e "  ${RED}✗${NC} Archivo no encontrado: $f"
      fi
    done
  fi

  if [[ ${#files[@]} -eq 0 ]]; then
    echo -e "  ${YELLOW}⚠${NC}  No se encontraron archivos .typ en content/"
    echo "     Crea archivos en content/ o especifica uno como argumento."
    exit 1
  fi

  echo -e "  ${BOLD}PPI:${NC}     ${PPI}"
  echo -e "  ${BOLD}Output:${NC}  ${OUTPUT_DIR}/"
  echo -e "  ${BOLD}Archivos:${NC} ${#files[@]}"
  echo ""

  local success=0
  local failed=0

  for f in "${files[@]}"; do
    if compile_file "$f"; then
      ((success++)) || true
    else
      ((failed++)) || true
    fi
  done

  echo ""
  echo -e "${BOLD}Resultado:${NC} ${GREEN}${success} generadas${NC}"
  if [[ $failed -gt 0 ]]; then
    echo -e "           ${RED}${failed} con errores${NC}"
  fi
  echo -e "${BOLD}Carpeta:${NC}   ${OUTPUT_DIR}/"
  echo ""
}

main "$@"
