# CorelCraft 2

CorelCraft 2 es una adaptación de **TraceDraw** (https://github.com/tracedraw/tracedraw),
editor vectorial de código abierto en Rust, con licencia MIT o Apache-2.0
(ver `LICENSE-MIT`, `LICENSE-APACHE`, `CONTRIBUTORS.md` y `ATTRIBUTION.md`, que se conservan sin cambios).

Cambios de CorelCraft sobre TraceDraw (rama `corelcraft`):
- Nombre CorelCraft en la interfaz (todos los idiomas), título de ventana, id de aplicación `corelcraft` y lanzador `packaging/linux/corelcraft.desktop`.
- Ícono de CorelCraft en `assets/icon`.
- Paleta integrada "Budapest" con los colores de marca.
- Búsqueda de actualizaciones desactivada por defecto (las actualizaciones de TraceDraw reemplazarían a CorelCraft).
- El texto "Acerca de" cita a TraceDraw y su licencia.

Para traer mejoras nuevas de TraceDraw: `git fetch upstream && git merge upstream/main` en la rama `corelcraft`.

CorelCraft y TraceDraw son proyectos independientes, sin relación con Corel Corporation ni Alludo. CorelDRAW es marca registrada de Corel Corporation.
