// Copyright 2023--present rgconf developers

//! Provenance leaves are RFC 9162 hashes from the `rfc9162` crate.
//!
//! rgconf does not hash a leaf itself. It builds the record bytes, then
//! calls `rfc9162::leaf_hash` and appends those bytes to `rfc9162_log::Store`.
//! Three operations the log does not offer yet are traits that return
//! [`Error::UpstreamPatch`].

use crate::edit::ParamEdit;
use crate::error::Error;
use crate::resolve::Resolution;
use crate::sbom::Component;
use rfc9162_log::{AuthenticatedLog, Store};
use rfc9162_tiles::layout::MAX_ENTRY_LEN;
use rfc9162_tiles::MemoryStore;
use std::sync::Arc;

pub const LEAF_MAGIC: &str = "rgconf-leaf/v1\n";

/// Generic patches `stackhands/rfc9162` v0.11.0 needs before rgconf can drop
/// these stubs. None of them are implemented here.
pub const UPSTREAM_PATCHES: &[(&str, &str)] = &[
    (
        "rfc9162::canonicalize_capnp",
        "leaf_hash hashes the bytes it is given. It does not canonicalize a Cap'n Proto message. Add a function that packs segments, lands pointers, and drops trailing zeroes, and have callers hash that result.",
    ),
    (
        "rfc9162_log::Store::put_blob_and_append",
        "append refuses an entry longer than 65535 bytes. Add a method that stores the bytes as a blob and appends a short pointer leaf, returning that leaf's index. Sidecar::put already writes a file and does not do this.",
    ),
    (
        "rfc9162_tiles::Writer::append_shared",
        "Writer holds treeState.lock from open until drop. Add an append that does not keep that lock across idle time, so two tasks can append without a private lock copy.",
    ),
    (
        "rfc9162-c-abi and rfc9162-py",
        "C and Python bindings for the log belong in the rfc9162 repository. rgconf does not ship a second copy.",
    ),
];

/// Cap'n Proto canonicalization is an upstream gap. Calling this is the stub.
pub trait CanonicalLeaf {
    fn capnp_canonical(&self, message: &[u8]) -> Result<Vec<u8>, Error>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MissingCapnpCanonicalization;

impl CanonicalLeaf for MissingCapnpCanonicalization {
    fn capnp_canonical(&self, _message: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::UpstreamPatch {
            function: UPSTREAM_PATCHES[0].0,
            detail: UPSTREAM_PATCHES[0].1,
        })
    }
}

/// Appending a record longer than the tile entry limit.
pub trait BlobAppend {
    fn put_blob_and_append(&mut self, bytes: &[u8]) -> Result<u64, Error>;
}

/// An append that does not hold the writer lock across idle time.
pub trait SharedAppend {
    fn append_shared(&mut self, bytes: &[u8]) -> Result<u64, Error>;
}

pub struct ProvenanceLog {
    store: Store<MemoryStore>,
    bodies: Vec<Vec<u8>>,
    hashes: Vec<[u8; 32]>,
}

impl ProvenanceLog {
    pub fn open() -> Result<Self, Error> {
        let keys = Arc::new(|entry: &[u8]| leaf_keys(entry));
        let store = Store::open(MemoryStore::new(), keys).map_err(log_error)?;
        Ok(Self {
            store,
            bodies: Vec::new(),
            hashes: Vec::new(),
        })
    }

    pub fn append_record(&mut self, kind: &str, key: &str, body: &[u8]) -> Result<u64, Error> {
        let material = record_bytes(kind, key, body);
        self.append_material(material)
    }

    pub fn append_config(&mut self, resolution: &Resolution) -> Result<u64, Error> {
        let profile = format!("profile {}", resolution.profile.name());
        let mut last = self.append_record("config", "profile", profile.as_bytes())?;
        for (key, field) in &resolution.fields {
            let body = format!("{} [{}]", field.value, field.status.name());
            last = self.append_record("config", key, body.as_bytes())?;
        }
        Ok(last)
    }

