use rustbus::signature;
use rustbus::wire::marshal::MarshalContext;
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
rustbus::dbus_variant_var!(CharVar, U16 => u16; String => String);

use rustbus::message_builder::marshal_as_variant;
impl Signature for &MyType {
    fn signature() -> signature::Type {
        // in dbus signature coding: (t(sv))
        // Note how the type of the `sub` is represented as `v`
        // variants include the signature of their content in marshalled form
        signature::Type::Container(signature::Container::Struct(
            signature::StructTypes::new(vec![
                u64::signature(),
                signature::Type::Container(signature::Container::Struct(
                    signature::StructTypes::new(vec![
                        signature::Type::Base(signature::Base::String),
                        signature::Type::Container(signature::Container::Variant),
                    ])
                    .unwrap(),
                )),
            ])
            .unwrap(),
        ))
    }

    fn alignment() -> usize {
        8
    }
}

impl Marshal for &MyType {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), rustbus::Error> {
        // always align structs to 8!
        ctx.align_to(8);

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
        signature::Type::Container(signature::Container::Struct(
            signature::StructTypes::new(vec![i32::signature(), i32::signature()]).unwrap(),
        ))
    }

    fn alignment() -> usize {
        8
    }
}
impl Marshal for &MySubType {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), rustbus::Error> {
        // always align to 8
        ctx.align_to(8);
        self.x.marshal(ctx)?;
        self.y.marshal(ctx)?;
        Ok(())
    }
}

impl Signature for &MyOtherSubType {
    fn signature() -> signature::Type {
        signature::Type::Container(signature::Container::Struct(
            signature::StructTypes::new(vec![u32::signature(), u32::signature()]).unwrap(),
        ))
    }

    fn alignment() -> usize {
        8
    }
}
impl Marshal for &MyOtherSubType {
    fn marshal(&self, ctx: &mut MarshalContext) -> Result<(), rustbus::Error> {
        // always align to 8
        ctx.align_to(8);
        self.x.marshal(ctx)?;
        self.y.marshal(ctx)?;
        Ok(())
    }
}

use rustbus::{connection::Timeout, get_session_bus_path, DuplexConn, MessageBuilder};

// Just to have a main here we will send a message containing two MyType structs
fn main() -> Result<(), rustbus::connection::Error> {
    let session_path = get_session_bus_path()?;
    let mut con = DuplexConn::connect_to_bus(session_path, true)?;
    con.send_hello(Timeout::Infinite)?;

    let mut sig = MessageBuilder::new()
        .signal(
            "io.killing.spark",
            "TestSignal",
            "/io/killing/spark",
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

    con.send.send_message(&mut sig)?.write_all().unwrap();

    Ok(())
}
