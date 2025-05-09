//! Programs that can be executed by the SP1 zkVM.

use std::{fs::File, io::Read};

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use sp1_primitives::types::IdentityBuildHasher;

use crate::{
    disassembler::{transpile, Elf},
    instruction::Instruction,
};

/// A program that can be executed by the SP1 zkVM.
///
/// Contains a series of instructions along with the initial memory image. It also contains the
/// start address and base address of the program.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Program {
    /// The instructions of the program.
    pub instructions: Vec<Instruction>,
    /// The start address of the program.
    pub pc_start: u32,
    /// The base address of the program.
    pub pc_base: u32,
    /// The initial memory image, useful for global constants.
    pub memory_image: HashMap<u32, u32, IdentityBuildHasher>,
}

impl Program {
    /// Create a new [Program].
    #[must_use]
    pub fn new(instructions: Vec<Instruction>, pc_start: u32, pc_base: u32) -> Self {
        Self {
            instructions,
            pc_start,
            pc_base,
            memory_image: HashMap::with_hasher(IdentityBuildHasher),
        }
    }

    /// Disassemble a RV32IM ELF to a program that be executed by the VM.
    ///
    /// # Errors
    ///
    /// This function may return an error if the ELF is not valid.
    pub fn from(input: &[u8]) -> eyre::Result<Self> {
        // Decode the bytes as an ELF.
        let elf = Elf::decode(input)?;

        // Transpile the RV32IM instructions.
        let instructions = transpile(&elf.instructions);

        // Return the program.
        Ok(Program {
            instructions,
            pc_start: elf.pc_start,
            pc_base: elf.pc_base,
            memory_image: elf.memory_image,
        })
    }

    /// Disassemble a RV32IM ELF to a program that be executed by the VM from a file path.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file cannot be opened or read.
    pub fn from_elf(path: &str) -> eyre::Result<Self> {
        let mut elf_code = Vec::new();
        File::open(path)?.read_to_end(&mut elf_code)?;
        Program::from(&elf_code)
    }
}
