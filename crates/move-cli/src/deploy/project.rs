// Copyright 2023-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/cargo-stylus/blob/main/licenses/COPYRIGHT.md

use crate::constants::{BROTLI_COMPRESSION_LEVEL, EOF_PREFIX_NO_DICT, PROJECT_HASH_SECTION_NAME};
use crate::deploy::greyln;
use brotli2::read::BrotliEncoder;
use eyre::{Result, WrapErr, eyre};
use std::ops::Range;
use std::{fs, io::Read, path::PathBuf};
use wasm_encoder::{Module, RawSection};
use wasmparser::{Parser, Payload};

/// Reads a WASM file at a specified path and returns its brotli compressed bytes.
pub fn compress_wasm(wasm: &PathBuf, project_hash: [u8; 32]) -> Result<(Vec<u8>, Vec<u8>)> {
    let wasm =
        fs::read(wasm).wrap_err_with(|| eyre!("failed to read Wasm {}", wasm.to_string_lossy()))?;

    // We convert the WASM from binary to text and back to binary as this trick removes any dangling
    // mentions of reference types in the wasm body, which are not yet supported by Arbitrum chain backends.
    let wat_str =
        wasmprinter::print_bytes(&wasm).map_err(|e| eyre!("failed to convert Wasm to Wat: {e}"))?;
    let wasm = wasmer::wat2wasm(wat_str.as_bytes())
        .map_err(|e| eyre!("failed to convert Wat to Wasm: {e}"))?;

    // We include the project's hash as a custom section
    // in the user's WASM so it can be verified by Cargo stylus'
    // reproducible verification. This hash is added as a section that is
    // ignored by WASM runtimes, so it will only exist in the file
    // for metadata purposes.
    let wasm = add_project_hash_to_wasm_file(&wasm, project_hash)
        .wrap_err("failed to add project hash to wasm file as custom section")?;

    let wasm =
        strip_user_metadata(&wasm).wrap_err("failed to strip user metadata from wasm file")?;

    let wasm = wasmer::wat2wasm(&wasm).wrap_err("failed to parse Wasm")?;

    let mut compressor = BrotliEncoder::new(&*wasm, BROTLI_COMPRESSION_LEVEL);
    let mut compressed_bytes = vec![];
    compressor
        .read_to_end(&mut compressed_bytes)
        .wrap_err("failed to compress WASM bytes")?;

    let mut contract_code = hex::decode(EOF_PREFIX_NO_DICT).unwrap();
    contract_code.extend(compressed_bytes);

    Ok((wasm.to_vec(), contract_code))
}

// Adds the hash of the project's source files to the wasm as a custom section
// if it does not already exist. This allows for reproducible builds by cargo stylus
// for all Rust stylus contracts. See `cargo stylus verify --help` for more information.
fn add_project_hash_to_wasm_file(
    wasm_file_bytes: &[u8],
    project_hash: [u8; 32],
) -> Result<Vec<u8>> {
    let section_exists = has_project_hash_section(wasm_file_bytes)?;
    if section_exists {
        greyln!(
            "Wasm file bytes already contains a custom section with a project hash, not overwriting'"
        );
        return Ok(wasm_file_bytes.to_vec());
    }
    Ok(add_custom_section(wasm_file_bytes, project_hash))
}

pub fn has_project_hash_section(wasm_file_bytes: &[u8]) -> Result<bool> {
    let parser = wasmparser::Parser::new(0);
    for payload in parser.parse_all(wasm_file_bytes) {
        if let wasmparser::Payload::CustomSection(reader) = payload? {
            if reader.name() == PROJECT_HASH_SECTION_NAME {
                println!(
                    "Found the project hash custom section name {}",
                    hex::encode(reader.data())
                );
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn add_custom_section(wasm_file_bytes: &[u8], project_hash: [u8; 32]) -> Vec<u8> {
    let mut bytes = vec![];
    bytes.extend_from_slice(wasm_file_bytes);
    wasm_gen::write_custom_section(&mut bytes, PROJECT_HASH_SECTION_NAME, &project_hash);
    bytes
}

fn strip_user_metadata(wasm_file_bytes: &[u8]) -> Result<Vec<u8>> {
    let mut module = Module::new();
    // Parse the input WASM and iterate over the sections
    let parser = Parser::new(0);
    for payload in parser.parse_all(wasm_file_bytes) {
        match payload? {
            Payload::CustomSection { .. } => {
                // Skip custom sections to remove sensitive metadata
                greyln!("stripped custom section from user wasm to remove any sensitive data");
            }
            Payload::UnknownSection { .. } => {
                // Skip unknown sections that might not be sensitive
                println!("stripped unknown section from user wasm to remove any sensitive data");
            }
            item => {
                // Handle other sections as normal.
                if let Some(section) = item.as_section() {
                    let (id, range): (u8, Range<usize>) = section;
                    let data_slice = &wasm_file_bytes[range.start..range.end]; // Start at the beginning of the range
                    let raw_section = RawSection {
                        id,
                        data: data_slice,
                    };
                    module.section(&raw_section);
                }
            }
        }
    }
    // Return the stripped WASM binary
    Ok(module.finish())
}
