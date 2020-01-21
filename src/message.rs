pub enum Message {
    Signal,
    Call(Call),
    Reply,
}

pub enum Base {
    Int32(i32),
    String(String),
    Signature(String),
    ObjectPath(String),
    Boolean(bool),
}

pub enum Container {
    Array(Vec<Param>),
    Struct(Vec<Param>),
    DictEntry(Base, Box<Param>)
}

pub enum Param {
    Base(Base),
    Container(Container),
}

pub struct Call {
    pub interface: String,
    pub member: String,
    pub params: Vec<Param>
}