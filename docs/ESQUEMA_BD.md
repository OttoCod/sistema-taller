# Esquema de base de datos — Espínola Motorepuestos

Esquema final acordado tras la revisión de los puntos A–H. Este documento es
la referencia para escribir las migraciones de la Fase 2 en adelante; la
Fase 1 solo crea las tres tablas de infraestructura transversal
(`usuarios`, `configuracion`, `auditoria`), ya presentes en
`src-tauri/migrations/0001_bootstrap.sql`.

## Convenciones que aplican a todas las tablas

- **SQLite en modo `STRICT`**: cada columna respeta su tipo declarado.
- **Dinero en centavos**: toda columna monetaria es `INTEGER` en centavos de
  peso argentino, nunca `REAL`. Evita errores de redondeo de punto flotante.
  `$ 12.500` se guarda como `1250000`. El formato de visualización (sin
  decimales por defecto) es responsabilidad de la capa de presentación y se
  controla desde `configuracion`, no desde el esquema.
- **Fechas en `TEXT` ISO-8601** (`strftime('%Y-%m-%dT%H:%M:%fZ', 'now')`).
- **Nada se borra físicamente**: las tablas con ciclo de vida (`productos`,
  `ventas`, `compras`) usan una columna `estado`, nunca `DELETE`.
- **Claves foráneas activas** (`PRAGMA foreign_keys = ON`, ya configurado en
  `src-tauri/src/db.rs`).

## Catálogo

```
marcas
  id                  INTEGER PK
  nombre              TEXT UNIQUE NOT NULL

categorias
  id                  INTEGER PK
  nombre              TEXT NOT NULL
  categoria_padre_id  INTEGER REFERENCES categorias(id)   -- subcategorías

productos
  id                          INTEGER PK
  codigo_interno              TEXT UNIQUE NOT NULL   -- autogenerado, nunca depende de un código externo
  nombre                      TEXT NOT NULL
  marca_id                    INTEGER REFERENCES marcas(id)
  categoria_id                INTEGER REFERENCES categorias(id)
  descripcion                 TEXT
  observaciones               TEXT
  imagen_path                 TEXT                   -- ruta relativa a un archivo, nunca BLOB
  stock_actual                INTEGER NOT NULL DEFAULT 0   -- cacheado; fuente de verdad = stock_movimientos
  stock_minimo                INTEGER NOT NULL DEFAULT 0
  costo_actual                INTEGER                -- centavos
  precio_venta_actual         INTEGER                -- centavos
  precio_publico_referencia   INTEGER                -- centavos
  precio_actualizado_en       TEXT
  estado                      TEXT NOT NULL DEFAULT 'activo'   -- activo | inactivo | fusionado
  fusionado_en_id             INTEGER REFERENCES productos(id)
  creado_en                   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))

producto_codigos_fabricante        -- resuelve el punto A: equivalencias, no un campo único
  id                  INTEGER PK
  producto_id         INTEGER NOT NULL REFERENCES productos(id)
  codigo              TEXT NOT NULL
  fabricante_nombre   TEXT
  observacion         TEXT

productos_fts   -- tabla virtual FTS5 (Fase 2), no es una tabla relacional más
  nombre, codigo_interno, marca, categoria, codigos_fabricante
  tokenize = "unicode61 remove_diacritics 2"   -- mayúsculas/acentos gratis
```

**Nota de implementación (Fase 2):** en la propuesta inicial se planteó
sincronizar `productos_fts` con triggers de SQLite. Se cambió por una
sincronización explícita desde `src-tauri/src/services/productos.rs`
(dentro de la misma transacción que crea/edita un producto): es más fácil
de leer, de testear y de depurar que lógica escondida en triggers. La
tolerancia a errores de tipeo reales (más allá de mayúsculas/acentos/orden
de palabras, que resuelve FTS5 solo) se agrega en el frontend con `fuse.js`,
reordenando los candidatos que ya devolvió FTS5 — nunca escaneando toda la
tabla en el cliente.

## Precios

