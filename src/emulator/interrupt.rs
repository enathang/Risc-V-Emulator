pub const MASK_INTERRUPT_BIT: u64 = 1 << 63;

#[derive(Debug, Copy, Clone)]
pub enum Interrupt {
    SupervisorSoftwareInterrupt,
    MachineSoftwareInterrupt,
    SupervisorTimerInterrupt,
    MachineTimerInterrupt,
    SupervisorExternalInterrupt,
    MachineExternalInterrupt,
}

impl Interrupt {
    pub fn code(&self) -> u64 {
        // [INTERRUPT_FLAG][INTERRUPT_CODE]
        let interrupt_code = match(self) {
            SupervisorSoftwareInterrupt => 1,
            MachineSoftwareInterrupt => 3,
            SupervisorTimerInterrupt => 5,
            MachineTimerInterrupt => 7,
            SupervisorExternalInterrupt => 9,
            MachineExternalInterrupt => 11,
        };

        // Set interrupt flag to 1 so we know the code should be interpreted as an
        // interrupt code and not an error code
        return interrupt_code | MASK_INTERRUPT_BIT;
    }
}
