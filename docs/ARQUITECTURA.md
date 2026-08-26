# Arquitectura — Espínola Motorepuestos

Este documento describe la arquitectura base construida en la **Fase 1** y
extendida en la **Fase 2** (catálogo de productos), la **Fase 4** (stock),
la **Fase 6** (clientes y cuenta corriente), la **Fase 5** (ventas — se
implementó después de la 6, ver más abajo por qué), la **Fase 7** (compras
y recepción), la **Fase 8** (gestión completa de proveedores y vínculo
producto-proveedor), la **Fase 3** (importación de listas de precios desde
Excel — se implementó después de la 8, en cuanto el negocio mandó el
archivo real), la **Fase 9** (caja) y la **Fase 11** (anulaciones y
devoluciones de venta). El esquema de base de datos completo está en
[`ESQUEMA_BD.md`](./ESQUEMA_BD.md).

La Fase 1 **no** implementaba productos, ventas, compras, clientes ni
stock: solo dejó funcionando el proyecto Tauri+React+TS, la conexión a
SQLite con migraciones, el patrón de manejo de errores, logging a archivo,
la navegación completa (con pantallas placeholder) y un comando de extremo
a extremo (`system_health_check`) que probó que toda la cadena funciona.

La Fase 2 agregó el primer módulo funcional real: catálogo de productos
(marcas, categorías, productos, códigos de fabricante, historial de
precios) con CRUD completo y buscador FTS5.

La Fase 3 (importación de Excel) se implementó cuando el negocio mandó el
archivo real (`lista_precio.xlsx`, ~2500 filas). Decisiones clave:

- **Nada se asume de la estructura fija del archivo**: la hoja, la fila de
  encabezado y las columnas (código/nombre/precio/porcentajes derivados) se
  detectan dinámicamente por el texto de los encabezados
  (`services::lector_excel`). Si no encuentra columnas reconocibles con
  datos reales debajo, falla con un error claro en vez de adivinar.
- **Lectura siempre de solo lectura**: `calamine` nunca escribe: el Excel
  original queda intacto.
- **Nunca se fusiona ni se aplica nada solo**: cada fila se clasifica
  (`producto_valido` / `seccion` / `requiere_revision` / `ignorada` /
  `error`), se marcan duplicados de código y de nombre dentro del mismo
  archivo, y coincidencias contra el catálogo ya existente por
  `codigo_legado` — todo como sugerencia. Las filas "limpias" (sin ningún
  flag) arrancan con `decision = crear_nuevo` para no tener que aprobar a
  mano miles de filas, pero el resto exige resolución manual explícita:
  crear nuevo, vincular a un producto existente (mostrando "costo actual"
  vs. "valor del Excel" antes de decidir si se actualiza — nunca se pisa
  solo) u omitir.
- **No se categoriza automáticamente**: se probó agrupar productos por los
  títulos de sección del Excel ("hasta el próximo título"), pero como el
  último título del archivo real no tenía otro que lo cerrara, terminó
  categorizando miles de productos que no correspondían. El título de
  sección se guarda como dato histórico en `categoria_excel_texto`, pero
  ningún producto importado queda con categoría asignada.
- **`productos.codigo_legado`** (nuevo campo, sin UNIQUE) guarda el código
  que traía el Excel, solo para trazabilidad — nunca es el ID interno.
- Todo el proceso (`importaciones`/`importacion_filas`) queda en
  "vista previa" hasta que cada fila se resuelve; pasa sola a "confirmada"
  cuando ya no queda ninguna pendiente. Nunca hay un botón que aplique
  todo sin revisión.

La Fase 4 agregó el ledger de `stock_movimientos`, la pantalla de Stock
(ver y ajustar cantidades, con motivo obligatorio) y la de Reposición
(productos en o por debajo del mínimo). También es el primer módulo que
escribe en la tabla `auditoria` que existía desde la Fase 1 sin uso.
Ventas y Compras van a generar movimientos de stock automáticamente
cuando lleguen esas fases; por ahora el único tipo posible es `ajuste`.

La Fase 6 agregó clientes (con "Consumidor final" como fila reservada,
`id = 1`, no editable) y cuenta corriente: mismo patrón de ledger que
`stock_movimientos`, con **pago** (reduce la deuda, con método de pago) y
**ajuste manual con motivo obligatorio** (para migrar deudas que ya
existían en papel). `venta_fiada` y `devolucion` quedan en el esquema
listos para cuando lleguen Ventas y Devoluciones. Como registrar un pago
necesita saber el método, esta fase también creó la tabla `metodos_pago`
(semilla: efectivo, transferencia, tarjeta, cuenta_corriente) que en el
diseño original estaba pensada para Ventas/Caja.

