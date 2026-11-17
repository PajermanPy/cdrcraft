# CorelCraft 2

CorelCraft 2 es una adaptación de **TraceDraw** (https://github.com/tracedraw/tracedraw),
editor vectorial de código abierto en Rust, con licencia MIT o Apache-2.0
(ver `LICENSE-MIT`, `LICENSE-APACHE`, `CONTRIBUTORS.md` y `ATTRIBUTION.md`, que se conservan sin cambios).

Cambios de CorelCraft sobre TraceDraw (rama `corelcraft`):
- Nombre CorelCraft en la interfaz (todos los idiomas), título de ventana, id de aplicación `corelcraft` y lanzador `packaging/linux/corelcraft.desktop`.
- Logo propio (16/11/2026): "C" roja Budapest dibujada como curva con nodos y tirador, sobre fondo noche. Fuentes en `assets/logo` (ícono SVG/PNG y logo horizontal claro/oscuro con Montserrat); los PNG de `assets/icon` salen de ahí. La ventana pasa su propio ícono (`ui/brand.rs`, `window_icon`) para que KDE no muestre uno viejo de caché.
- Paleta integrada "Budapest" con los colores de marca.
- Búsqueda de actualizaciones desactivada por defecto (las actualizaciones de TraceDraw reemplazarían a CorelCraft).
- El texto "Acerca de" cita a TraceDraw y su licencia.
- Guarda .cdr en versión 26 por defecto (CorelDRAW 2025 no abre la 27, que es la predeterminada de TraceDraw). En Guardar como se puede elegir otra.
- Herramienta de línea de comandos instalada como `corelcraft-cli` (convertir, inspeccionar, miniaturas).
- Pruebas: correrlas con `LANG=C.UTF-8 LC_ALL=C.UTF-8 cargo test --release -p tracedraw`; con el sistema en español muchas fallan porque esperan textos en inglés.

Para traer mejoras nuevas de TraceDraw: `git fetch upstream && git merge upstream/main` en la rama `corelcraft`.

CorelCraft y TraceDraw son proyectos independientes, sin relación con Corel Corporation ni Alludo. CorelDRAW es marca registrada de Corel Corporation.
