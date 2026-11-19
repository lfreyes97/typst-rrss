# Estado del rework (`rework-instatypst`)

Actualizado el 2026-11-11. La CLI pasó de generar layouts propios (`rrss-pkg`) a ser un compilador de presets de `instatypst`. El detalle de uso está en el `README`.

## Hecho

- Escapado único de todo lo que llega al `.typ` (`typst.rs`), verificado contra `typst` real.
- Parser de `posts.toml` estricto (`config.rs`): campos desconocidos = error, herencia con `defaults`, `por_preset` y `defs`.
- `render.rs`: un post → llamada a `carteles.<preset>`; sin `formato` manda el del preset.
- `generate` y `full` rehechos sobre el mismo camino que `build`.
- `ppi` por defecto 72 en `build`, `full` y `compile`; el `.typ` ya no se escribe en la raíz (`output/typ/`).
- `build` termina con código ≠ 0 si algún post falla; el desborde de página se explica.
- Aviso de contraste texto/fondo contra la paleta.
- Paridad comprobada: 88 láminas de `examples/presets/` de `instatypst` salen idénticas píxel a píxel.
- Retirados `rrss-pkg`, `content/`, `generate.sh`, `install-pkg.sh`, los `.typ` generados, los informes de auditoría y `tests/` (esquema viejo).
- Arreglos de UX: trazas a stderr (nivel `warn`, respeta `RUST_LOG`); `compile` termina con ≠ 0 si algo falla; avisos de Typst como `⚠`; carpetas de fuentes comprobadas; `init` solo escribe plantillas validadas de presets conocidos; flags unificados (`-o` = salida, `--solo`, `--imagen`); `extract -f json`; retirados `compile --all` / `--template` y `extract --name`.
- Renombrada la crate: `rrss-cli-rs` → `instatypst-cli` (binario `instatypst`; el `-rs` distinguía el puerto a Rust del `rrss.py` original, ya fuera del árbol).

## Pendiente

**Funcionalidad**
- Contraste contra la **foto** de fondo (hoy solo se compara con la paleta; `atardecer` salió casi ilegible sin aviso).
- `build --solo-typ`: generar los `.typ` de un TOML sin compilar (`--dry-run` solo imprime).
- `compile` no tiene `--font-path`.
- Varias páginas (`{p}`): carruseles y fondos continuos (`curvas.campo` / `ventana`, `escalera-deck`). `carrusel-test` quedó fuera de `posts.toml` por esto.
- Compilar en paralelo (17 s para 7 posts con foto; casi todo es extraer colores).
- Recoloreo de SVG: `recolor-svg` vivía en `rrss-pkg` y se perdió con él. Decidir dónde vive (aquí o en `instatypst`).

**Comprobar**
- Paridad de `cita-social`, `fleurones`, `orlas` (no se tradujeron aún).
- Test de paridad en el repo: los TOML de la comparación y el script están en un directorio temporal.

**Fuera de este repo**
- `instatypst`: `carteles.epigrafe` falla con un `tema` explícito en los estilos de la familia C (`luto`, `aura`, `pizarra`…); `_temas.typ:321` hace `str(defecto)` con `defecto = none` (commit `e52d9fe`).

**Limpieza**
- `assets/`: 9 archivos sin referencia (`Logo.svg`, `Solid-bg.svg`, `contours.svg`, una foto de 13 MB…). Decidir cuáles se quedan.
