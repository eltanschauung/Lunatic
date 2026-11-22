// The SDK mirror of item_model retains imports from this private module.
pub use ::horizon_assets::DecodeError;

pub fn error(message: &str) -> DecodeError {
    DecodeError(message.to_owned())
}
