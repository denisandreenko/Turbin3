use anchor_lang::error_code;

#[error_code]
pub enum MarkeplaceError{
    #[msg("Name must be between 1 and 33 characters")]
    NameTooLong,
}

