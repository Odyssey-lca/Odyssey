use std::fmt;

#[derive(Debug)]
pub enum Error {
    InvalidInput { details: String },
    NotInvertible,
    DimensionMismatch { expected: usize, actual: usize },
    NullPointer(&'static str),
    SymbolicAnalysisFailed,
    NumericFactorizationFailed,
    MatrixMulFailed,
    SolveFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidInput { details } => write!(f, "invalid input: {details}"),
            Error::NotInvertible => write!(f, "matrix is not invertible"),
            Error::DimensionMismatch { expected, actual } => {
                write!(f, "dimension mismatch: expected {expected}, got {actual}")
            }
            Error::NullPointer(name) => write!(f, "null pointer: {name}"),
            Error::SymbolicAnalysisFailed => write!(f, "symbolic analysis failed"),
            Error::NumericFactorizationFailed => write!(f, "numeric factorization failed"),
            Error::MatrixMulFailed => write!(f, "matrix multiplication failed"),
            Error::SolveFailed => write!(f, "solve returned a non null code"),
        }
    }
}

impl std::error::Error for Error {}