    pub fn append_sbom(&mut self, components: &[Component]) -> Result<u64, Error> {
        let mut body = String::new();
        for component in components {
            body.push_str(&component.purl);
            body.push('\n');
        }
        self.append_record("sbom", "cyclonedx", body.as_bytes())
    }

    pub fn append_archive(&mut self, name: &str, digest: &[u8]) -> Result<u64, Error> {
        self.append_record("archive", name, digest)
    }

    pub fn append_edit(&mut self, edit: &ParamEdit) -> Result<u64, Error> {
        let body = format!(
            "{} {} -> {} via {}",
            edit.key,
            edit.previous,
            edit.next,
            edit.origin.name()
        );
        self.append_record("edit", &edit.key, body.as_bytes())
    }

    pub fn root_matches_rfc9162(&self) -> bool {
        self.store.root() == rfc9162::root(&self.hashes)
    }

    pub fn inclusion_ok(&self, index: usize) -> bool {
        let Some(leaf) = self.hashes.get(index) else {
            return false;
        };
        let Some(path) = rfc9162::inclusion_proof(&self.hashes, index) else {
            return false;
        };
        rfc9162::verify_inclusion(
            leaf,
            index,
            self.hashes.len(),
            &path,
            &rfc9162::hex(&rfc9162::root(&self.hashes)),
        )
    }

    pub fn len(&self) -> usize {
        self.hashes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hashes.is_empty()
    }

    fn append_material(&mut self, material: Vec<u8>) -> Result<u64, Error> {
        if material.len() > MAX_ENTRY_LEN {
            return self.put_blob_and_append(&material);
        }
        let hash = rfc9162::leaf_hash(&material);
        let index = self.store.append(&material).map_err(log_error)?;
        self.bodies.push(material);
        self.hashes.push(hash);
        Ok(index)
    }
}

impl BlobAppend for ProvenanceLog {
    fn put_blob_and_append(&mut self, bytes: &[u8]) -> Result<u64, Error> {
        if bytes.len() <= MAX_ENTRY_LEN {
            let hash = rfc9162::leaf_hash(bytes);
            let index = self.store.append(bytes).map_err(log_error)?;
            self.hashes.push(hash);
            self.bodies.push(bytes.to_vec());
            return Ok(index);
        }
        Err(Error::UpstreamPatch {
            function: UPSTREAM_PATCHES[1].0,
            detail: UPSTREAM_PATCHES[1].1,
        })
    }
}

impl SharedAppend for ProvenanceLog {
    fn append_shared(&mut self, _bytes: &[u8]) -> Result<u64, Error> {
        Err(Error::UpstreamPatch {
            function: UPSTREAM_PATCHES[2].0,
            detail: UPSTREAM_PATCHES[2].1,
        })
    }
}

pub fn record_bytes(kind: &str, key: &str, body: &[u8]) -> Vec<u8> {
    let mut out = LEAF_MAGIC.as_bytes().to_vec();
    out.extend_from_slice(kind.as_bytes());
    out.push(b'\n');
    out.extend_from_slice(key.as_bytes());
    out.push(b'\n');
    out.extend_from_slice(body);
    out
}

fn leaf_keys(entry: &[u8]) -> Vec<Vec<u8>> {
    let text = String::from_utf8_lossy(entry);
    let mut lines = text.lines();
    let _magic = lines.next();
    let kind = match lines.next() {
        Some(line) => line,
        None => "unknown",
    };
    let key = match lines.next() {
        Some(line) => line,
        None => "",
    };
    vec![
        format!("kind={kind}").into_bytes(),
        format!("param={key}").into_bytes(),
    ]
}

fn log_error(err: rfc9162_log::Error) -> Error {
    Error::message(format!("rfc9162 log: {err}"))
}
