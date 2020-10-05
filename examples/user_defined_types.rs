use rustbus::signature;
use rustbus::wire::marshal::traits::MarshalContext;
use rustbus::wire::util;
use rustbus::Marshal;
use rustbus::Signature;

// A Type with some trivial member and some enum
// The enum will be packed into a Variant. The decoding can look at the signature of the Variant and figure out
// which kind it was. To be even more explicit, we will put a string in the message that tells us which kind it was.
struct MyType {
    x: u64,
    sub: Sub,
}

enum Sub {
    Main(MySubType),
    Other(MyOtherSubType),
}

rustbus::dbus_variant_sig!(MyVar, Int32 => i32; Int64 => i64);

use rustbus::message_builder::marshal_as_variant;
impl Signature for &MyType {
    fn signature() -> signature::Type {
        // in dbus signature coding: (t(sv))
        // Note how the type of the `sub` is represented as `v`
        // variants include the signature of their content in marshalled form
        signature::Type::Container(signature::Container::Struct(vec![
            u64::signature(),
            signature::Type::Container(signature::Container::Struct(vec![
                signature::Type::Base(signature::Base::String),
                signature::Type::Container(signature::Container::Variant),
            ])),
        ]))
    }

    fn alignment() -> usize {
        8
    }
}

impl Marshal for &MyType {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), rustbus::Error> {
        // always align structs to 8!
        util::pad_to_align(8, ctx.buf);

        // boring
        self.x.marshal(ctx)?;

        // match on which kind this contains
        match &self.sub {
            Sub::Main(t) => {
                // marshal the type-name and the MySubType as a Variant
                "Main".marshal(ctx)?;
                marshal_as_variant(t, ctx.byteorder, ctx.buf, ctx.fds)?;
            }
            Sub::Other(t) => {
                // marshal the type-name and the MyOtherSubType as a Variant
                "Other".marshal(ctx)?;
                marshal_as_variant(t, ctx.byteorder, ctx.buf, ctx.fds)?
            }
        };
        Ok(())
    }
}

// The impl for these types are trivial. They should be derivable in the future.
struct MySubType {
    x: i32,
    y: i32,
}
struct MyOtherSubType {
    x: u32,
    y: u32,
}

impl Signature for &MySubType {
    fn signature() -> signature::Type {
        signature::Type::Container(signature::Container::Struct(vec![
            i32::signature(),
            i32::signature(),
        ]))
    }

    fn alignment() -> usize {
        8
    }
}
impl Marshal for &MySubType {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), rustbus::Error> {
        // always align to 8
        util::pad_to_align(8, ctx.buf);
        self.x.marshal(ctx)?;
        self.y.marshal(ctx)?;
        Ok(())
    }
}

impl Signature for &MyOtherSubType {
    fn signature() -> signature::Type {
        signature::Type::Container(signature::Container::Struct(vec![
            u32::signature(),
            u32::signature(),
        ]))
    }

    fn alignment() -> usize {
        8
    }
}
impl Marshal for &MyOtherSubType {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), rustbus::Error> {
        // always align to 8
        util::pad_to_align(8, ctx.buf);
        self.x.marshal(ctx)?;
        self.y.marshal(ctx)?;
        Ok(())
    }
}

use rustbus::{
    client_conn::Timeout, get_session_bus_path, standard_messages, Conn, MessageBuilder,
};

// Just to have a main here we will send a message containing two MyType structs
fn main() -> Result<(), rustbus::client_conn::Error> {
    let session_path = get_session_bus_path()?;
    let mut con = Conn::connect_to_bus(session_path, true)?;
    con.send_message(&mut standard_messages::hello(), Timeout::Infinite)?;

    let mut sig = MessageBuilder::new()
        .signal(
            "io.killing.spark".into(),
            "TestSignal".into(),
            "/io/killing/spark".into(),
        )
        .build();

    let t = MyType {
        x: 123456,
        sub: Sub::Main(MySubType {
            x: 42387i32,
            y: 34875i32,
        }),
    };
    let t2 = MyType {
        x: 123456,
        sub: Sub::Other(MyOtherSubType {
            x: 42387u32,
            y: 34875u32,
        }),
    };

    sig.body.push_param(&t)?;
    sig.body.push_param(&t2)?;
    sig.body.push_param(MyVar::Int32(100))?;
    sig.body.push_param(MyVar::Int64(-100))?;

    con.send_message(&mut sig, Timeout::Infinite)?;

    Ok(())
}