La Fase 5 (Ventas) agregó `ventas`, `venta_detalles` y `venta_pagos`. Se
implementó después de clientes/cuenta corriente por decisión del negocio,
no por dependencia técnica. Decisiones clave:

- **Nada se recibe calculado del frontend**: `subtotal`/`descuento_total`/
  `total` se recalculan siempre del lado del servidor a partir de los
  items del carrito, para que no se pueda mandar un total manipulado.
- **La suma de los pagos tiene que coincidir exactamente con el total**
  (punto C) antes de poder confirmar — validado en la misma transacción.
- **Venta fiada exige cliente real**: si algún pago usa el método
  "cuenta_corriente", no se puede confirmar con "Consumidor final" (punto
  D) — genera además un movimiento `venta_fiada` en la cuenta corriente
  del cliente, con la misma mecánica de la Fase 6.
- **Stock insuficiente avisa, no bloquea**: si el carrito pide más
  cantidad de la que hay, la fila se marca en la interfaz pero la venta se
  puede confirmar igual — decisión explícita del negocio, no un
  descuido.
- **Número de venta = mismo mecanismo que `codigo_interno` de producto**:
  se deriva del `id` autoincremental, nunca se reutiliza.
- Anulación (Fase 11) y comprobante/impresión (Fase 10) llegaron después —
  hasta entonces la confirmación de venta era solo una pantalla en la app.

La Fase 7 (Compras y recepción) agregó `proveedores`, `compras` y
`compra_detalles`. Decisiones clave:

- **`proveedores` se creó completo en esta fase** (todos los campos de
  contacto ya definidos en `ESQUEMA_BD.md`) porque una compra necesita
  poder asignarle un proveedor desde el día uno — mismo criterio que
  `metodos_pago` en la Fase 6. Pero la **pantalla de gestión completa**
  (listado, edición, vínculo producto-proveedor, "Consultar proveedor"
  abriendo el sitio web) sigue siendo la Fase 8: por ahora solo existe un
  selector con búsqueda y un "crear proveedor nuevo" inline (mismo
  `ProveedorFormDialog` que va a reutilizar la Fase 8 para editar) dentro
  de la pantalla de recepción.
- **Recibir stock nunca se bloquea por el estado del producto**: a
  diferencia de Ventas, una compra puede cargar cantidad para un producto
  inactivo (reponer inventario no debería depender de si está a la venta).
- **El costo se actualiza igual que el precio en el formulario de
  producto**: si el `costoUnitario` cargado difiere del `costo_actual`
  del producto, se actualiza el cacheado y se agrega una fila en
  `precios_historial` (tipo `costo`) — se reutiliza la misma función
  `registrar_precio` de la Fase 2, ahora expuesta como `pub(crate)`.
- **`subtotal`/`total` se recalculan siempre del lado del servidor**,
  mismo criterio que Ventas: nunca se confían del frontend.
- **No hay "numero" separado como en `ventas`**: `compras` se identifica
  directamente por su `id` (mostrado como `C-000001`); a diferencia de
  `ventas.numero`, este esquema no necesitaba esa columna extra.
- Anulación de compra queda fuera de esta fase (no la pidió el negocio
  todavía; `compras.estado` ya admite `anulada` pero sin motivo/fecha
  propios) y el vínculo `producto_proveedores` llegó en la Fase 8.

La Fase 8 (Proveedores) completó lo que la Fase 7 había dejado mínimo.
Decisiones clave:

- **`proveedores::listar`/`buscar` dejaron de filtrar solo activos**:
  ahora devuelven todos, activos e inactivos, mismo criterio que
  `productos::listar` (que solo excluye `fusionado`). El selector de
  proveedor de la pantalla de compra (Fase 7) sigue mostrando solo
  activos, pero ahora filtrando del lado del frontend -- igual que el
  carrito de venta filtra productos `estado == 'activo'` sobre un
  `buscarProductos` que también devuelve todos.
- **Baja de proveedor = desactivar, nunca borrar**: se agregó un campo
  `activo` editable en el mismo `ProveedorFormDialog` que ya existía
  desde la Fase 7 (visible solo al editar, no al crear un proveedor
  nuevo).
