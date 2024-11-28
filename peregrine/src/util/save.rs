#![allow(dead_code)]
use std::{fs, path::Path};
use serde::{Serialize, Deserialize};
use anyhow::Result;

pub trait Save<Output, Additional> : Serialize + for<'de> Deserialize<'de> {
    fn build(self, _additional: Additional) -> Output;
    fn from_bytes(bytes: &[u8], additional: Additional) -> Result<Output> {
        let template = bincode::deserialize(bytes)?;
        Ok(Self::build(template, additional))
    }
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let output = bincode::serialize(self)?;
        Ok(output)
    }
    fn from_file(path: &Path, additional: Additional) -> Result<Output> {
        let bytes = fs::read(path)?;
        Self::from_bytes(&bytes, additional)
    }
    fn to_file(&self, path: &Path) -> Result<()> {
        let bytes = self.to_bytes()?;
        fs::write(path, &bytes)?;
        Ok(())
    }
}