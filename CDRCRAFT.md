# CDRCraft

CDRCraft 2 es una adaptación de **TraceDraw** (https://github.com/tracedraw/tracedraw),
editor vectorial de código abierto en Rust, con licencia MIT o Apache-2.0
(ver `LICENSE-MIT`, `LICENSE-APACHE`, `CONTRIBUTORS.md` y `ATTRIBUTION.md`, que se conservan sin cambios).

Cambios de CDRCraft sobre TraceDraw (rama `cdrcraft`):
- Nombre CDRCraft en la interfaz (todos los idiomas), título de ventana, id de aplicación `cdrcraft` y lanzador `packaging/linux/cdrcraft.desktop`.
- Logo propio (16/11/2026): "C" roja Budapest dibujada como curva con nodos y tirador, sobre fondo noche. Fuentes en `assets/logo` (ícono SVG/PNG y logo horizontal claro/oscuro con Montserrat); los PNG de `assets/icon` salen de ahí. La ventana pasa su propio ícono (`ui/brand.rs`, `window_icon`) para que KDE no muestre uno viejo de caché.
- Paleta integrada "Budapest" con los colores de marca.
- Actualizaciones propias (21/11/2026): Ayuda > Actualizaciones y la búsqueda diaria al abrir miran las versiones de github.com/PajermanPy/cdrcraft; una copia instalada desde el .tar.gz (o el zip portátil de Windows) se reemplaza sola con el botón Instalar, comprobando el SHA-256 que da GitHub.
- El texto "Acerca de" cita a TraceDraw y su licencia.
- Guarda .cdr en versión 26 por defecto (CorelDRAW 2025 no abre la 27, que es la predeterminada de TraceDraw). En Guardar como se puede elegir otra.
- Herramienta de línea de comandos instalada como `cdrcraft-cli` (convertir, inspeccionar, miniaturas).
- Pruebas: correrlas con `LANG=C.UTF-8 LC_ALL=C.UTF-8 cargo test --release -p tracedraw`; con el sistema en español muchas fallan porque esperan textos en inglés.

Para traer mejoras nuevas de TraceDraw: `git fetch upstream && git merge upstream/main` en la rama `cdrcraft`.

CDRCraft y TraceDraw son proyectos independientes, sin relación con Corel Corporation ni Alludo. CorelDRAW es marca registrada de Corel Corporation.

Más cambios (16/11/2026):
- Nombre CDRCraft (sin "Corel", por la marca registrada), id `cdrcraft`, carpeta de ajustes `cdrcraft`.
- Informes de fallos automáticos con permiso (`apps/tracedraw/src/crash_report.rs`, protocolo Sentry). Prueba de punta a punta: `CDRCRAFT_TEST_CRASH=1` cierra el programa a los 6 s a propósito.
- Enlaces de "Informar un problema" y "Debates" al repositorio de CDRCraft (`ui/brand.rs`, `REPO_URL`).
- Las pruebas corren en inglés aunque el sistema esté en otro idioma (`system_locale.rs`).
- Las tareas automáticas de GitHub de TraceDraw quedan apagadas en `.github/workflows-tracedraw` (GitHub solo ejecuta `.github/workflows`).

Cambios del 21/11/2026:
- Versión propia en `ui/brand.rs` (`VERSION`, por ejemplo `0.47.0-cdrcraft.3`). La etiqueta de la versión en GitHub tiene que ser `v` más ese texto: la tarea de GitHub lo comprueba. Para publicar: subir `VERSION`, commit, `git tag v<VERSION>` y `git push origin v<VERSION>`.
- Con el driver de NVIDIA en una sesión Wayland la ventana se dibuja con OpenGL (`gpu.rs`, `gl_first_here`): ahí wgpu/Vulkan copiaba cada imagen del lienzo a decenas de MB/s y el zoom daba un cuadro cada 0,8 a 2 s; con OpenGL, uno cada 24 ms. `CDRCRAFT_NO_GL_FIRST=1` vuelve a wgpu. El lanzador ya no fuerza XWayland (el OpenGL de NVIDIA no dibuja en ventanas de XWayland).
- Pruebas más firmes: la del portapapeles X11 usa un Xvfb propio (antes usaba el escritorio y Klipper le robaba la imagen), la del calco en reposo espera 10 cuadros quietos, la de franjas acepta máquinas con muchos núcleos y las de hindi se saltean si hay otra versión de Noto Sans Devanagari instalada.
- GitHub arma y prueba Linux y Windows en cada envío a `main`: convierte un .cdr y abre el programa 20 s.