```
precios_historial     -- append-only; nunca se edita ni se borra una fila
  id            INTEGER PK
  producto_id   INTEGER NOT NULL REFERENCES productos(id)
  tipo          TEXT NOT NULL     -- costo | venta | publico_referencia
  valor         INTEGER NOT NULL  -- centavos
  fecha         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
  observacion   TEXT
```

El precio de venta **siempre** puede modificarse manualmente al vender; no
hay ningún cálculo automático obligatorio de aumento (decisión adicional
confirmada). `precio_venta_actual` en `productos` es solo el valor de
referencia que la pantalla de venta propone por defecto.

## Proveedores

```
proveedores
  id            INTEGER PK
  nombre        TEXT NOT NULL
  telefono      TEXT
  whatsapp      TEXT
  email         TEXT
  sitio_web     TEXT
  observaciones TEXT
  activo        INTEGER NOT NULL DEFAULT 1

producto_proveedores     -- N a N, con metadatos por relación
  id                INTEGER PK
  producto_id       INTEGER NOT NULL REFERENCES productos(id)
  proveedor_id      INTEGER NOT NULL REFERENCES proveedores(id)
  codigo_proveedor  TEXT
  url_producto      TEXT
  url_busqueda      TEXT
  es_principal      INTEGER NOT NULL DEFAULT 0
  activo            INTEGER NOT NULL DEFAULT 1
  UNIQUE (producto_id, proveedor_id)
```

## Stock

```
stock_movimientos     -- libro mayor; stock_actual en productos es el cacheado
  id                INTEGER PK
  producto_id       INTEGER NOT NULL REFERENCES productos(id)
  tipo              TEXT NOT NULL   -- venta | compra | devolucion | ajuste | fusion | anulacion
  cantidad          INTEGER NOT NULL   -- positivo o negativo
  stock_resultante  INTEGER NOT NULL
  referencia_tipo   TEXT            -- 'venta', 'compra', etc.
  referencia_id     INTEGER
  fecha             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
  observacion       TEXT
```

Toda operación que cambia stock inserta acá **y** actualiza
`productos.stock_actual` en la misma transacción SQL.

## Clientes y cuenta corriente

```
clientes
  id                          INTEGER PK
  nombre                      TEXT NOT NULL
  telefono                    TEXT
  patente                     TEXT   -- patente de la moto (Fase 6, migración 0005). Se guarda en mayúsculas.
  direccion                   TEXT
  observaciones               TEXT
  saldo_cuenta_corriente      INTEGER NOT NULL DEFAULT 0   -- centavos, cacheado

cuenta_corriente_movimientos
  id                INTEGER PK
  cliente_id        INTEGER NOT NULL REFERENCES clientes(id)
  tipo              TEXT NOT NULL   -- venta_fiada | pago | ajuste | devolucion
  monto             INTEGER NOT NULL   -- centavos; positivo aumenta deuda, negativo la reduce
  saldo_resultante  INTEGER NOT NULL
  metodo_pago_id    INTEGER REFERENCES metodos_pago(id)   -- solo en pagos
  referencia_tipo   TEXT
  referencia_id     INTEGER
  fecha             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
  observacion       TEXT
```

La fila `clientes.id = 1` es **"Consumidor final"**: reservada,
no eliminable, es el valor por defecto de `ventas.cliente_id`.

**Regla de negocio (punto D, aplicada en la capa de servicios, no en el
esquema):** una venta con método de pago "cuenta corriente" no puede
confirmarse con `cliente_id = 1`.

## Ventas

