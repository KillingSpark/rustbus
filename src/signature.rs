#[derive(Copy, Clone)]
pub enum Base {
    Int32,
    Uint32,
    String,
    Signature,
    ObjectPath,
    Boolean,
}

#[derive(Clone)]
pub enum Container {
    Array(Box<Type>),
    Struct(Vec<Type>),
    Dict(Base, Box<Type>),
    Variant
}

#[derive(Clone)]
pub enum Type {
    Base(Base),
    Container(Container),
}

pub enum Error {
    InvalidSignature,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq, Eq)]
enum Token {
    Structstart,
    Structend,
    Array,
    Int32,
    Uint32,
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
            'i' => Token::Int32,
            'u' => Token::Uint32,
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

impl Type {
    pub fn from_str(sig: &str) -> Result<Type> {
        let mut tokens = make_tokens(sig)?;
        Self::parse_next_type(&mut tokens)
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
                Self::parse_next_type(tokens)
            }

            Token::Int32 => Ok(Type::Base(Base::Int32)),
            Token::Uint32 => Ok(Type::Base(Base::Uint32)),
            Token::String => Ok(Type::Base(Base::String)),
            Token::ObjectPath => Ok(Type::Base(Base::ObjectPath)),
            Token::Signature => Ok(Type::Base(Base::Signature)),
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
                Ok(Type::Container(Container::Dict(
                    key_type,
                    Box::new(value_type),
                )))
            }

            _ => Err(Error::InvalidSignature),
        }
    }

    fn parse_next_base(tokens: &mut Vec<Token>) -> Result<Base> {
        match tokens[0] {
            Token::Int32 => Ok(Base::Int32),
            Token::Uint32 => Ok(Base::Uint32),
            Token::String => Ok(Base::String),
            Token::ObjectPath => Ok(Base::ObjectPath),
            Token::Signature => Ok(Base::Signature),
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
