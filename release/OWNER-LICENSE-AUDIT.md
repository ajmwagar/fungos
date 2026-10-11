# Mycelium owner-source and notice audit

This is an artifact-linked evidence inventory, **not legal clearance**. Publication
of the existing standalone package/image remains blocked: third-party notices are
not embedded, and the checks below do not establish complete file-local notice
coverage. Do not turn `coverage=incomplete` into `reviewed-complete` automatically.

## Qualified bytes

| Input | Exact identity |
| --- | --- |
| Mycelium executable source | `10a3c18ebea8b9f3f7be981657957ac10a5a2f16` |
| Owner `.deb`, version `0.1.0+qualification5` | SHA256 `db6c2e929b127fa89146175ff063fb0bf43c1785ef9a08a398aee4230b218daa` |
| Static amd64 executable | SHA256 `60664012ba7b7b3a7c3ece05975de582caa9b6cc207a4f1b3ce535f17d3c2fa8` |
| Cargo.lock | SHA256 `1577e65d674b4ddfd132efec0bf11df0bc87ed1ec30c74bf843b01b342a42544` |
| Corrected installer ISO | SHA256 `601a631bb41cd93ef091e9b6a38b25d09bb49395a036f0bf668053dc9bde0b5c` |

No binary, package, ISO, production identity or fleet configuration was changed by
this audit. Later notice packaging needs truthful executable and notice/packager
provenance; a new packager commit is not the old executable's source revision.

## Completed observations

The clean owner source and lock were compared with the retained vendor closure.
`owner-license-audit` verified all **500 registry package checksums and per-file
checksums**. Cargo all-features metadata is a source-coverage superset, not the
actual linked dependency graph. The actual build was:

```sh
cargo zigbuild --locked --release --target x86_64-unknown-linux-musl \
  -p mycelium-cli --features iroh-sync
```

Actual tools: Rust `1.98.1`, commit
`48a229ceaefd4985c50990b14116b6d856af0985`, cargo-zigbuild `0.23.4`, Zig
`0.14.1`. Build-tool licensing is not automatically the application's licensing.

The successful v3 inventory has **19,061 source/notice file records** and **969
raw filename-selected notices**. It retains exact installed Rust library source,
compiler-builtins source, Rust COPYRIGHT documents and license texts, materialized
Zig runtime sources, and Rust's exact LLVM libunwind submodule
`52ed14fcd56afc30f9cccd8ca8ce237c2eef7e04`. The Rust std notice document is
`COPYRIGHT-library.html`, SHA256
`68129500b616d5838629e68f55ff3aed5e096dacf60ce9eb41bbe599a563afa6`.
[Rust describes the distinction between library and entire-toolchain notices](https://github.com/rust-lang/rust/blob/main/COPYRIGHT).

Actual native outputs include aws-lc-sys `0.45.0` with bundled AWS-LC **5.7.0**,
ring `0.17.14` with BoringSSL/ISC/file-local notices, and mlua-sys `0.10.0`
linking vendored **Lua 5.4.8** through lua-src `550.0.0`. LuaJIT and other Lua
versions in the source closure are not evidence of runtime selection.

Zig's installed include path points into Ubuntu LLVM20 headers, package
`libclang-common-20-dev:amd64` version `1:20.1.8-2ubuntu8`; the materialized
headers have distinct provenance, not assumed Zig-upstream provenance. Rust's
prebuilt musl `libunwind.a` is covered conservatively by the exact Rust LLVM
source, not relabelled as Zig's libunwind. Exact link selection remains a separate
review question.

## Missing-archive notices and remaining blockers

33 crate archives had no filename-selected notice. The supplementary collector
uses declared GitHub repository and published `.cargo_vcs_info.json` revision;
it never substitutes current main/master. Public API failures are retained as
UNKNOWN. Exact git/raw-source collection additionally retained rumqtt and russh
notices when API access was rate limited. In particular russh-util `0.52.0` uses
`LICENSE-2.0.txt` at `3d09c20c52a542ffa327723d3a44cb1edd9b0ca4`, not the
different latest license filename.

Still unresolved:

- enum-assoc `1.4.1`, exact revision
  `351b306939b150601451df752a183b077b0de85a`: MIT OR Apache-2.0 metadata, but no raw
  license/copyright notice found in the exact upstream root or source headers.
- pharos `0.5.3`, exact revision
  `f5b985a59cfeced5ae66325d477b6c982c9408bf`: README declares Unlicense,
  but no complete raw license text found in that source revision.
- File-local header attribution review is not proven by filename discovery.
  Complete original sources and published upstream test fixtures remain retained;
  these must not be stripped because some fixtures resemble keys.
- Matching Ubuntu LLVM20 source-package closure needs retention/verification.
- Raw notice payload must accompany independently downloadable `.deb` and ISO.
  A private cache or separately linked source archive does not prove this.

The exact Cargo and README declarations for those two crates are retained in
`mycelium-declaration-notices`, alongside SPDX license-list-data `v3.28.0` standard
MIT/Apache-2.0/Unlicense texts clearly labelled **supplements**, not upstream
LICENSE files. Author metadata is retained as metadata, not invented copyright
ownership. Full fetched git history contained no LICENSE/COPYING path for either
repository. These are documented evidence exceptions requiring explicit review,
not permission to fabricate missing statements.

Follow-up v4 collection retains an entire original source file as a `.NOTICE`
when its first 8 KiB contains copyright, SPDX identifier, grant or redistribution
markers. It does not guess/truncate an ambiguous comment boundary. This is a
conservative attribution evidence set, not a claim that every textual marker is
a binary license obligation; the compiler-wide and inactive dependencies must
remain distinguishable during runtime notice-payload review.

## Generic notice input and runbook

`notice-input` stages only raw notice-named files, preserving exact contents and
hashing every retained file. It always emits incomplete coverage until evidence
review resolves the blockers. The Mycelium-owned packager must validate all
bindings and inventory before any output write, rejecting incomplete coverage,
missing files, altered hashes, unsafe paths and symlinks.

```text
metadata.txt: schema_version=1, source_commit, executable_sha256,
              cargo_lock_sha256, inventory_sha256, coverage,
              legal_clearance=not-asserted
inventory.tsv: sha256<TAB>path (header), then raw notices/... file hashes
```

Private Agora cache root (no runtime identities or claims in these source inputs):
`/home/ajmwagar/.cache/fungos-release-source.20261010`.
`mycelium-owner-audit-v3` contains successful checksum/source evidence;
`mycelium-upstream-notices-v1` and `v2` retain source-revision provenance including
failed collection rows; `mycelium-exact-extra-notices` contains exact supplementary
rumqtt/russh notices. Initial v1/v2 owner audits were incomplete and are not proofs.

Staged `mycelium-notice-input-v1` has 1,044 raw notice files (27 MiB, including
conservative toolchain-wide notices), inventory SHA256
`8471f202f207f00bb00e08820928f33b62fe9d7c2acb6a431c7f5c124ca62304`.
Its `coverage=incomplete` must be rejected. Runtime-only selection should be
reviewed to avoid implying every compiler license applies to the executable.

Cheap validation used the existing Agora release workspace: cargo check, five
focused unit tests across the three tools, then release builds and real inventory
execution. These tools are deterministic collection helpers, not SPDX or legal
certification engines. The Debian media/source closure is documented separately.
