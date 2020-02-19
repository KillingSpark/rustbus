//! lowlevel message stuff

use crate::signature;
use std::os::unix::io::RawFd;

#[derive(Copy, Clone, Debug)]
pub enum MessageType {
    Signal,
    Error,
    Call,
    Reply,
    Invalid,
}

/// The base types a message can have as parameters
/// There are From<T> impls for most of them
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Base {
    Double(u64),
    Byte(u8),
    Int16(i16),
    Uint16(u16),
    Int32(i32),
    Uint32(u32),
    UnixFd(u32),
    Int64(i64),
    Uint64(u64),
    String(String),
    Signature(String),
    ObjectPath(String),
    Boolean(bool),
}

pub type DictMap = std::collections::HashMap<Base, Param>;

/// The container types a message can have as parameters
#[derive(Debug, Clone)]
pub enum Container {
    Array(Array),
    Struct(Vec<Param>),
    Dict(Dict),
    Variant(Box<Variant>),
}

impl Container {
    pub fn make_array(element_sig: &str, elements: Vec<Param>) -> Result<Container> {
        let mut sigs =
            signature::Type::parse_description(element_sig).map_err(|_| Error::InvalidSignature)?;

        if sigs.len() != 1 {
            return Err(Error::InvalidSignature);
        }

        let sig = sigs.remove(0);
        let arr = Array {
            element_sig: sig,
            values: elements,
        };
        Ok(Container::Array(arr))
    }

    pub fn make_dict(key_sig: &str, val_sig: &str, map: DictMap) -> Result<Container> {
        let mut valsigs =
            signature::Type::parse_description(val_sig).map_err(|_| Error::InvalidSignature)?;

        if valsigs.len() != 1 {
            return Err(Error::InvalidSignature);
        }

        let value_sig = valsigs.remove(0);
        let mut keysigs =
            signature::Type::parse_description(key_sig).map_err(|_| Error::InvalidSignature)?;

        if keysigs.len() != 1 {
            return Err(Error::InvalidSignature);
        }
        let key_sig = keysigs.remove(0);
        let key_sig = if let signature::Type::Base(sig) = key_sig {
            sig
        } else {
            return Err(Error::InvalidSignature);
        };

        let dict = Dict {
            key_sig,
            value_sig,
            map,
        };
        Ok(Container::Dict(dict))
    }
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub sig: signature::Type,
    pub value: Param,
}

#[derive(Debug, Clone)]
pub struct Array {
    pub element_sig: signature::Type,
    pub values: Vec<Param>,
}

#[derive(Debug, Clone)]
pub struct Dict {
    pub key_sig: signature::Base,
    pub value_sig: signature::Type,
    pub map: DictMap,
}

/// The Types a message can have as parameters
/// There are From<T> impls for most of the Base ones
#[derive(Debug, Clone)]
pub enum Param {
    Base(Base),
    Container(Container),
}

/// A message with all the different fields it may or may not have
#[derive(Debug, Clone)]
pub struct Message {
    pub typ: MessageType,
    pub interface: Option<String>,
    pub member: Option<String>,
    pub object: Option<String>,
    pub destination: Option<String>,
    pub params: Vec<Param>,
    pub serial: Option<u32>,
    pub response_serial: Option<u32>,
    pub sender: Option<String>,
    pub error_name: Option<String>,

    pub num_fds: Option<u32>,
    pub raw_fds: Vec<RawFd>,
}

impl Default for Message {
    fn default() -> Self {
        Self::new()
    }
}

impl Message {
    /// Create a new empty message
    pub fn new() -> Message {
        Message {
            typ: MessageType::Invalid,
            interface: None,
            member: None,
            params: Vec::new(),
            object: None,
            destination: None,
            serial: None,
            raw_fds: Vec::new(),
            num_fds: None,
            response_serial: None,
            sender: None,
            error_name: None,
        }
    }

