#!/bin/bash

# Script para instalar el paquete de Typst localmente
# Lee la información directamente de typst.toml

set -e

PKG_DIR="rrss-pkg"
VERSION=$(grep -m 1 "^version =" "$PKG_DIR/typst.toml" | cut -d '"' -f 2)
NAME=$(grep -m 1 "^name =" "$PKG_DIR/typst.toml" | cut -d '"' -f 2)

if [ -z "$VERSION" ] || [ -z "$NAME" ]; then
    echo "❌ Error: No se pudo encontrar el nombre o la versión en $PKG_DIR/typst.toml"
    exit 1
fi

TARGET_DIR="$HOME/.local/share/typst/packages/local/$NAME/$VERSION"

echo "📦 Instalando paquete '$NAME' versión '$VERSION'..."
echo "📂 Destino: $TARGET_DIR"

# Limpiar si ya existe
if [ -d "$TARGET_DIR" ] || [ -L "$TARGET_DIR" ]; then
    echo "🧹 Limpiando instalación previa..."
    rm -rf "$TARGET_DIR"
fi

# Crear directorio base
mkdir -p "$HOME/.local/share/typst/packages/local/$NAME"

# Crear enlace simbólico (preferible para desarrollo)
ln -s "$(pwd)/$PKG_DIR" "$TARGET_DIR"

echo "✅ Paquete instalado correctamente."
echo "💡 Ahora puedes usarlo en Typst como: #import \"@local/$NAME:$VERSION\": *"
