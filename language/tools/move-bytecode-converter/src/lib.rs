// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use move_binary_format::CompiledModule;
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum BytecodeFormat {
    Json,
    Mv,
}

impl Display for BytecodeFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BytecodeFormat::Json => write!(f, "json"),
            BytecodeFormat::Mv => write!(f, "mv"),
        }
    }
}

impl BytecodeFormat {
    pub fn from_path(path: &Path) -> Result<Self> {
        match path.extension().and_then(|s| s.to_str()) {
            Some("json") => Ok(Self::Json),
            Some("mv") => Ok(Self::Mv),
            _ => bail!("Invalid file extension: {:?}", path.extension()),
        }
    }
}

#[derive(Debug)]
pub struct InputSource {
    path: PathBuf,
    format: BytecodeFormat,
}

impl InputSource {
    pub fn new(path: PathBuf) -> Result<Self> {
        let format = BytecodeFormat::from_path(&path)?;
        Ok(Self { path, format })
    }

    pub fn convert(&self, output_dir: &Path, verify: bool) -> Result<()> {
        let (compiled_module, output_format) = match self.format {
            BytecodeFormat::Json => {
                let json = std::fs::read_to_string(&self.path)?;
                let compiled_module: CompiledModule = serde_json::from_str(&json)?;
                (compiled_module, BytecodeFormat::Mv)
            }
            BytecodeFormat::Mv => {
                let bytes = std::fs::read(&self.path)?;
                let compiled_module = CompiledModule::deserialize(&bytes)?;
                (compiled_module, BytecodeFormat::Json)
            }
        };
        if verify{
            move_bytecode_verifier::verify_module(&compiled_module)?;
        }
        let output = match output_format {
            BytecodeFormat::Json => {
                serde_json::to_string(&compiled_module)?.as_bytes().to_vec()
            }
            BytecodeFormat::Mv => {
                let mut bytes = vec![];
                compiled_module.serialize(&mut bytes)?;
                bytes
            }
        };
        let mut output_path = output_dir.join(self.path.file_stem().unwrap());
        output_path.set_extension(output_format.to_string());
        std::fs::write(output_path, output)?;
        Ok(())
    }
}
