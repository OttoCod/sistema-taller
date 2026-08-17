# Espínola Motorepuestos

Aplicación de escritorio (Windows) para reemplazar el uso de Excel, papel y
consultas manuales en la gestión del negocio: productos, stock, ventas,
clientes/cuenta corriente, compras, proveedores y caja. Funciona offline;
internet solo se usa para consultar páginas de proveedores.

Estado actual: **Fase 1 — arquitectura base**. Los módulos funcionales se
agregan progresivamente; ver el plan de fases y las decisiones tomadas en
[`docs/ARQUITECTURA.md`](./docs/ARQUITECTURA.md) y
[`docs/ESQUEMA_BD.md`](./docs/ESQUEMA_BD.md).

## Stack

Tauri 2 + React + TypeScript + SQLite (`sqlx`). Ver la evaluación completa
del stack y las alternativas consideradas en `docs/ARQUITECTURA.md`.

## Desarrollo

Requiere Node.js, Rust/Cargo, y en Linux los paquetes de desarrollo de
WebKitGTK (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`,
`libayatana-appindicator3-dev`, `libsoup-3.0-dev`) — no hacen falta en
Windows, que es la plataforma de destino.

```bash
npm install
npm run tauri dev     # levanta la app de escritorio
npm run build          # build de producción del frontend
cd src-tauri && cargo test   # pruebas del backend
```

## Recomendado para el editor

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
