// 


use crate::{instruction, opcode, Executor, Instruction, Opcode};

// Fixed-size circular buffer for prefetched instructions
pub struct InstructionPipeline {
    buffer: [Option<Instruction>; 16], // Power of 2 size for efficient wrapping
    head: usize,                       // Read position
    tail: usize,                       // Write position
    size: usize,                       // Current number of instructions in buffer
    capacity: usize,                   // Maximum capacity (always 16 in this case)
    branch_predictor: BranchPredictor, // Simple branch predictor
}

// Simple 2-bit saturating counter branch predictor
struct BranchPredictor {
    // Map PC -> prediction state (0-3)
    // 0-1: Not taken, 2-3: Taken
    prediction_table: fnv::FnvHashMap<u32, u8>, 
}

impl BranchPredictor {
    fn new() -> Self {
        Self {
            prediction_table: fnv::FnvHashMap::default(),
        }
    }
    
    fn predict(&self, pc: u32) -> bool {
        match self.prediction_table.get(&pc) {
            Some(&state) => state >= 2, // Predict taken if state is 2 or 3
            None => false,              // Default: predict not taken
        }
    }
    
    fn update(&mut self, pc: u32, taken: bool) {
        let state = self.prediction_table.entry(pc).or_insert(1);
        if taken {
            // Increment counter, saturate at 3
            *state = (*state + 1).min(3);
        } else {
            // Decrement counter, saturate at 0
            *state = state.saturating_sub(1);
        }
    }
}

impl InstructionPipeline {
    pub fn new() -> Self {
        Self {
            buffer: [None; 16],
            head: 0,
            tail: 0,
            size: 0,
            capacity: 16,
            branch_predictor: BranchPredictor::new(),
        }
    }

    #[inline]
    pub fn push(&mut self, instruction: Instruction) -> bool {
        if self.size == self.capacity {
            return false; // Buffer full
        }
        
        self.buffer[self.tail] = Some(instruction);
        self.tail = (self.tail + 1) & (self.capacity - 1); // Wrap using bitwise AND
        self.size += 1;
        true
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Instruction> {
        if self.size == 0 {
            return None; // Buffer empty
        }
        
        let instruction = self.buffer[self.head].take();
        self.head = (self.head + 1) & (self.capacity - 1); // Wrap using bitwise AND
        self.size -= 1;
        instruction
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.size == self.capacity
    }

    // Clear the buffer
    #[inline]
    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.size = 0;
        // No need to clear the actual buffer elements
    }

    // Prefetch instructions into the buffer
    pub fn prefetch(&mut self, executor: &Executor) {
        if self.is_full() {
            return;
        }

        let mut current_pc = executor.state.pc;
        
        // Try to fill the buffer, but limit iterations to avoid infinite loops
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 32; // Safety limit
        
        while !self.is_full() && iterations < MAX_ITERATIONS {
            iterations += 1;
            
            let next_offset = (current_pc - executor.program.pc_base) / 4;
            
            // Check if we're still within program bounds
            if current_pc.wrapping_sub(executor.program.pc_base) >= (executor.program.instructions.len() * 4) as u32 {
                break;
            }
            
            let instruction = executor.program.instructions[next_offset as usize];
            self.push(instruction);
            
            // Handle branches using prediction
            if is_control_flow_instruction(instruction.opcode) {
                if matches!(instruction.opcode, Opcode::JAL | Opcode::JALR) {
                    // For unconditional jumps, always take the jump
                    // Calculate target PC (simplified, in reality would need to decode the instruction)
                    // This is just a placeholder logic
                    current_pc = calculate_jump_target(instruction, current_pc);
                } else {
                    // For conditional branches, use the predictor
                    let taken = self.branch_predictor.predict(current_pc);
                    if taken {
                        current_pc = calculate_branch_target(instruction, current_pc);
                    } else {
                        current_pc += 4;
                    }
                }
            } else {
                // Regular instruction, move to next
                current_pc += 4;
            }
        }
    }
    
    // Update branch predictor based on actual execution
    pub fn update_branch_predictor(&mut self, pc: u32, taken: bool) {
        self.branch_predictor.update(pc, taken);
    }
}

// Helper function to calculate jump target PC
#[inline]
fn calculate_jump_target(instruction: Instruction, pc: u32) -> u32 {
    // This would require decoding the instruction to extract the immediate
    // Simplified placeholder
    pc.wrapping_add(4) // Just a placeholder
}

// Helper function to calculate branch target PC
#[inline]
fn calculate_branch_target(instruction: Instruction, pc: u32) -> u32 {
    // This would require decoding the instruction to extract the immediate
    // Simplified placeholder
    pc.wrapping_add(4) // Just a placeholder
}

/// Check if an instruction is a control flow instruction
#[inline]
pub fn is_control_flow_instruction(opcode: Opcode) -> bool {
    matches!(
        opcode,
        Opcode::BEQ
            | Opcode::BGE
            | Opcode::BGEU
            | Opcode::BLT
            | Opcode::BLTU
            | Opcode::BNE
            | Opcode::JAL
            | Opcode::JALR
    )
}