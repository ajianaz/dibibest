//! Prisma codec conformance: Prisma → IR → Prisma, DBML → Prisma → DBML over the
//! whole corpus, and the fidelity report for what Prisma cannot hold.

use dibibest_core::{FormatId, Registry};
use std::fs;
use std::path::Path;

const SCHEMA: &str = r#"datasource db {
  provider = "postgresql"
  url      = env("DATABASE_URL")
}

enum Role {
  USER
  ADMIN
}

/// Registered accounts
model User {
  id      Int     @id @default(autoincrement())
  /// Login e-mail
  email   String  @unique @db.VarChar(255)
  role    Role    @default(USER)
  bio     String?
  tags    String[]
  posts   Post[]
  profile Profile?
}

model Post {
  id        Int      @id @default(autoincrement())
  author_id Int
  title     String
  created   DateTime @default(now())
  author    User     @relation(fields: [author_id], references: [id])

  @@index([author_id, created], map: "post_author_created")
}

model Profile {
  user_id Int  @id
  loc     Unsupported("geography(Point, 4326)")?
  user    User @relation(fields: [user_id], references: [id])
}

model Membership {
  user_id Int
  org_id  Int

  @@id([user_id, org_id])
  @@map("memberships")
}
"#;

fn id(s: &str) -> FormatId {
    FormatId::new(s)
}

#[test]
fn prisma_round_trip_is_idempotent() {
    let reg = Registry::with_defaults();
    let p = reg.codec(&id("prisma")).unwrap();
    let ir1 = p.parse(SCHEMA).expect("parse prisma");
    assert_eq!(ir1.tables.len(), 4);
    assert_eq!(ir1.enums.len(), 1);
    assert_eq!(ir1.refs.len(), 2);
    assert_eq!(ir1.refs[1].kind, dibibest_core::RefKind::OneToOne);
    assert_eq!(ir1.table("User").unwrap().columns[1].ty, "varchar(255)");
    assert_eq!(
        ir1.table("memberships")
            .unwrap()
            .columns
            .iter()
            .filter(|c| c.pk)
            .count(),
        2
    );
    let out = p.emit(&ir1).unwrap();
    let ir2 = p.parse(&out).expect("re-parse emitted prisma");
    assert_eq!(ir1, ir2, "Prisma round-trip must be lossless");
    assert_eq!(out, p.emit(&ir2).unwrap(), "emit must be stable");
}

/// DBML → Prisma → DBML (twice): the second DBML must equal the first, i.e. the
/// trip through Prisma reaches a fixed point on every corpus file, and what the
/// first pass keeps never changes afterwards.
#[test]
fn corpus_dbml_prisma_dbml_is_idempotent() {
    let reg = Registry::with_defaults();
    let (dbml, prisma) = (
        reg.codec(&id("dbml")).unwrap(),
        reg.codec(&id("prisma")).unwrap(),
    );
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
    let mut n = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|x| x != "dbml") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let src = fs::read_to_string(&path).unwrap();
        let ir0 = dbml.parse(&src).unwrap();

        let p1 = prisma.emit(&ir0).unwrap();
        let ir_p = prisma
            .parse(&p1)
            .unwrap_or_else(|e| panic!("{name}: re-parse prisma: {e}\n{p1}"));
        assert_eq!(
            p1,
            prisma.emit(&ir_p).unwrap(),
            "{name}: prisma emit unstable"
        );
        let d1 = dbml.emit(&ir_p).unwrap();

        let p2 = prisma.emit(&dbml.parse(&d1).unwrap()).unwrap();
        let d2 = dbml.emit(&prisma.parse(&p2).unwrap()).unwrap();
        assert_eq!(d1, d2, "{name}: DBML → Prisma → DBML is not idempotent");

        // structure survives the trip (indexes are covered by the fidelity test)
        let ir1 = dbml.parse(&d1).unwrap();
        assert_eq!(ir0.tables.len(), ir1.tables.len(), "{name}: tables");
        assert_eq!(
            ir0.tables.iter().map(|t| t.columns.len()).sum::<usize>(),
            ir1.tables.iter().map(|t| t.columns.len()).sum::<usize>(),
            "{name}: columns"
        );
        assert_eq!(ir0.refs.len(), ir1.refs.len(), "{name}: refs");
        assert_eq!(ir0.enums, ir1.enums, "{name}: enums");
        n += 1;
    }
    assert!(n >= 12, "expected the whole corpus, got {n}");
    println!("DBML → Prisma → DBML idempotent on {n} corpus files");
}

#[test]
fn expression_indexes_are_reported_as_dropped() {
    let reg = Registry::with_defaults();
    let src = "Table t {\n  id int [pk]\n  a int\n  indexes {\n    (a) [name: 'plain']\n    \
               (`lower(a::text)`) [name: 'expr_idx']\n    (`a WHERE a > 0`) [unique, name: 'partial_idx']\n  }\n}\n";
    let c = reg
        .convert_explicit(src, &id("dbml"), &id("prisma"))
        .unwrap();
    println!("{}\nfidelity: {:?}", c.output, c.fidelity);
    assert!(c.output.contains("@@index([a], map: \"plain\")"));
    assert!(
        !c.output.contains("lower("),
        "expression index must not leak"
    );
    assert_eq!((c.fidelity.indexes.kept, c.fidelity.indexes.total), (1, 3));
    assert!(!c.fidelity.indexes.lossless());
    let note = c.fidelity.dropped.join(" | ");
    assert!(
        note.contains("2 expression/partial index")
            && note.contains("t.expr_idx")
            && note.contains("t.partial_idx"),
        "drop must be reported, got: {note}"
    );
    // DBML → DBML keeps them: no drop reported
    let keep = reg.convert_explicit(src, &id("dbml"), &id("sql")).unwrap();
    assert!(keep.fidelity.dropped.is_empty());
}

#[test]
fn relations_and_names_survive_dbml_to_prisma() {
    let reg = Registry::with_defaults();
    let src =
        "Table \"order items\" {\n  \"line no\" int [pk]\n  parent int\n  note: 'a note'\n}\n\
               Table u {\n  id int [pk]\n}\n\
               Ref r_parent: \"order items\".parent > \"order items\".\"line no\"\n\
               Ref: \"order items\".parent > u.id\n";
    let c = reg
        .convert_explicit(src, &id("dbml"), &id("prisma"))
        .unwrap();
    assert!(c.output.contains("@@map(\"order items\")"));
    assert!(c.output.contains("@map(\"line no\")"));
    assert!(c.output.contains("/// a note"));
    assert!(
        c.fidelity.tables.lossless() && c.fidelity.columns.lossless() && c.fidelity.refs.lossless()
    );
    let back = reg
        .convert_explicit(&c.output, &id("prisma"), &id("dbml"))
        .unwrap();
    assert!(back.output.contains("Ref r_parent:"));
    assert!(back.fidelity.idempotent);
}

#[test]
fn bad_prisma_is_a_clean_error() {
    let reg = Registry::with_defaults();
    let p = reg.codec(&id("prisma")).unwrap();
    assert!(p
        .parse("model A {\n  id Int\n")
        .unwrap_err()
        .to_string()
        .contains("not closed"));
    assert!(p.parse("model A { id Int }").is_err());
}

#[test]
fn prisma_is_auto_detected() {
    let reg = Registry::with_defaults();
    let c = reg.convert(SCHEMA, &id("dbml")).unwrap();
    assert!(c.output.contains("Table User"));
}