    pub fn set_interface(&mut self, interface: String) {
        self.interface = Some(interface);
    }
    pub fn set_member(&mut self, member: String) {
        self.member = Some(member);
    }
    pub fn set_object(&mut self, object: String) {
        self.object = Some(object);
    }
    pub fn set_destination(&mut self, dest: String) {
        self.destination = Some(dest);
    }
    pub fn push_params(&mut self, params: Vec<Param>) {
        self.params.extend(params);
    }

    /// Make a correctly addressed response with the correct response serial
    pub fn make_response(&self) -> Message {
        Message {
            typ: MessageType::Reply,
            interface: None,
            member: None,
            params: Vec::new(),
            object: None,
            destination: self.sender.clone(),
            serial: None,
            raw_fds: Vec::new(),
            num_fds: None,
            sender: None,
            response_serial: self.serial,
            error_name: None,
        }
    }

    /// Make a correctly addressed error response with the correct response serial
    pub fn make_error_response(&self, error_name: String) -> Message {
        Message {
            typ: MessageType::Reply,
            interface: None,
            member: None,
            params: Vec::new(),
            object: None,
            destination: self.sender.clone(),
            serial: None,
            raw_fds: Vec::new(),
            num_fds: None,
            sender: None,
            response_serial: self.serial,
            error_name: Some(error_name),
        }
    }
}

impl Param {
    pub fn make_signature(&self, buf: &mut String) -> Result<()> {
        match self {
            Param::Base(b) => b.make_signature(buf),
            Param::Container(c) => c.make_signature(buf)?,
        }
        Ok(())
    }
}

impl Base {
    pub fn make_signature(&self, buf: &mut String) {
        match self {
            Base::Boolean(_) => buf.push('b'),
            Base::Double(_) => buf.push('d'),
            Base::Byte(_) => buf.push('y'),
            Base::Int16(_) => buf.push('n'),
            Base::Uint16(_) => buf.push('q'),
            Base::Int32(_) => buf.push('i'),
            Base::Uint32(_) => buf.push('u'),
            Base::UnixFd(_) => buf.push('h'),
            Base::Int64(_) => buf.push('x'),
            Base::Uint64(_) => buf.push('t'),
            Base::ObjectPath(_) => buf.push('o'),
            Base::String(_) => buf.push('s'),
            Base::Signature(_) => buf.push('g'),
        }
    }
}
impl Container {
    pub fn make_signature(&self, buf: &mut String) -> Result<()> {
        match self {
            Container::Array(elements) => {
                buf.push('a');
                elements.element_sig.to_str(buf);
            }
            Container::Dict(map) => {
                buf.push('a');
                buf.push('{');
                map.key_sig.to_str(buf);
                map.value_sig.to_str(buf);
                buf.push('}');
            }
            Container::Struct(elements) => {
                buf.push('(');
                for el in elements {
                    el.make_signature(buf)?;
                }
                buf.push(')');
            }
            Container::Variant(_) => {
                buf.push('v');
            }
        }
        Ok(())
    }
}

/// The different errors that can occur when dealing with messages
#[derive(Debug)]
pub enum Error {
    InvalidObjectPath,
    InvalidSignature,
    InvalidBusname,
    InvalidErrorname,
    InvalidMembername,
    InvalidInterface,
    InvalidHeaderFields,
    DuplicatedHeaderFields,
    NoSerial,
    InvalidType,
    ArrayElementTypesDiffer,
    DictKeyTypesDiffer,
    DictValueTypesDiffer,
}

/// The supported byte orders
#[derive(Clone, Copy, Debug)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

pub enum HeaderFlags {
    NoReplyExpected,
    NoAutoStart,
    AllowInteractiveAuthorization,
}