- **Vínculo producto-proveedor (`producto_proveedores`)**: se administra
  desde un diálogo por proveedor ("Productos"), no desde el formulario
  de producto. `agregar` hace upsert por `(producto_id, proveedor_id)` --
  como esa combinación es UNIQUE en el esquema, volver a vincular un
  producto que ya se había "quitado" (soft-delete, `activo = 0`)
  reactiva la misma fila en vez de chocar contra la restricción o crear
  un duplicado.
- **`es_principal` es un flag informativo, sin exclusividad automática**:
  marcar un vínculo como principal no desmarca otros del mismo producto
  con otros proveedores. No hay ninguna regla de negocio especificada
  para eso en el documento original, así que se evitó inventarla.
- **"Consultar proveedor"** abre `sitio_web` en el navegador del sistema
  con `@tauri-apps/plugin-opener` (`openUrl`, comando
  `plugin:opener|open_url`), agregando `https://` si el valor cargado no
  tiene esquema. Ya estaba instalado y con permiso `opener:default`
  desde la Fase 1, sin cambios de capabilities necesarios.

La Fase 9 (Caja) es, deliberadamente, la más chica de todas: **no agrega
ninguna tabla nueva**. Tal como ya estaba decidido desde el diseño original
(`ESQUEMA_BD.md`, punto B), la caja se calcula agrupando `venta_pagos` por
método de pago sobre las ventas `confirmada` de un día, excluyendo
`cuenta_corriente` del total cobrado (fiado no es plata que entró). Así "lo
que dice caja" nunca se puede desincronizar de lo que dicen las ventas —
no hay nada que recalcular ni mantener sincronizado a mano. Arqueo (contar
la plata física) y egresos quedan fuera de esta fase, según lo ya definido.

La Fase 11 (anulaciones y devoluciones) agregó dos formas de revertir una
venta, tal como estaban distinguidas desde el diseño original
(`ESQUEMA_BD.md`, punto E): **anulación** = la venta completa fue un error
y se cancela entera; **devolución** = la venta fue válida y el cliente
trae de vuelta uno o más productos después. Ninguna borra la venta ni sus
filas — `ventas.numero` nunca se reutiliza. Decisiones clave:

- **Anular (`services::ventas::anular`) no necesitó migración**:
  `ventas.estado`, `motivo_anulacion` y `fecha_anulacion` ya existían desde
  la Fase 5, dejados previstos a propósito. Revierte, en una sola
  transacción, el stock que había descontado (`stock_movimientos` tipo
  `anulacion`) y la deuda de cuenta corriente que haya generado (si algún
  pago fue `cuenta_corriente`, un movimiento `devolucion` la cancela). La
  caja (Fase 9) ya filtraba por `estado = 'confirmada'` desde que se
  escribió esa consulta, así que una venta anulada desaparece de la caja
  de ese día sin tocar `services::caja`.
- **Devolución (`devoluciones`, `devolucion_detalles`, migración 0010) es
  parcial o total**: cada línea se puede devolver hasta la cantidad que le
  quede sin devolver (sumando devoluciones previas sobre esa misma línea),
  y el monto de cada línea es proporcional al importe efectivo que quedó
  en `venta_detalles.subtotal` (con su descuento ya aplicado), no un
  precio recalculado de cero.
- **El estado del producto devuelto decide si vuelve a stock**:
  `estado_producto = 'vuelve_a_stock'` genera un `stock_movimientos` tipo
  `devolucion`; `en_revision` / `defectuoso` / `dañado` solo quedan
  registrados, sin tocar el stock.
- **`metodo_devolucion = 'reduccion_deuda'`** reduce la cuenta corriente
  del cliente de la venta (exige que no sea "Consumidor final"); los otros
  tres métodos (`reembolso_efectivo`, `nota_credito`, `cambio_producto`)
  quedan solo como clasificación — no hay todavía una tabla de notas de
  crédito ni un vínculo automático a una nueva venta de cambio, porque el
  esquema original no las especificaba y el negocio no las pidió en esta
  fase.
- **Una venta con devoluciones ya registradas no se puede anular**, y una
  venta anulada no admite devoluciones nuevas: mezclar ambas reversiones
  sobre las mismas líneas dejaría el ledger de stock/cuenta corriente
  ambiguo.
