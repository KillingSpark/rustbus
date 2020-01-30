#[derive(Copy, Clone, Debug)]
pub enum Base {
    Byte,
    Int16,
    Uint16,
    Int32,
    Uint32,
    UnixFd,
    Int64,
    Uint64,
    Double,
    String,
    Signature,
    ObjectPath,
    Boolean,
}

#[derive(Clone, Debug)]
pub enum Container {
    Array(Box<Type>),
    Struct(Vec<Type>),
    Dict(Base, Box<Type>),
    Variant,
}

#[derive(Clone, Debug)]
pub enum Type {
    Base(Base),
    Container(Container),
}

pub enum Error {
    InvalidSignature,
    EmptySignature,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq, Eq, Debug)]
enum Token {
    Structstart,
    Structend,
    Array,
    Byte,
    Int16,
    Uint16,
    Int32,
    Uint32,
    UnixFd,
    Int64,
    Uint64,
    Double,
    String,
    ObjectPath,
    Signature,
    DictEntryStart,
    DictEntryEnd,
}

fn make_tokens(sig: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();

    for c in sig.chars().filter(|c| *c != ' ' && *c != '\t') {
        let token = match c {
            '(' => Token::Structstart,
            ')' => Token::Structend,
            'a' => Token::Array,
            'y' => Token::Byte,
            'n' => Token::Int16,
            'q' => Token::Uint16,
            'i' => Token::Int32,
            'u' => Token::Uint32,
            'h' => Token::UnixFd,
            'x' => Token::Int64,
            't' => Token::Uint64,
            'd' => Token::Double,
            's' => Token::String,
            'o' => Token::ObjectPath,
            'g' => Token::Signature,
            '{' => Token::DictEntryStart,
            '}' => Token::DictEntryEnd,
            _ => return Err(Error::InvalidSignature),
        };
        tokens.push(token);
    }

    Ok(tokens)
}

impl Container {
    pub fn to_str(&self, buf: &mut String) {
        match self {
            Container::Array(el) => {
                buf.push('a');
                el.to_str(buf);
            }
            Container::Dict(key, val) => {
                buf.push('{');
                key.to_str(buf);
                val.to_str(buf);
                buf.push('}');
            }
            Container::Struct(types) => {
                buf.push('(');
                for t in types {
                    t.to_str(buf);
                }
                buf.push(')');
            }
            Container::Variant => {
                buf.push('v');
            }
        }
    }
}

impl Base {
    pub fn to_str(&self, buf: &mut String) {
        match self {
            Base::Boolean => buf.push('b'),
            Base::Byte => buf.push('y'),
            Base::Int16 => buf.push('n'),
            Base::Uint16 => buf.push('q'),
            Base::Int32 => buf.push('i'),
            Base::Uint32 => buf.push('u'),
            Base::UnixFd => buf.push('h'),
            Base::Int64 => buf.push('x'),
            Base::Uint64 => buf.push('t'),
            Base::Double => buf.push('d'),
            Base::String => buf.push('s'),
            Base::ObjectPath => buf.push('o'),
            Base::Signature => buf.push('g'),
        }
    }
}

impl Type {
    pub fn from_str(sig: &str) -> Result<Vec<Type>> {
        let mut tokens = make_tokens(sig)?;
        if tokens.is_empty() {
            return Err(Error::EmptySignature);
        }
        let mut types = Vec::new();
        while !tokens.is_empty() {
            let t = Self::parse_next_type(&mut tokens)?;
            types.push(t);
        }
        Ok(types)
    }

    pub fn to_str(&self, buf: &mut String) {
        match self {
            Type::Container(c) => c.to_str(buf),
            Type::Base(b) => b.to_str(buf),
        }
    }

    fn parse_next_type(tokens: &mut Vec<Token>) -> Result<Type> {
        match tokens[0] {
            Token::Structstart => {
                tokens.remove(0);
                let types = Self::parse_struct(tokens)?;
                if tokens.is_empty() {
                    return Err(Error::InvalidSignature);
                }
                tokens.remove(0);
                Ok(Type::Container(Container::Struct(types)))
            }
            Token::Array => {
                tokens.remove(0);
                let elem_type = Self::parse_next_type(tokens)?;
                if let Type::Container(Container::Dict(_, _)) = &elem_type {
                    // if the array contains dictentries this is a dict
                    Ok(elem_type)
                } else {
                    Ok(Type::Container(Container::Array(Box::new(elem_type))))
                }
            }

            Token::Byte => {
                tokens.remove(0);
                Ok(Type::Base(Base::Byte))
            }
            Token::Int16 => {
                tokens.remove(0);
                Ok(Type::Base(Base::Int16))
            }
            Token::Uint16 => {
                tokens.remove(0);
                Ok(Type::Base(Base::Uint16))
            }
            Token::Int32 => {
                tokens.remove(0);
                Ok(Type::Base(Base::Int32))
            }
            Token::Uint32 => {
                tokens.remove(0);
                Ok(Type::Base(Base::Uint32))
            }
            Token::Int64 => {
                tokens.remove(0);
                Ok(Type::Base(Base::Int64))
            }
            Token::Uint64 => {
                tokens.remove(0);
                Ok(Type::Base(Base::Uint64))
            }
            Token::Double => {
                tokens.remove(0);
                Ok(Type::Base(Base::Double))
            }
            Token::String => {
                tokens.remove(0);
                Ok(Type::Base(Base::String))
            }
            Token::ObjectPath => {
                tokens.remove(0);
                Ok(Type::Base(Base::ObjectPath))
            }
            Token::Signature => {
                tokens.remove(0);
                Ok(Type::Base(Base::Signature))
            }
            Token::DictEntryStart => {
                tokens.remove(0);
                let key_type = Self::parse_next_base(tokens)?;
                let value_type = Self::parse_next_type(tokens)?;
                if tokens.is_empty() {
                    return Err(Error::InvalidSignature);
                }
                if tokens[0] != Token::DictEntryEnd {
                    return Err(Error::InvalidSignature);
                }
                tokens.remove(0);
                Ok(Type::Container(Container::Dict(
                    key_type,
                    Box::new(value_type),
                )))
            }

            _ => Err(Error::InvalidSignature),
        }
    }

    fn parse_next_base(tokens: &mut Vec<Token>) -> Result<Base> {
        let token = tokens.remove(0);
        match token {
            Token::Byte => Ok(Base::Byte),
            Token::Int16 => Ok(Base::Int16),
            Token::Uint16 => Ok(Base::Uint16),
            Token::Int32 => Ok(Base::Int32),
            Token::Uint32 => Ok(Base::Uint32),
            Token::Int64 => Ok(Base::Int64),
            Token::Uint64 => Ok(Base::Uint64),
            Token::String => Ok(Base::String),
            Token::ObjectPath => Ok(Base::ObjectPath),
            Token::Signature => Ok(Base::Signature),
            Token::Double => Ok(Base::Double),
            Token::UnixFd => Ok(Base::UnixFd),
            _ => Err(Error::InvalidSignature),
        }
    }

    fn parse_struct(tokens: &mut Vec<Token>) -> Result<Vec<Type>> {
        let mut types = Vec::new();
        while !tokens.is_empty() {
            if tokens[0] == Token::Structend {
                break;
            }
            types.push(Self::parse_next_type(tokens)?);
        }
        Ok(types)
    }
}
