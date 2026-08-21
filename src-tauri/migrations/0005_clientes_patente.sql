-- Reemplaza DNI/CUIT por patente de la moto: para este negocio es un dato
-- más útil para identificar al cliente que su documento (siempre sabe la
-- patente, no siempre lleva el DNI encima).
--
-- Migración nueva en vez de editar 0004_clientes.sql: las migraciones son
-- solo hacia adelante (ver docs/ARQUITECTURA.md, "Estrategia de migraciones").

ALTER TABLE clientes RENAME COLUMN dni_cuit TO patente;