- **Anulación de compras y la tabla genérica `correcciones` quedan fuera
  de esta fase** — el negocio pidió específicamente anulaciones y
  devoluciones de venta, no ese alcance más amplio.

Estructura y decisiones nuevas están marcadas como "(Fase 2)" / "(Fase 3)"
/ "(Fase 4)" / "(Fase 5)" / "(Fase 6)" / "(Fase 7)" / "(Fase 8)" /
"(Fase 9)" / "(Fase 11)" abajo; el resto sigue siendo tal cual quedó en
fases anteriores.

## 1. Estructura del proyecto

```
sistema-taller/
├── src/                              # React + TypeScript
│   ├── modules/
│   │   ├── inicio/
│   │   │   └── InicioPage.tsx        # llama a system_health_check
│   │   ├── productos/                # (Fase 2)
│   │   │   ├── CatalogoPage.tsx      # listado + búsqueda (FTS5 + fuse.js)
│   │   │   ├── ProductoFormDialog.tsx
│   │   │   ├── productoSchema.ts     # validación zod + mapeos pesos↔centavos
│   │   │   ├── StockPage.tsx         # (Fase 4)
│   │   │   ├── ReposicionPage.tsx    # (Fase 4) stock_actual <= stock_minimo
│   │   │   ├── StockTable.tsx        # (Fase 4) tabla compartida por ambas
│   │   │   └── AjusteStockDialog.tsx # (Fase 4) ajuste con motivo + stock mínimo
│   │   ├── clientes/                 # (Fase 6)
│   │   │   ├── ClientesPage.tsx
│   │   │   ├── CuentasPendientesPage.tsx  # saldo > 0
│   │   │   ├── ClienteFormDialog.tsx
│   │   │   ├── CuentaCorrienteDialog.tsx  # historial + registrar pago/ajuste
│   │   │   └── clienteSchema.ts
│   │   ├── ventas/                   # (Fase 5)
│   │   │   ├── NuevaVentaPage.tsx    # buscar → carrito → cliente → pagos → confirmar
│   │   │   ├── ClienteSelector.tsx   # combobox liviano, reutilizado acá
│   │   │   ├── HistorialVentasPage.tsx
│   │   │   └── VentaDetalleDialog.tsx  # (Fase 11) + anular / registrar devolución
│   │   ├── compras/                  # (Fase 7)
│   │   │   ├── NuevaCompraPage.tsx   # buscar → items → proveedor → confirmar
│   │   │   ├── ProveedorSelector.tsx # combobox + "crear proveedor nuevo" inline
│   │   │   ├── HistorialComprasPage.tsx
│   │   │   └── CompraDetalleDialog.tsx
│   │   ├── proveedores/              # (Fase 7 el formulario, Fase 8 el resto)
│   │   │   ├── ProveedoresPage.tsx      # (Fase 8) listado + búsqueda + Consultar proveedor
│   │   │   ├── ProveedorFormDialog.tsx  # (Fase 7) reutilizado acá para alta/edición, con toggle Activo agregado en Fase 8
│   │   │   ├── ProveedorProductosDialog.tsx  # (Fase 8) vínculo producto_proveedores por proveedor
│   │   │   └── proveedorSchema.ts
│   │   ├── importaciones/            # (Fase 3)
│   │   │   ├── ImportarExcelPage.tsx # elegir archivo + historial de importaciones
│   │   │   ├── RevisionImportacionPage.tsx  # resumen + pestañas + aplicar en bloque las limpias
│   │   │   ├── FilaRevisionRow.tsx   # crear nuevo / vincular / omitir, por fila
│   │   │   └── ProductoVincularSelector.tsx # combobox para elegir el producto a vincular
│   │   ├── caja/                     # (Fase 9)
│   │   │   └── CajaPage.tsx          # selector de fecha + resumen calculado al momento
│   │   └── placeholder/
│   │       └── PlaceholderPage.tsx   # pantalla "módulo pendiente — Fase N"
│   ├── components/layout/
│   │   ├── AppShell.tsx              # sidebar + topbar + <Outlet/>
│   │   ├── Sidebar.tsx               # navegación (sección 29), generada desde lib/nav.ts
│   │   ├── Topbar.tsx                # buscador global -- sigue solo visual; NuevaVentaPage tiene su propio buscador de productos, no comparte este
│   │   └── ErrorBoundary.tsx         # red de contención de errores de render
│   ├── lib/
│   │   ├── nav.ts                    # única fuente de verdad de la navegación
│   │   ├── money.ts                  # (Fase 2) centavos↔pesos, formato ARS
│   │   └── api/
│   │       ├── client.ts             # invoke() tipado + AppError
│   │       ├── system.ts             # wrapper de system_health_check
│   │       ├── marcas.ts             # (Fase 2)
│   │       ├── categorias.ts         # (Fase 2)
│   │       ├── productos.ts          # (Fase 2)
│   │       ├── stock.ts              # (Fase 4)
│   │       ├── clientes.ts           # (Fase 6)
│   │       ├── cuentaCorriente.ts    # (Fase 6)
│   │       ├── metodosPago.ts        # (Fase 6)
│   │       ├── ventas.ts             # (Fase 5, anularVenta en Fase 11)
│   │       ├── devoluciones.ts       # (Fase 11)
│   │       ├── proveedores.ts        # (Fase 7)
│   │       ├── compras.ts            # (Fase 7)
│   │       ├── productoProveedores.ts  # (Fase 8)
│   │       ├── importaciones.ts      # (Fase 3)
│   │       └── caja.ts               # (Fase 9)
│   ├── styles/globals.css            # Tailwind v4 + tokens de color provisorios
│   ├── App.tsx                       # rutas (HashRouter) + QueryClientProvider
│   └── main.tsx
├── src-tauri/                        # Rust
│   ├── src/
│   │   ├── commands/                 # #[tauri::command], una capa delgada
│   │   │   ├── system.rs
│   │   │   ├── marcas.rs             # (Fase 2)
│   │   │   ├── categorias.rs         # (Fase 2)
│   │   │   ├── productos.rs          # (Fase 2)
│   │   │   ├── stock.rs              # (Fase 4)
│   │   │   ├── clientes.rs           # (Fase 6)
│   │   │   ├── cuenta_corriente.rs   # (Fase 6)
│   │   │   ├── metodos_pago.rs       # (Fase 6)
│   │   │   ├── ventas.rs             # (Fase 5, ventas_anular en Fase 11)
│   │   │   ├── proveedores.rs        # (Fase 7)
│   │   │   ├── compras.rs            # (Fase 7)
│   │   │   ├── producto_proveedores.rs  # (Fase 8)
│   │   │   ├── importaciones.rs      # (Fase 3)
│   │   │   ├── caja.rs               # (Fase 9)
│   │   │   └── devoluciones.rs       # (Fase 11)
│   │   ├── services/                 # reglas de negocio, sin nada de Tauri
│   │   │   ├── system.rs
│   │   │   ├── marcas.rs             # (Fase 2)
│   │   │   ├── categorias.rs         # (Fase 2)
│   │   │   ├── productos.rs          # (Fase 2) CRUD + historial de precios + FTS5
│   │   │   ├── stock.rs              # (Fase 4) ajuste con motivo + escribe en auditoria
│   │   │   ├── clientes.rs           # (Fase 6)
│   │   │   ├── cuenta_corriente.rs   # (Fase 6) pago/ajuste + escribe en auditoria
│   │   │   ├── metodos_pago.rs       # (Fase 6)
│   │   │   ├── ventas.rs             # (Fase 5) totales recalculados server-side, stock, cuenta corriente y auditoria en una sola transacción; anular() en Fase 11
│   │   │   ├── proveedores.rs        # (Fase 7) CRUD, mismo patrón que clientes.rs; listar/buscar ampliado en Fase 8
│   │   │   ├── compras.rs            # (Fase 7) suma stock + costo/precios_historial + auditoria en una sola transacción
│   │   │   ├── producto_proveedores.rs  # (Fase 8) upsert por (producto_id, proveedor_id), quitar = soft-delete
│   │   │   ├── lector_excel.rs       # (Fase 3) puro: detecta estructura + lee filas, sin DB ni clasificación
│   │   │   ├── importaciones.rs      # (Fase 3) clasificar, marcar duplicados, resolver/aplicar filas
│   │   │   ├── caja.rs               # (Fase 9) agrega venta_pagos, sin tabla propia
│   │   │   └── devoluciones.rs       # (Fase 11) devolución parcial/total, repone stock según estado_producto
│   │   ├── models/                   # (Fase 2) structs compartidos entre commands/services
│   │   │   ├── marca.rs
│   │   │   ├── categoria.rs
│   │   │   ├── producto.rs           # codigo_legado agregado en Fase 3
│   │   │   ├── stock.rs              # (Fase 4)
│   │   │   ├── cliente.rs            # (Fase 6)
│   │   │   ├── cuenta_corriente.rs   # (Fase 6)
│   │   │   ├── metodo_pago.rs        # (Fase 6)
│   │   │   ├── venta.rs              # (Fase 5, motivo_anulacion/fecha_anulacion expuestos y AnularVenta en Fase 11)
│   │   │   ├── proveedor.rs          # (Fase 7, activo en GuardarProveedor desde Fase 8)
│   │   │   ├── compra.rs             # (Fase 7)
│   │   │   ├── producto_proveedor.rs # (Fase 8)
│   │   │   ├── importacion.rs        # (Fase 3)
│   │   │   ├── caja.rs               # (Fase 9)
│   │   │   └── devolucion.rs         # (Fase 11)
│   │   ├── db.rs                     # pool SQLite, migraciones, AppState
│   │   ├── error.rs                  # AppError (thiserror + Serialize)
│   │   ├── logging.rs                # tracing a archivo diario
│   │   ├── lib.rs                    # arma el Builder de Tauri
│   │   └── main.rs
│   ├── migrations/
│   │   ├── 0001_bootstrap.sql        # usuarios, configuracion, auditoria
│   │   ├── 0002_catalogo.sql         # (Fase 2) marcas, categorias, productos, productos_fts
│   │   ├── 0003_stock.sql            # (Fase 4) stock_movimientos
│   │   ├── 0004_clientes.sql         # (Fase 6) metodos_pago, clientes, cuenta_corriente_movimientos
│   │   ├── 0005_clientes_patente.sql # dni_cuit → patente (pedido tras probar la Fase 6)
│   │   ├── 0006_ventas.sql           # (Fase 5) ventas, venta_detalles, venta_pagos
│   │   ├── 0007_compras.sql          # (Fase 7) proveedores, compras, compra_detalles
│   │   ├── 0008_producto_proveedores.sql  # (Fase 8) vínculo N:N producto-proveedor
│   │   ├── 0009_importaciones.sql    # (Fase 3) importaciones, importacion_filas, productos.codigo_legado
│   │   └── 0010_devoluciones.sql     # (Fase 11) devoluciones, devolucion_detalles
│   └── Cargo.toml
└── docs/
    ├── ARQUITECTURA.md               # este archivo
    └── ESQUEMA_BD.md
```