```
metodos_pago
  id      INTEGER PK
  nombre  TEXT NOT NULL   -- efectivo | transferencia | tarjeta | cuenta_corriente (semilla)
  activo  INTEGER NOT NULL DEFAULT 1
  orden   INTEGER NOT NULL DEFAULT 0

ventas
  id                 INTEGER PK
  numero             INTEGER NOT NULL UNIQUE   -- secuencial, nunca se reutiliza
  cliente_id         INTEGER NOT NULL DEFAULT 1 REFERENCES clientes(id)
  fecha              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
  estado             TEXT NOT NULL DEFAULT 'confirmada'   -- confirmada | anulada
  subtotal           INTEGER NOT NULL   -- centavos
  descuento_total    INTEGER NOT NULL DEFAULT 0
  total              INTEGER NOT NULL
  motivo_anulacion   TEXT
  fecha_anulacion    TEXT

venta_detalles
  id                     INTEGER PK
  venta_id               INTEGER NOT NULL REFERENCES ventas(id)
  producto_id            INTEGER NOT NULL REFERENCES productos(id)
  cantidad               INTEGER NOT NULL
  precio_unitario        INTEGER NOT NULL   -- el aplicado, puede diferir del de referencia
  descuento              INTEGER NOT NULL DEFAULT 0
  subtotal               INTEGER NOT NULL
  costo_unitario_snapshot INTEGER   -- copia del costo al vender, para margen histórico

venta_pagos      -- 1 a N: soporta pago dividido desde el día uno (punto C)
  id             INTEGER PK
  venta_id       INTEGER NOT NULL REFERENCES ventas(id)
  metodo_pago_id INTEGER NOT NULL REFERENCES metodos_pago(id)
  monto          INTEGER NOT NULL   -- centavos
  fecha          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
```

**Regla de negocio (punto C, validada en la capa de servicios dentro de la
misma transacción):** `SUM(venta_pagos.monto) = ventas.total` antes de
confirmar la venta. La caja (Fase 9) se calcula agrupando
`venta_pagos` por `metodo_pago_id` sobre ventas `estado = 'confirmada'`,
excluyendo `cuenta_corriente` del total efectivamente cobrado — **no existe
una tabla física de movimientos de caja** (punto B); esto evita que "lo que
dice caja" se desincronice de "lo que dicen las ventas". Cuando se agregue
arqueo/egresos (fuera de V1), ahí se crea una tabla de sesiones de caja.

## Compras y recepción

```
compras
  id              INTEGER PK
  proveedor_id    INTEGER NOT NULL REFERENCES proveedores(id)
  numero_factura  TEXT
  fecha           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
  subtotal        INTEGER NOT NULL
  total           INTEGER NOT NULL
  estado          TEXT NOT NULL DEFAULT 'registrada'   -- registrada | anulada

compra_detalles
  id              INTEGER PK
  compra_id       INTEGER NOT NULL REFERENCES compras(id)
  producto_id     INTEGER NOT NULL REFERENCES productos(id)
  cantidad        INTEGER NOT NULL
  costo_unitario  INTEGER NOT NULL
  subtotal        INTEGER NOT NULL
```

Al confirmar una recepción: suma stock (`stock_movimientos` tipo
`compra`), y si `costo_unitario` difiere del `costo_actual` del producto,
agrega una fila en `precios_historial` y actualiza el cacheado — todo en
una transacción.

## Devoluciones, anulaciones y correcciones

```
devoluciones
  id                  INTEGER PK
  venta_id            INTEGER NOT NULL REFERENCES ventas(id)
  fecha               TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
  motivo              TEXT NOT NULL
  metodo_devolucion   TEXT NOT NULL   -- reembolso_efectivo | nota_credito | cambio_producto | reduccion_deuda
  total_devuelto      INTEGER NOT NULL

devolucion_detalles
  id                INTEGER PK
  devolucion_id     INTEGER NOT NULL REFERENCES devoluciones(id)
  venta_detalle_id  INTEGER NOT NULL REFERENCES venta_detalles(id)
  cantidad          INTEGER NOT NULL
  estado_producto   TEXT NOT NULL   -- vuelve_a_stock | en_revision | defectuoso | dañado
  observacion       TEXT

correcciones     -- genérica: cualquier edición retroactiva, sin tocar el historial original
  id                INTEGER PK
  entidad_tipo      TEXT NOT NULL
  entidad_id        INTEGER NOT NULL
  campo             TEXT NOT NULL
  valor_anterior    TEXT
  valor_nuevo       TEXT
  motivo            TEXT NOT NULL
  fecha             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
```

**Criterio anulación vs. devolución (punto E), tal como se confirmó:**
anulación = la venta completa fue un error y se cancela; devolución = la
venta fue válida y el cliente trae de vuelta uno o más productos después.
Ninguna borra la venta original; `ventas.numero` nunca se reutiliza.

