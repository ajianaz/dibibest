# PRODUCT.md — codename `erdhub` (nama final = keputusan owner)

## What this product is
Bidirectional database schema tool: **dbdiagram + Liam ERD jadi satu, semua format bolak-balik.**
Satu IR (intermediate representation) skema di tengah; format di sekelilingnya:
DBML (dbdiagram) ↔ SQL DDL (PostgreSQL/MySQL/SQLite) ↔ Prisma schema ↔ ERD kanvas interaktif
↔ live-DB introspect. Jualan utama = **round-trip fidelity**: konversi bolak-balik yang jujur,
dengan laporan apa yang selamat dan apa yang bocor.

## Who it is for
Developer & data engineer yang hari ini pakai dbdiagram.io (tapi risi skema di cloud orang)
atau Liam ERD (tapi frustasi satu arah). Butuh tool self-host, cepat, tanpa akun.

## Mode (impeccable)
**Operate** — pengguna menyelesaikan tugas (import → lihat → edit → export).
Scanability, konsistensi, ekspektasi tool dev (mirip VS Code/dbdiagram) menang atas ekspresi.
Brand hidup di presisi detail, bukan dekorasi.

## Platform & stack (mandat owner, tidak bisa dilanggar)
- Core codec: Rust murni, in-process, compilable ke WASM + CLI.
- UI: Svelte 5 + svelte-flow (kanvas), self-host web.
- Metadata: SQLite. Monorepo, default branch `develop`.

## Explicit brief from owner (22 Okt 2026, Discord #research)
- "dbdiagram + liam itu sih dan bisa bolak balik juga" — round-trip adalah fitur inti, bukan bonus.
- REA (morluto/rea) = alat riset SAJA, bukan komponen produk.
- Stack Rust + Svelte "seperti biasanya".

## ASSUMPTIONS (labeled — owner belum konfirmasi)
- A1: Web-first desktop-class app (bukan desktop native) — infer dari pembandingan dbdiagram/Liam.
- A2: Single-user local dulu; kolaborasi real-time = non-goal v1 — infer dari kompleksitas.
- A3: Open-source MIT/Apache — infer dari pola produk fleet (belum diputuskan owner).
- A4: Visual: dark-first tool aesthetic ala dev tool modern (VS Code/Liam vibes) — belum ada
  brand direction resmi; mockup v1 pakai dark, eval di critique.