/// The different header fields a message may or maynot have
#[derive(Debug)]
pub enum HeaderField {
    Path(String),
    Interface(String),
    Member(String),
    ErrorName(String),
    ReplySerial(u32),
    Destination(String),
    Sender(String),
    Signature(String),
    UnixFds(u32),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn validate_object_path(op: &str) -> Result<()> {
    if op.is_empty() {
        return Err(Error::InvalidObjectPath);
    }
    if !op.starts_with('/') {
        return Err(Error::InvalidObjectPath);
    }
    if op.len() > 1 {
        let split = op.split('/').collect::<Vec<_>>();
        if split.len() < 2 {
            return Err(Error::InvalidObjectPath);
        }
        for element in &split[1..] {
            if element.is_empty() {
                return Err(Error::InvalidObjectPath);
            }
            if let Some(true) = element.chars().nth(0).map(|c| c.is_numeric()) {
                return Err(Error::InvalidObjectPath);
            }
            let alphanum_or_underscore = element.chars().all(|c| c.is_alphanumeric() || c == '_');
            if !alphanum_or_underscore {
                return Err(Error::InvalidObjectPath);
            }
        }
    }
    Ok(())
}
pub fn validate_interface(int: &str) -> Result<()> {
    if int.len() < 3 {
        return Err(Error::InvalidInterface);
    }
    if !int.contains('.') {
        return Err(Error::InvalidInterface);
    }

    let split = int.split('.').collect::<Vec<_>>();
    if split.len() < 2 {
        return Err(Error::InvalidInterface);
    }
    for element in split {
        if element.is_empty() {
            return Err(Error::InvalidInterface);
        }
        if let Some(true) = element.chars().nth(0).map(|c| c.is_numeric()) {
            return Err(Error::InvalidInterface);
        }
        let alphanum_or_underscore = element.chars().all(|c| c.is_alphanumeric() || c == '_');
        if !alphanum_or_underscore {
            return Err(Error::InvalidInterface);
        }
    }

    Ok(())
}

pub fn validate_errorname(en: &str) -> Result<()> {
    validate_interface(en).map_err(|_| Error::InvalidErrorname)
}

pub fn validate_busname(bn: &str) -> Result<()> {
    if bn.len() < 3 {
        return Err(Error::InvalidBusname);
    }
    if !bn.contains('.') {
        return Err(Error::InvalidBusname);
    }

    let (unique, bn) = if bn.chars().nth(0).unwrap() == ':' {
        (true, &bn[1..])
    } else {
        (false, &bn[..])
    };

    let split = bn.split('.').collect::<Vec<_>>();
    if split.len() < 2 {
        return Err(Error::InvalidBusname);
    }

    for element in split {
        if element.is_empty() {
            return Err(Error::InvalidBusname);
        }
        if !unique && element.chars().nth(0).map(|c| c.is_numeric()) == Some(true) {
            return Err(Error::InvalidBusname);
        }
        let alphanum_or_underscore_or_dash = element
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-');
        if !alphanum_or_underscore_or_dash {
            return Err(Error::InvalidBusname);
        }
    }

    Ok(())
}

pub fn validate_membername(mem: &str) -> Result<()> {
    if mem.is_empty() {
        return Err(Error::InvalidBusname);
    }

    let alphanum_or_underscore = mem.chars().all(|c| c.is_alphanumeric() || c == '_');
    if !alphanum_or_underscore {
        return Err(Error::InvalidMembername);
    }

    Ok(())
}

pub fn validate_signature(sig: &str) -> Result<()> {
    if signature::Type::parse_description(sig).is_err() {
        Err(Error::InvalidSignature)
    } else {
        Ok(())
    }
}

pub fn validate_array(array: &Array) -> Result<()> {
    // TODO check that all elements have the same type
    if array.values.is_empty() {
        return Ok(());
    }
    let mut first_sig = String::new();
    array.element_sig.to_str(&mut first_sig);
    let mut element_sig = String::new();
    for el in &array.values {
        element_sig.clear();
        el.make_signature(&mut element_sig)?;
        if !element_sig.eq(&first_sig) {
            return Err(Error::ArrayElementTypesDiffer);
        }
    }
    Ok(())
}

pub fn validate_dict(dict: &Dict) -> Result<()> {
    // TODO check that all elements have the same type
    if dict.map.is_empty() {
        return Ok(());
    }
    // check key sigs
    let mut first_sig = String::new();
    dict.key_sig.to_str(&mut first_sig);
    let mut element_sig = String::new();
    for el in dict.map.keys() {
        element_sig.clear();
        el.make_signature(&mut element_sig);
        if !element_sig.eq(&first_sig) {
            return Err(Error::DictKeyTypesDiffer);
        }
    }

    // check value sigs
    let mut first_sig = String::new();
    dict.value_sig.to_str(&mut first_sig);
    let mut element_sig = String::new();
    for el in dict.map.values() {
        element_sig.clear();
        el.make_signature(&mut element_sig)?;
        if !element_sig.eq(&first_sig) {
            return Err(Error::DictValueTypesDiffer);
        }
    }
    Ok(())
}

pub fn validate_header_fields(msg_type: MessageType, header_fields: &[HeaderField]) -> Result<()> {
    let mut have_path = false;
    let mut have_interface = false;
    let mut have_member = false;
    let mut have_errorname = false;
    let mut have_replyserial = false;
    let mut have_destination = false;
    let mut have_sender = false;
    let mut have_signature = false;
    let mut have_unixfds = false;

    for h in header_fields {
        match h {
            HeaderField::Destination(_) => {
                if have_destination {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_destination = true;
            }
            HeaderField::ErrorName(_) => {
                if have_errorname {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_errorname = true;
            }
            HeaderField::Interface(_) => {
                if have_interface {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_interface = true;
            }
            HeaderField::Member(_) => {
                if have_member {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_member = true;
            }
            HeaderField::Path(_) => {
                if have_path {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_path = true;
            }
            HeaderField::ReplySerial(_) => {
                if have_replyserial {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_replyserial = true;
            }
            HeaderField::Sender(_) => {
                if have_sender {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_sender = true;
            }
            HeaderField::Signature(_) => {
                if have_signature {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_signature = true;
            }
            HeaderField::UnixFds(_) => {
                if have_unixfds {
                    return Err(Error::DuplicatedHeaderFields);
                }
                have_unixfds = true;
            }
        }
    }

    let valid = match msg_type {
        MessageType::Invalid => false,
        MessageType::Call => have_path && have_member,
        MessageType::Signal => have_path && have_member && have_interface,
        MessageType::Reply => have_replyserial,
        MessageType::Error => have_errorname && have_replyserial,
    };
    if valid {
        Ok(())
    } else {
        Err(Error::InvalidHeaderFields)
    }
}

//
//
// Param TO
//
//

impl std::convert::From<&Param> for signature::Type {
    fn from(p: &Param) -> crate::signature::Type {
        match p {
            Param::Base(b) => signature::Type::Base(b.into()),
            Param::Container(c) => signature::Type::Container(c.into()),
        }
    }
}

//
//
// Param FROM
//
//

impl std::convert::From<Base> for Param {
    fn from(s: Base) -> Self {
        Param::Base(s)
    }
}
impl std::convert::From<Container> for Param {
    fn from(s: Container) -> Self {
        Param::Container(s)
    }
}

impl std::convert::From<bool> for Param {
    fn from(s: bool) -> Self {
        Param::Base(Base::Boolean(s))
    }
}
impl std::convert::From<String> for Param {
    fn from(s: String) -> Self {
        Param::Base(Base::String(s))
    }
}
impl std::convert::From<u8> for Param {
    fn from(s: u8) -> Self {
        Param::Base(Base::Byte(s))
    }
}
impl std::convert::From<u16> for Param {
    fn from(s: u16) -> Self {
        Param::Base(Base::Uint16(s))
    }
}
impl std::convert::From<u32> for Param {
    fn from(s: u32) -> Self {
        Param::Base(Base::Uint32(s))
    }
}
impl std::convert::From<u64> for Param {
    fn from(s: u64) -> Self {
        Param::Base(Base::Uint64(s))
    }
}
impl std::convert::From<i16> for Param {
    fn from(s: i16) -> Self {
        Param::Base(Base::Int16(s))
    }
}
impl std::convert::From<i32> for Param {
    fn from(s: i32) -> Self {
        Param::Base(Base::Int32(s))
    }
}
impl std::convert::From<i64> for Param {
    fn from(s: i64) -> Self {
        Param::Base(Base::Int64(s))
    }
}

//
//
// Base FROM
//
//

impl std::convert::From<bool> for Base {
    fn from(s: bool) -> Self {
        Base::Boolean(s)
    }
}
impl std::convert::From<String> for Base {
    fn from(s: String) -> Self {
        Base::String(s)
    }
}
impl std::convert::From<u8> for Base {
    fn from(s: u8) -> Self {
        Base::Byte(s)
    }
}
impl std::convert::From<u16> for Base {
    fn from(s: u16) -> Self {
        Base::Uint16(s)
    }
}
impl std::convert::From<u32> for Base {
    fn from(s: u32) -> Self {
        Base::Uint32(s)
    }
}
impl std::convert::From<u64> for Base {
    fn from(s: u64) -> Self {
        Base::Uint64(s)
    }
}
impl std::convert::From<i16> for Base {
    fn from(s: i16) -> Self {
        Base::Int16(s)
    }
}
impl std::convert::From<i32> for Base {
    fn from(s: i32) -> Self {
        Base::Int32(s)
    }
}
impl std::convert::From<i64> for Base {
    fn from(s: i64) -> Self {
        Base::Int64(s)
    }
}

//
//
// Container TO
//
//

impl std::convert::From<&Container> for signature::Container {
    fn from(c: &Container) -> crate::signature::Container {
        match c {
            Container::Array(arr) => signature::Container::Array(Box::new(arr.element_sig.clone())),
            Container::Dict(dict) => {
                signature::Container::Dict(dict.key_sig, Box::new(dict.value_sig.clone()))
            }
            Container::Struct(params) => {
                signature::Container::Struct(params.iter().map(|param| param.into()).collect())
            }
            Container::Variant(_) => signature::Container::Variant,
        }
    }
}

//
//
// Base TO
//
//

impl std::convert::From<&Base> for signature::Base {
    fn from(b: &Base) -> crate::signature::Base {
        match b {
            Base::Boolean(_) => signature::Base::Boolean,
            Base::Byte(_) => signature::Base::Byte,
            Base::Double(_) => signature::Base::Double,
            Base::Int16(_) => signature::Base::Int16,
            Base::Int32(_) => signature::Base::Int32,
            Base::Int64(_) => signature::Base::Int64,
            Base::Uint16(_) => signature::Base::Uint16,
            Base::Uint32(_) => signature::Base::Uint32,
            Base::Uint64(_) => signature::Base::Uint64,
            Base::ObjectPath(_) => signature::Base::ObjectPath,
            Base::Signature(_) => signature::Base::Signature,
            Base::String(_) => signature::Base::String,
            Base::UnixFd(_) => signature::Base::UnixFd,
        }
    }
}

impl std::convert::TryFrom<&Base> for bool {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<bool, Error> {
        if let Base::Boolean(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for String {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<String, Error> {
        if let Base::String(value) = b {
            Ok(value.clone())
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for u8 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<u8, Error> {
        if let Base::Byte(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for u16 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<u16, Error> {
        if let Base::Uint16(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for u32 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<u32, Error> {
        if let Base::Uint32(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for u64 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<u64, Error> {
        if let Base::Uint64(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for i16 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<i16, Error> {
        if let Base::Int16(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for i32 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<i32, Error> {
        if let Base::Int32(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}

impl std::convert::TryFrom<&Base> for i64 {
    type Error = Error;
    fn try_from(b: &Base) -> std::result::Result<i64, Error> {
        if let Base::Int64(value) = b {
            Ok(*value)
        } else {
            Err(Error::InvalidType)
        }
    }
}
