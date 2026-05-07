# Feature Ideas

## Input / UX
- **Vi motions en Normal mode** — `w`, `b`, `0`, `$`, `gg`, `G` para navegar el scrollback sin mouse
- **Search en scrollback** ✓ — `/pattern` en Normal mode, `n`/`N` para navegar matches con highlight
- **Click derecho → menú contextual** — copiar, pegar, abrir URL bajo el cursor

## Rendering
- **Hyperlinks OSC 8** — detectar `\e]8;;url\e\\` y renderizarlos subrayados/clicables
- **Sixel / iTerm2 inline images** — renderizar imágenes directamente en el grid
- **Indicador de actividad en tabs** — punto/badge en tabs con salida nueva (útil cuando hay muchas pestañas)
- **Dim de panes inactivos** — bajar el brillo de panes que no tienen foco

## Terminal Features
- **Bracketed paste mode** — envolver paste con `\e[?2004h`
- **OSC 7 (working directory)** — mostrar CWD en la barra de estado
- **Bell visual** — flash breve al recibir BEL (`\a`) en vez de sonido

## Ergonomía
- **Session persistence** — guardar/restaurar tabs y splits al cerrar (sin el contenido del PTY, solo el layout y el CWD)
- **Zoom de pane** — `Ctrl-W z` para expandir el pane activo a pantalla completa temporalmente
- **Command palette** — overlay tipo `Ctrl-Shift-P` para ejecutar cualquier `Action` por nombre

## Config
- **Temas de color** — sección `[theme]` en el TOML con perfiles nombrados (Dracula, Gruvbox, etc.)
- **Per-tab font size persistido por nombre** — si el tab tiene nombre, recordar su font_px

---

Candidatas de mayor impacto / menor esfuerzo: **search en scrollback**, **OSC 7 en status bar**, **zoom de pane**.
