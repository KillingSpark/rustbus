use thiserror::Error;

/// Errors that can occur while marshalling a value into a dbus message
#[derive(Debug, Eq, PartialEq, Error)]
pub enum MarshalError {
    /// Tried to marshal a message with the "invalid" message type
    #[error("Tried to marshal a message with the 'invalid' message type")]
    InvalidMessageType,
    /// Tried to marshal an empty UnixFd
    #[error("Tried to marshal an empty UnixFd")]
    EmptyUnixFd,
    /// Error while trying to dup a UnixFd
    #[error("Error while trying to dup a UnixFd: {0}")]
    DupUnixFd(std::io::ErrorKind),
    /// Errors occuring while validating the input
    #[error("Errors occured while validating: {0}")]
    Validation(#[from] crate::params::validation::Error),
}

//--------
// Conversion to MarshalError
//--------

impl From<crate::signature::Error> for MarshalError {
    fn from(e: crate::signature::Error) -> Self {
        MarshalError::Validation(crate::params::validation::Error::InvalidSignature(e))
    }
}

/// Errors that can  occur while unmarshaling a value from a dbus message
#[derive(Debug, PartialEq, Eq, Error)]
pub enum UnmarshalError {
    /// Found an empty struct while unmarshalling
    #[error("Found an empty struct while unmarshalling")]
    EmptyStruct,
    /// There were not enough bytes in the buffer to unmarshal the value
    #[error("There were not enough bytes in the buffer to unmarshal the value")]
    NotEnoughBytes,
    /// There were not enough bytes in the buffer to unmarshal the collection
    #[error("There were not enough bytes in the buffer to unmarshal the collection")]
    NotEnoughBytesForCollection,
    /// Unmarshalling a message did not use all bytes in the body
    #[error("Unmarshalling a message did not use all bytes in the body")]
    NotAllBytesUsed,
    /// A message indicated an invalid byteorder in the header
    #[error("A message indicated an invalid byteorder in the header")]
    InvalidByteOrder,
    /// A message has an invalid (zero) serial in the header
    #[error("A message has an invalid (zero) serial in the header")]
    InvalidSerial,
    /// A message indicated an invalid message type
    #[error("A message indicated an invalid message type")]
    InvalidMessageType,
    /// There was a mismatch between expected an encountered signatures
    /// (e.g. trying to unmarshal a string when there is a u64 in the message)
    #[error("There was a mismatch between expected an encountered signatures")]
    WrongSignature,
    /// Any error encountered while validating input
    #[error("Error encountered while validating input: {0}")]
    Validation(#[from] crate::params::validation::Error),
    /// A message contained an invalid header field
    #[error("A message contained an invalid header field")]
    InvalidHeaderField,
    /// A message contained an invalid header fields
    #[error("A message contained an invalid header fields")]
    InvalidHeaderFields,
    /// A message contained unknown header fields
    #[error("A message contained unknown header fields")]
    UnknownHeaderField,
    /// Returned when data is encountered in padding between values. This is a sign of a corrupted message (or a bug in this library)
    #[error("Returned when data is encountered in padding between values. This is a sign of a corrupted message (or a bug in this library)")]
    PaddingContainedData,
    /// A boolean did contain something other than 0 or 1
    #[error("A boolean did contain something other than 0 or 1")]
    InvalidBoolean,
    /// No more values can be read from this message
    #[error("No more values can be read from this message")]
    EndOfMessage,
    /// A message did not contain a signature for a header field
    #[error("A message did not contain a signature for a header field")]
    NoSignature,
    /// A unix fd member had an index that is bigger than the size of the list of unix fds passed along with the message
    #[error("A unix fd member had an index that is bigger than the size of the list of unix fds passed along with the message")]
    BadFdIndex(usize),
    /// When unmarshalling a Variant and there is not matching variant in the enum that had the unmarshal impl derived
    #[error("When unmarshalling a Variant and there is not matching variant in the enum that had the unmarshal impl derived")]
    NoMatchingVariantFound,
}
