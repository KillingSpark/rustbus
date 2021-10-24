/// Errors that can occur while marshalling a value into a dbus message
#[derive(Debug, Eq, PartialEq)]
pub enum MarshalError {
    /// Tried to marshal a message with the "invalid" message type
    InvalidMessageType,
    /// Tried to marshal an empty UnixFd
    EmptyUnixFd,
    /// Error while trying to dup a UnixFd
    DupUnixFd(nix::Error),
    /// Errors occuring while validating the input
    Validation(crate::params::validation::Error),
}

//--------
// Conversion to MarshalError
//--------

impl From<crate::params::validation::Error> for MarshalError {
    fn from(e: crate::params::validation::Error) -> Self {
        MarshalError::Validation(e)
    }
}

impl From<crate::signature::Error> for MarshalError {
    fn from(e: crate::signature::Error) -> Self {
        MarshalError::Validation(crate::params::validation::Error::InvalidSignature(e))
    }
}

/// Errors that can  occur while unmarshaling a value from a dbus message
#[derive(Debug, PartialEq, Eq)]
pub enum UnmarshalError {
    /// Found an empty struct while unmarshalling
    EmptyStruct,
    /// There were not enough bytes in the buffer to unmarshal the value
    NotEnoughBytes,
    /// There were not enough bytes in the buffer to unmarshal the collection
    NotEnoughBytesForCollection,
    /// Unmarshalling a message did not use all bytes in the body
    NotAllBytesUsed,
    /// A message indicated an invalid byteorder in the header
    InvalidByteOrder,
    /// A message indicated an invalid message type
    InvalidMessageType,
    /// There was a mismatch between expected an encountered signatures
    /// (e.g. trying to unmarshal a string when there is a u64 in the message)
    WrongSignature,
    /// Any error encountered while validating input
    Validation(crate::params::validation::Error),
    /// A message contained an invalid header field
    InvalidHeaderField,
    /// A message contained an invalid header fields
    InvalidHeaderFields,
    /// A message contained unknown header fields
    UnknownHeaderField,
    /// Returned when data is encountered in padding between values. This is a sign of a corrupted message (or a bug in this library)
    PaddingContainedData,
    /// A boolean did contain something other than 0 or 1
    InvalidBoolean,
    /// No more values can be read from this message
    EndOfMessage,
    /// A message did not contain a signature for a header field
    NoSignature,
    /// A unix fd member had an index that is bigger than the size of the list of unix fds passed along with the message
    BadFdIndex(usize),
    /// When unmarshalling a Variant and there is not matching variant in the enum that had the unmarshal impl derived
    NoMatchingVariantFound,
}