Cada fase futura agrega una carpeta de dominio (`modules/ventas/`,
`services/venta.rs`, etc.) sin tocar las ya existentes. `commands/` y
`services/` están separados a propósito: `commands/*.rs` es una capa
delgada que solo traduce entre Tauri y el dominio (extrae el `State`,
llama al servicio, devuelve el resultado); toda la regla de negocio real
vive en `services/*.rs`, que no sabe nada de Tauri y por lo tanto es
trivial de testear (como en `db.rs`, que testea `services::system` sin
levantar ninguna ventana).

## 2. Flujo de comunicación: React → Tauri/Rust → SQLite

```
InicioPage.tsx
  useQuery(["system","health-check"], getHealthCheck)
        │
        ▼
lib/api/system.ts → invoke("system_health_check")
        │
        ▼
lib/api/client.ts   ← único punto de la app que llama a @tauri-apps/api
  invoke(cmd, args)   normaliza cualquier rechazo en un AppError tipado
        │  IPC de Tauri
        ▼
src-tauri/commands/system.rs
  #[tauri::command] async fn system_health_check(state: State<AppState>)
        │
        ▼
src-tauri/services/system.rs
  health_check(pool, db_path) → arma dos SELECT y devuelve HealthCheck
        │
        ▼
SQLite (sqlx::SqlitePool, WAL, foreign_keys=ON)
```