## Fusión de duplicados

```
producto_duplicados_candidatos
  id                  INTEGER PK
  producto_a_id       INTEGER NOT NULL REFERENCES productos(id)
  producto_b_id       INTEGER NOT NULL REFERENCES productos(id)
  score_similitud     REAL NOT NULL
  criterios_json      TEXT NOT NULL   -- qué coincidió: código, nombre, marca...
  estado              TEXT NOT NULL DEFAULT 'pendiente'   -- pendiente | fusionado | no_duplicado
  fecha_deteccion     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
  fecha_resolucion    TEXT

producto_fusiones
  id                     INTEGER PK
  producto_principal_id  INTEGER NOT NULL REFERENCES productos(id)
  producto_fusionado_id  INTEGER NOT NULL REFERENCES productos(id)
  fecha                  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
  motivo                 TEXT
  snapshot_json          TEXT NOT NULL   -- estado completo del producto absorbido antes de fusionar
```

Nunca se fusiona solo porque dos productos compartan un código (punto A);
la fusión siempre es una decisión humana desde la cola de candidatos. El
producto absorbido pasa a `estado = 'fusionado'`, sus códigos migran a
`producto_codigos_fabricante` del principal y su stock se suma vía
`stock_movimientos` tipo `fusion` — nunca se pierde información.

## Importación de Excel

```
excel_importaciones
  id                   INTEGER PK
  archivo_nombre       TEXT NOT NULL
  archivo_hash         TEXT NOT NULL
  fecha                TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
  mapeo_columnas_json  TEXT NOT NULL
  total_filas          INTEGER NOT NULL
  filas_importadas     INTEGER NOT NULL DEFAULT 0
  filas_con_error      INTEGER NOT NULL DEFAULT 0

excel_importacion_filas
  id                        INTEGER PK
  importacion_id            INTEGER NOT NULL REFERENCES excel_importaciones(id)
  fila_numero               INTEGER NOT NULL
  datos_originales_json     TEXT NOT NULL   -- dato crudo; el Excel fuente nunca se toca
  estado                    TEXT NOT NULL   -- importada | duplicado_pendiente | error | omitida
  producto_id_resultante    INTEGER REFERENCES productos(id)
  error_detalle             TEXT
```

## Sistema (comprobantes, backups, auditoría, configuración, usuarios)

```
comprobantes
  id          INTEGER PK
  venta_id    INTEGER NOT NULL REFERENCES ventas(id)
  numero      TEXT NOT NULL
  tipo        TEXT NOT NULL   -- ticket | a4
  creado_en   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))

comprobante_eventos
  id              INTEGER PK
  comprobante_id  INTEGER NOT NULL REFERENCES comprobantes(id)
  tipo_evento     TEXT NOT NULL   -- impreso | pdf_generado
  fecha           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))

backups
  id                INTEGER PK
  archivo_path      TEXT NOT NULL
  fecha             TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
  tamaño_bytes      INTEGER NOT NULL
  tipo              TEXT NOT NULL   -- manual | automatico
  version_esquema   INTEGER NOT NULL

-- ya creadas en la Fase 1 (migración 0001_bootstrap.sql):

auditoria
  id             INTEGER PK
  entidad_tipo   TEXT NOT NULL
  entidad_id     INTEGER
  accion         TEXT NOT NULL
  detalle_json   TEXT
  fecha          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
  usuario_id     INTEGER NOT NULL REFERENCES usuarios(id)

configuracion
  clave           TEXT PK
  valor           TEXT
  actualizado_en  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))

usuarios
  id       INTEGER PK
  nombre   TEXT NOT NULL
  activo   INTEGER NOT NULL DEFAULT 1
  creado_en TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
```

`configuracion` guarda, entre otras, las claves de negocio (nombre,
dirección, teléfono, logo), de comprobante (formato, numeración) y de
formato de moneda (`moneda.decimales_visibles`, por ahora `0` para
Argentina — punto G). Al ser clave-valor, agregar una configuración nueva
nunca requiere una migración de esquema.
