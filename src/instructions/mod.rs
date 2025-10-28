pub mod check_contributions;
pub mod contribute;
pub mod initialize;
pub mod refund;

pub use check_contributions::*;
pub use contribute::*;
pub use initialize::*;
pub use refund::*;

// #[repr(u8)]
pub enum FundraiserInstructions {
    Initialize = 0,
    Contribute = 1,
    CheckContributions = 2,
    Refund = 3,
}

// - intialize
// - contribute
// - check_contributions
// - refund
impl TryFrom<&u8> for FundraiserInstructions {
    type Error = pinocchio::program_error::ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FundraiserInstructions::Initialize),
            1 => Ok(FundraiserInstructions::Contribute),
            2 => Ok(FundraiserInstructions::CheckContributions),

            3 => Ok(FundraiserInstructions::Refund),
            _ => Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
        }
    }
}
