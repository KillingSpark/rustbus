#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Container {
    Array(Box<Type>),
    Struct(Vec<Type>),
    Dict(Base, Box<Type>),
    Variant,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Base(Base),
    Container(Container),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InvalidSignature,
    SignatureTooLong,
    NestingTooDeep,
    EmptySignature,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq, Eq, Debug)]
enum Token {
    Structstart,
    Structend,
    Array,
    Boolean,
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
    Variant,
}

fn char_to_token(c: char) -> Result<Token> {
    let t = match c {
        '(' => Token::Structstart,
        ')' => Token::Structend,
        'a' => Token::Array,
        'b' => Token::Boolean,
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
        'v' => Token::Variant,
        _ => return Err(Error::InvalidSignature),
    };
    Ok(t)
}

struct TokenIter<I: Iterator<Item = char>> {
    chars: I,
}

impl<I: Iterator<Item = char>> Iterator for TokenIter<I> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.chars.next() {
            Some(char_to_token(c))
        } else {
            None
        }
    }
}

fn make_tokens<I: Iterator<Item = char>>(sig: I) -> TokenIter<I> {
    TokenIter { chars: sig }
}

impl Container {
    pub fn to_str(&self, buf: &mut String) {
        match self {
            Container::Array(el) => {
                buf.push('a');
                el.to_str(buf);
            }
            Container::Dict(key, val) => {
                buf.push('a');
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

    pub fn get_alignment(&self) -> usize {
        match self {
            Container::Variant => 1,
            Container::Array(_) => 4,
            Container::Dict(_, _) => 4,
            Container::Struct(_) => 8,
        }
    }
}

impl Base {
    pub fn to_str(self, buf: &mut String) {
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

    pub fn get_alignment(self) -> usize {
        match self {
            Base::Boolean => 4,
            Base::Byte => 1,
            Base::Int16 => 2,
            Base::Uint16 => 2,
            Base::Int32 => 4,
            Base::Uint32 => 4,
            Base::UnixFd => 4,
            Base::Int64 => 8,
            Base::Uint64 => 8,
            Base::Double => 8,
            Base::String => 4,
            Base::ObjectPath => 4,
            Base::Signature => 1,
        }
    }
}

impl Type {
    pub fn parse_description(sig: &str) -> Result<Vec<Type>> {
        if sig.len() > 255 {
            return Err(Error::SignatureTooLong);
        }
        if sig.is_empty() {
            return Err(Error::EmptySignature);
        }

        let mut tokens = make_tokens(sig.chars());
        let mut types = Vec::new();
        while let Some(t) = Self::parse_next_type(&mut tokens, None)? {
            types.push(t);
        }
        for t in &types {
            Self::check_nesting_depth(t, 0, 0)?;
        }
        Ok(types)
    }

    fn check_nesting_depth(t: &Type, struct_depth: u8, array_depth: u8) -> Result<()> {
        if struct_depth >= 32 || array_depth >= 32 {
            Err(Error::NestingTooDeep)
        } else {
            match t {
                Type::Base(_) => Ok(()),
                Type::Container(Container::Struct(types)) => {
                    for t in types {
                        Self::check_nesting_depth(t, struct_depth + 1, array_depth)?;
                    }
                    Ok(())
                }
                Type::Container(Container::Array(elem_t)) => {
                    Self::check_nesting_depth(elem_t, struct_depth, array_depth + 1)
                }
                Type::Container(Container::Dict(_, elem_t)) => {
                    Self::check_nesting_depth(elem_t, struct_depth, array_depth + 1)
                }
                Type::Container(Container::Variant) => Ok(()),
            }
        }
    }

    pub fn to_str(&self, buf: &mut String) {
        match self {
            Type::Container(c) => c.to_str(buf),
            Type::Base(b) => b.to_str(buf),
        }
    }

    pub fn get_alignment(&self) -> usize {
        match self {
            Type::Base(b) => b.get_alignment(),
            Type::Container(c) => c.get_alignment(),
        }
    }

    fn parse_next_type<I: Iterator<Item = Result<Token>>>(
        tokens: &mut I,
        delim: Option<Token>,
    ) -> Result<Option<Type>> {
        if let Some(token) = tokens.next() {
            let token = token?;
            match token {
                Token::Structstart => {
                    let types = Self::parse_struct(tokens)?;
                    Ok(Some(Type::Container(Container::Struct(types))))
                }
                Token::Structend => {
                    if Some(token) == delim {
                        Ok(None)
                    } else {
                        Err(Error::InvalidSignature)
                    }
                }
                Token::Array => {
                    let elem_type = Self::parse_next_type(tokens, None)?;
                    match elem_type {
                        Some(Type::Container(Container::Dict(_, _))) => Ok(elem_type),
                        Some(elem_type) => {
                            Ok(Some(Type::Container(Container::Array(Box::new(elem_type)))))
                        }
                        None => Err(Error::InvalidSignature),
                    }
                }
                Token::DictEntryStart => {
                    let key_type = Self::parse_next_base(tokens)?;
                    if let Some(value_type) = Self::parse_next_type(tokens, None)? {
                        if tokens.next() != Some(Ok(Token::DictEntryEnd)) {
                            return Err(Error::InvalidSignature);
                        }
                        Ok(Some(Type::Container(Container::Dict(
                            key_type,
                            Box::new(value_type),
                        ))))
                    } else {
                        Err(Error::InvalidSignature)
                    }
                }

                Token::Byte => Ok(Some(Type::Base(Base::Byte))),
                Token::Boolean => Ok(Some(Type::Base(Base::Boolean))),
                Token::Int16 => Ok(Some(Type::Base(Base::Int16))),
                Token::Uint16 => Ok(Some(Type::Base(Base::Uint16))),
                Token::Int32 => Ok(Some(Type::Base(Base::Int32))),
                Token::Uint32 => Ok(Some(Type::Base(Base::Uint32))),
                Token::Int64 => Ok(Some(Type::Base(Base::Int64))),
                Token::Uint64 => Ok(Some(Type::Base(Base::Uint64))),
                Token::Double => Ok(Some(Type::Base(Base::Double))),
                Token::String => Ok(Some(Type::Base(Base::String))),
                Token::ObjectPath => Ok(Some(Type::Base(Base::ObjectPath))),
                Token::Signature => Ok(Some(Type::Base(Base::Signature))),
                Token::Variant => Ok(Some(Type::Container(Container::Variant))),
                _ => Err(Error::InvalidSignature),
            }
        } else if delim.is_none() {
            // we are just parsing types and are not within a struct
            Ok(None)
        } else {
            // we are in a struct and need to stop at a delimiter
            Err(Error::InvalidSignature)
        }
    }

    fn parse_next_base<I: Iterator<Item = Result<Token>>>(tokens: &mut I) -> Result<Base> {
        if let Some(token) = tokens.next() {
            let token = token?;
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
        } else {
            Err(Error::InvalidSignature)
        }
    }

    fn parse_struct<I: Iterator<Item = Result<Token>>>(tokens: &mut I) -> Result<Vec<Type>> {
        let mut types = Vec::new();
        while let Some(t) = Self::parse_next_type(tokens, Some(Token::Structend))? {
            types.push(t);
        }
        Ok(types)
    }
}