Regla fija para todas las fases siguientes: la interfaz **nunca** llama a
`@tauri-apps/api` directamente fuera de `lib/api/client.ts`, y **nunca**
hay una segunda vía de acceso a SQLite desde el frontend (se descartó el
enfoque híbrido con `tauri-plugin-sql` que se había mencionado como
posibilidad en la propuesta inicial: tener una sola vía de acceso a datos
es más simple de mantener que tener dos). Toda lectura y escritura pasa por
un comando Rust, y las escrituras que tocan más de una tabla (vender,
recibir compra, pagar cuenta corriente, fusionar productos) corren dentro
de una transacción SQL en el servicio correspondiente.

## 3. Dependencias principales

**Frontend** (`package.json`)

| Paquete | Para qué |
|---|---|
| `react`, `react-dom` | UI |
| `react-router-dom` (`HashRouter`) | Navegación — `HashRouter` porque la app se sirve desde el binario empaquetado, no desde un servidor con rutas propias |
| `@tanstack/react-query` | Cache, estados de carga/error alrededor de cada `invoke()` |
| `tailwindcss` v4 + `@tailwindcss/vite` | Estilos, vía tokens en `styles/globals.css` |
| `clsx`, `tailwind-merge` | Combinar clases condicionalmente sin duplicar utilidades |
| `@tauri-apps/api` | Puente IPC con Rust |
| `@tauri-apps/plugin-opener` | Abrir el navegador del sistema (Fase 8, "Consultar proveedor") |
| `zod` (Fase 2) | Validación del formulario de producto |
| `@radix-ui/react-dialog` (Fase 2) | Modal accesible del formulario de producto |
| `fuse.js` (Fase 2) | Reordenamiento difuso de los candidatos que devuelve FTS5 (punto H) |

