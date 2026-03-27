#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
FLUTTER_DIR="$ROOT_DIR/flutter_web"

# Step 1: wasm-pack build → flutter_web/web/wasm/
cd "$SCRIPT_DIR"
wasm-pack build --target web --out-dir "$FLUTTER_DIR/web/wasm" --out-name road_drawing_wasm
echo "✓ WASM built → flutter_web/web/wasm/"

# --wasm-only: stop after WASM build (default behavior for dev)
if [ "$1" = "--wasm-only" ]; then
  exit 0
fi

# Step 2: flutter build web
if [ "$1" = "--serve" ] || [ "$1" = "--full" ]; then
  FLUTTER_BIN="${FLUTTER_SDK:-$HOME/flutter}/bin/flutter"
  cd "$FLUTTER_DIR"
  "$FLUTTER_BIN" build web
  echo "✓ Flutter built → flutter_web/build/web/"

  # Step 3: Copy WASM to build output
  mkdir -p "$FLUTTER_DIR/build/web/wasm"
  cp "$FLUTTER_DIR/web/wasm/road_drawing_wasm.js" "$FLUTTER_DIR/build/web/wasm/"
  cp "$FLUTTER_DIR/web/wasm/road_drawing_wasm_bg.wasm" "$FLUTTER_DIR/build/web/wasm/"
  echo "✓ WASM copied → flutter_web/build/web/wasm/"
fi

# Step 4: Serve with python http.server
if [ "$1" = "--serve" ]; then
  echo ""
  echo "Serving at http://localhost:8080"
  echo "Press Ctrl+C to stop"
  cd "$FLUTTER_DIR/build/web"
  python -m http.server 8080
fi
