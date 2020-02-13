use rustbus::{
    get_session_bus_path,
    message::{Base, Container, Error, Message, Param},
    standard_messages, Conn, MessageBuilder, RpcConn,
};
use std::convert::TryFrom;

/**
from introspection:
...
<method name="resuest1">
   <arg type="u" direction="in" name="request1"/>
   <arg type="(usb)" direction="out" name="response1"/>
</method>

<method name="request2">
   <arg type="(uus)" direction="in" name="request2"/>
   <arg type="(bss)" direction="out" name="response2"/>
</method>
...
 **/

#[derive(Debug)]
struct Response1 {
    a: u32,
    b: String,
    c: bool,
}

#[derive(Debug)]
struct Request2 {
    a: u32,
    b: u32,
    c: String,
}

#[derive(Debug)]
struct Response2 {
    a: bool,
    b: String,
    c: String,
}

// helper for conversion from our struct to rustbus::message::Param
impl std::convert::From<Request2> for Param {
    fn from(r: Request2) -> Self {
        Container::Struct(vec![r.a.into(), r.b.into(), r.c.into()]).into()
    }
}

// wrapper for make conversion from rustbus::message::Param to string
// bool or ordinal types
fn convert<'a, T: TryFrom<&'a Base>>(p: &'a Param) -> Result<T, Error> {
    if let Param::Base(base) = p {
        return T::try_from(base).map_err(|_| Error::InvalidType);
    }
    Err(Error::InvalidType)
}

impl TryFrom<Vec<Param>> for Response1 {
    type Error = Error;
    fn try_from(p: Vec<Param>) -> Result<Response1, Self::Error> {
        if p.len() == 1 {
            if let Param::Container(c) = &p[0] {
                if let Container::Struct(params) = c {
                    if params.len() == 3 {
                        let a = convert::<u32>(&params[0])?;
                        let b = convert::<String>(&params[1])?;
                        let c = convert::<bool>(&params[2])?;
                        return Ok(Response1 { a, b, c });
                    }
                }
            }
        }
        Err(Error::InvalidType)
    }
}

impl TryFrom<Vec<Param>> for Response2 {
    type Error = Error;
    fn try_from(p: Vec<Param>) -> Result<Response2, Self::Error> {
        if p.len() == 1 {
            if let Param::Container(c) = &p[0] {
                if let Container::Struct(params) = c {
                    if params.len() == 3 {
                        let a = convert::<bool>(&params[0])?;
                        let b = convert::<String>(&params[1])?;
                        let c = convert::<String>(&params[2])?;
                        return Ok(Response2 { a, b, c });
                    }
                }
            }
        }
        Err(Error::InvalidType)
    }
}

fn build_message1(value: u32) -> Message {
    MessageBuilder::new()
        .call("request1".into())
        .on("/io/killing/spark".into())
        .with_interface("io.killing.spark".into())
        .at("io.killing.spark".into())
        .with_params(vec![value.into()])
        .build()
}

fn build_message2(value: Request2) -> Message {
    MessageBuilder::new()
        .call("request1".into())
        .on("/io/killing/spark".into())
        .with_interface("io.killing.spark".into())
        .at("io.killing.spark".into())
        .with_params(vec![value.into()])
        .build()
}

fn send_and_recv<T: TryFrom<Vec<Param>> + std::fmt::Debug>(
    conn: &mut RpcConn,
    msg: Message,
) -> Result<(), rustbus::client_conn::Error> {
    let serial = conn.send_message(msg)?.serial.unwrap();
    let response = conn.wait_response(serial)?;
    let response_converted = T::try_from(response.params).map_err(|_| Error::InvalidType)?;
    println!("Got response {:?}", response_converted);
    Ok(())
}

fn main() -> Result<(), rustbus::client_conn::Error> {
    let session_path = get_session_bus_path()?;
    let con = Conn::connect_to_bus(session_path, true)?;
    let mut rpc_con = RpcConn::new(con);
    rpc_con.send_message(standard_messages::hello())?;
    send_and_recv::<Response1>(&mut rpc_con, build_message1(42))?;
    send_and_recv::<Response2>(
        &mut rpc_con,
        build_message2(Request2 {
            a: 42,
            b: 24,
            c: "test".into(),
        }),
    )?;
    Ok(())
}