Deliberadamente **no** instalados todavía: `zustand` (no hace falta estado
global más allá del cache de React Query), `react-hook-form` (el
formulario de producto alcanza con estado de React simple; se reconsidera
si Ventas/Compras lo necesitan).

**Backend** (`src-tauri/Cargo.toml`)

| Crate | Para qué |
|---|---|
| `tauri` 2.x, `tauri-plugin-opener` | Runtime de la app y apertura de URLs |
| `tauri-plugin-dialog` (Fase 3) | Selector nativo de archivo para elegir el Excel a importar |
| `sqlx` (`sqlite`, `runtime-tokio`, `migrate`) | Acceso a SQLite y migraciones versionadas |
| `tokio` | Runtime async que usan Tauri y sqlx |
| `thiserror` | `AppError` (sección 5) |
| `serde`, `serde_json` | (de)serialización IPC |
| `chrono` | Fechas |
| `tracing`, `tracing-subscriber`, `tracing-appender` | Logging a archivo (sección 6) |
| `calamine` (Fase 3) | Lectura de `.xlsx`, siempre solo lectura |
| `sha2` (Fase 3) | Hash del archivo importado, para avisar si ya se importó antes |
| `tempfile` (dev) | Test de `db.rs` sobre una base temporal |

Deliberadamente no instalados todavía: `rust_xlsxwriter` (no hace falta
escribir Excel, solo leerlo), `printpdf` (solo si el enfoque de impresión
vía WebView2 de la Fase 10 no alcanza), `zip` y `tauri-plugin-fs` (Fase 12,
backups).

## 4. Estrategia de migraciones

- `sqlx::migrate!("./migrations")` embebe los `.sql` en el binario en
  tiempo de compilación y los aplica automáticamente al arrancar la app,
  antes de que cualquier comando pueda ejecutarse (`lib.rs`, dentro de
  `setup`).
- Convención de nombres: `NNNN_descripcion.sql`, secuencial y
  autoincremental (`0001_bootstrap.sql`, `0002_...`). Cada archivo es
  **solo hacia adelante** — no hay migraciones de "bajada". Sobre datos
  reales del negocio, revertir un cambio de esquema se resuelve
  restaurando el backup automático (ver sección 5), que es más seguro que
  una migración de bajada mal probada.
- `sqlx` registra lo aplicado en su propia tabla `_sqlx_migrations`; el
  campo `schemaVersion` que muestra la pantalla de Inicio es
  `COUNT(*)` sobre esa tabla, así que confirma en vivo que las migraciones
  corrieron.
- Convención de esquema: toda tabla nueva se crea con `STRICT` (ver
  `ESQUEMA_BD.md`).
- A partir de que exista el módulo de backups (Fase 12): antes de aplicar
  una migración nueva sobre una base que ya tiene datos reales, la app
  dispara un backup automático primero. En la Fase 1 no aplica todavía
  porque no hay datos de negocio que proteger.

## 5. Estrategia de backups (diseño — se implementa en la Fase 12)

- **Snapshot atómico**: `VACUUM INTO 'archivo.db'` en vez de copiar el
  archivo `.db` en caliente — evita capturar un estado a medio escribir,
  incluso en modo WAL.
- **Manifiesto**: cada backup se guarda junto a un `.json` con
  `version_esquema`, fecha y versión de la app, y queda registrado en la
  tabla `backups`.
- **Ubicación**: carpeta elegida por el usuario (se recuerda en
  `configuracion`), vía `tauri-plugin-dialog`.
- **Retención**: se conservan las últimas *N* copias (configurable en
  `configuracion`, default a definir), rotando las más viejas.
- **Automático**: al abrir la app, si el último backup tiene más de 24 h,
  se dispara uno nuevo en segundo plano.
- **Restauración**: siempre en tres pasos — (1) backup de seguridad de la
  base actual, (2) confirmación explícita del usuario, (3) reemplazo del
  archivo `.db` y reinicio de la app.

## 6. Estrategia de manejo de errores

- **Un solo tipo de error para toda la app**: `AppError`
  (`src-tauri/src/error.rs`), con variantes `Database`, `Validation`,
  `NotFound`, `Conflict`, `Io`, `Unexpected`. Todo comando de Tauri
  devuelve `Result<T, AppError>`.
- **Serialización consistente**: `AppError` implementa `Serialize` a mano
  para viajar al frontend siempre como `{ kind, message }` — nunca el
  `Debug` crudo de Rust.
- **Log automático de fallas del sistema**: al serializar, si el error
  *no* es uno de negocio (`Validation`/`NotFound`/`Conflict` son
  esperables y ya tienen mensaje pensado para el usuario), se registra con
  `tracing::error!` antes de cruzar el IPC. Esto es clave porque la app no
  corre desde una terminal: sin este log, un fallo de base de datos en la
  PC del local sería indiagnosticable después del hecho.
- **Logging a archivo diario** (`logging.rs`), en el directorio de datos
  de la app (`.../logs/espinola.log.YYYY-MM-DD`), sin salida por consola.
- **Un solo choke point en el frontend**: `lib/api/client.ts` es el único
  lugar que llama a `@tauri-apps/api`. Normaliza cualquier rechazo a la
  clase `AppError` (TS), que expone `userMessage` — el texto de negocio
  tal cual si es un error esperable, o un mensaje genérico
  ("quedó registrado en el log") si es un fallo de sistema. Los
  componentes nunca manejan el error crudo de Tauri.
- **Red de contención de UI**: `ErrorBoundary` alrededor de toda la app
  evita una pantalla en blanco ante un error de render no controlado.

## 7. Extensibilidad — por qué esto no se reescribe en fases futuras

- **Módulos por dominio** en ambos lados (`src/modules/<dominio>`,
  `src-tauri/src/{commands,services}/<dominio>.rs`): una fase nueva agrega
  carpetas, no reestructura las existentes.
- **`configuracion` clave-valor**: ajustes nuevos (formato de comprobante,
  stock mínimo por defecto, decimales de moneda) nunca requieren una
  migración de esquema.
- **`usuarios` ya existe, con una sola fila** ("Usuario principal"): toda
  tabla que necesite "quién hizo esto" (`auditoria.usuario_id`, y las que
  se agreguen en fases futuras) ya referencia esta tabla desde el día uno.
  Agregar login/roles más adelante (sección 32) es sumar filas y una
  pantalla de autenticación — no una migración destructiva.
- **`auditoria` genérica**: un evento nuevo es un `accion` de texto nuevo,
  no una columna ni una tabla nueva.
- **Dinero en centavos + formato en `configuracion`**: cambiar cómo se
  *muestra* la moneda no toca cómo se *guarda*.
- **`venta_pagos` como tabla 1 a N desde el día uno**: la pantalla de
  venta puede lanzarse simple (un método) y habilitar el pago dividido más
  adelante sin migrar datos existentes.
- **Único choke point de acceso a datos** (comandos Rust) y **único choke
  point de IPC en el frontend** (`lib/api/client.ts`): agregar, cambiar o
  instrumentar (logging, métricas, reintentos) cómo se llama a Rust se
  hace en un lugar, no en cada componente.
- **Límite documentado, no escondido**: SQLite es de un solo escritor.
  Mientras el negocio use una sola PC esto no es un problema; si en algún
  momento hace falta una segunda PC o sincronización (sección 33), el
  cambio queda contenido en `src-tauri/src/db.rs` y los `services/*`,
  porque el frontend nunca habla con SQLite directamente.
