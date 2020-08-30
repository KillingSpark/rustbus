//! Everything that deals with converting from/to raw bytes. You probably do not need this.

pub mod marshal;
pub mod unmarshal;
pub mod util;
pub mod validate_raw;

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

#[derive(Eq, PartialEq, Debug)]
pub enum Variant2<'a, V1, V2> {
    V1(V1),
    V2(V2),
    Catchall(crate::params::Param<'a, 'a>),
}

impl<'a, V1, V2> marshal::traits::Signature for Variant2<'a, V1, V2> {
    fn signature() -> crate::signature::Type {
        crate::signature::Type::Container(crate::signature::Container::Variant)
    }
    fn alignment() -> usize {
        1
    }
}

impl<'a, V1, V2> marshal::traits::Marshal for Variant2<'a, V1, V2>
where
    V1: marshal::traits::Marshal + marshal::traits::Signature,
    V2: marshal::traits::Marshal + marshal::traits::Signature,
{
    fn marshal(&self, byteorder: crate::ByteOrder, buf: &mut Vec<u8>) -> Result<(), crate::Error> {
        match self {
            Self::V1(v1) => {
                let mut sig_str = String::new();
                V1::signature().to_str(&mut sig_str);
                crate::wire::marshal::base::marshal_base_param(
                    byteorder,
                    &crate::params::Base::Signature(sig_str),
                    buf,
                )
                .unwrap();
                v1.marshal(byteorder, buf)?;
            }
            Self::V2(v2) => {
                let mut sig_str = String::new();
                V2::signature().to_str(&mut sig_str);
                crate::wire::marshal::base::marshal_base_param(
                    byteorder,
                    &crate::params::Base::Signature(sig_str),
                    buf,
                )
                .unwrap();
                v2.marshal(byteorder, buf)?;
            }
            Self::Catchall(p) => {
                let mut sig_str = String::new();
                p.sig().to_str(&mut sig_str);
                crate::wire::marshal::base::marshal_base_param(
                    byteorder,
                    &crate::params::Base::Signature(sig_str),
                    buf,
                )
                .unwrap();
                marshal::container::marshal_param(p, byteorder, buf)?;
            }
        }
        Ok(())
    }
}

impl<'a, 'ret, 'buf: 'ret, V1, V2> unmarshal::traits::Unmarshal<'ret, 'buf> for Variant2<'a, V1, V2>
where
    V1: unmarshal::traits::Unmarshal<'ret, 'buf> + marshal::traits::Signature,
    V2: unmarshal::traits::Unmarshal<'ret, 'buf> + marshal::traits::Signature,
{
    fn unmarshal(
        byteorder: crate::ByteOrder,
        buf: &'buf [u8],
        offset: usize,
    ) -> unmarshal::UnmarshalResult<Self> {
        let (bytes, sig_str) = crate::wire::util::unmarshal_signature(&buf[offset..])?;
        let mut sig = crate::signature::Type::parse_description(&sig_str)?;
        let sig = if sig.len() == 1 {
            sig.remove(0)
        } else {
            return Err(crate::wire::unmarshal::Error::WrongSignature);
        };
        let offset = offset + bytes;
        if sig == V1::signature() {
            let (vbytes, v1) = V1::unmarshal(byteorder, buf, offset)?;
            Ok((bytes + vbytes, Self::V1(v1)))
        } else if sig == V2::signature() {
            let (vbytes, v2) = V2::unmarshal(byteorder, buf, offset)?;
            Ok((bytes + vbytes, Self::V2(v2)))
        } else {
            let (vbytes, p) = crate::wire::unmarshal::container::unmarshal_with_sig(
                byteorder, &sig, buf, offset,
            )?;
            Ok((bytes + vbytes, Self::Catchall(p)))
        }
    }
}

#[test]
fn variant_trait_impl() {
    use crate::Marshal;
    use crate::Unmarshal;

    type VT<'a> = Variant2<'a, String, i32>;
    let variant1 = VT::V1("ABCD".to_owned());
    let variant2 = VT::V2(1234);
    let variant3 = VT::V2(-2345);
    let variant4 = VT::V1("EFGHIJKL".to_owned());

    let mut buf = vec![];
    (&variant1, &variant2, &variant3, &variant4)
        .marshal(crate::ByteOrder::LittleEndian, &mut buf)
        .unwrap();

    // add a unknown variant here
    crate::message_builder::marshal_as_variant(0xFFFFu64, crate::ByteOrder::LittleEndian, &mut buf)
        .unwrap();

    let (bytes, (uv1, uv2, uv3, uv4)) =
        <(VT, VT, VT, VT) as Unmarshal>::unmarshal(crate::ByteOrder::LittleEndian, &buf, 0)
            .unwrap();
    assert_eq!(&variant1, &uv1);
    assert_eq!(&variant2, &uv2);
    assert_eq!(&variant3, &uv3);
    assert_eq!(&variant4, &uv4);

    let (_bytes, uv5) = VT::unmarshal(crate::ByteOrder::LittleEndian, &buf, bytes).unwrap();
    assert_eq!(
        uv5,
        VT::Catchall(crate::params::Param::Base(crate::params::Base::Uint64(
            0xFFFFu64
        )))
    );
}

#[macro_export(local_inner_macros)]
/// This macro provides a convenient way to create enums to represent relatively simple Variants, with fitting marshal/unmarshal implementations.
/// It can be used like this:
/// ```rust
///    type Map = std::collections::HashMap<String, (i32, u8, (u64, MyVariant))>;
///    type Struct = (u32, u32, MyVariant);
///    dbus_variant!(MyVariant, CaseMap => Map; CaseStruct => Struct);
/// ```
/// And it will generate an enum like this:
/// ```rust
/// enum MyVariant {
///     CaseMap(Map),
///     CaseStruct(Struct),
///     Catchall(rustbus::signature::Type),   
/// }
/// ```
/// The `Catchall` case is used for unmarshalling, when encountering a Value that did not match any of the other cases. **The generated marshal impl will
/// refuse to marshal the Catchall case!** If you want to have a case for a signature you need to make it explicitly.
///
/// ## Current limitations
/// 1. The type needs to be an identifier, so a single word. std::u64 does not work, but you can use local type-aliases to make this work
/// as shown in the example above.
/// 1. References like &str are not supported
macro_rules! dbus_variant {
    ($vname: ident, $($name: ident => $typ: ident);+) => {
        dbus_variant_type!($vname, $(
            $name => $typ
        )+);

        impl marshal::traits::Signature for $vname {
            fn signature() -> rustbus::signature::Type {
                rustbus::signature::Type::Container(rustbus::signature::Container::Variant)
            }
            fn alignment() -> usize {
                1
            }
        }

        dbus_variant_marshal!($vname, $(
            $name => $typ
        )+);
        dbus_variant_unmarshal!($vname, $(
            $name => $typ
        )+);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dbus_variant_type {
    ($vname: ident, $($name: ident => $typ: ident)+) => {
        #[derive(Eq, PartialEq, Debug)]
        pub enum $vname {
            $(
                $name($typ),
            )+
            Catchall(rustbus::signature::Type)
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dbus_variant_marshal {
    ($vname: ident, $($name: ident => $typ: ident)+) => {
        impl rustbus::Marshal for $vname {
            fn marshal(&self, byteorder: rustbus::ByteOrder, buf: &mut Vec<u8>) -> Result<(), rustbus::Error> {
                use rustbus::Signature;

                match self {
                    $(
                        Self::$name(v) => {
                            let mut sig_str = String::new();
                            $typ::signature().to_str(&mut sig_str);
                            rustbus::wire::marshal::base::marshal_base_param(
                                byteorder,
                                &rustbus::params::Base::Signature(sig_str),
                                buf,
                            )
                            .unwrap();
                            v.marshal(byteorder, buf)?;
                        }
                    )+
                    Self::Catchall(_) => unimplemented!("Do not use Catchall for Marshal cases!"),
                }
                Ok(())
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dbus_variant_unmarshal {
    ($vname: ident, $($name: ident => $typ: ident)+) => {
        impl<'ret, 'buf: 'ret> unmarshal::traits::Unmarshal<'ret, 'buf> for $vname {
            fn unmarshal(
                byteorder: rustbus::ByteOrder,
                buf: &'buf [u8],
                offset: usize,
            ) -> unmarshal::UnmarshalResult<Self> {
                use rustbus::Signature;

                let (bytes, sig_str) = rustbus::wire::util::unmarshal_signature(&buf[offset..])?;
                let mut sig = rustbus::signature::Type::parse_description(&sig_str)?;
                let sig = if sig.len() == 1 {
                    sig.remove(0)
                } else {
                    return Err(rustbus::wire::unmarshal::Error::WrongSignature);
                };
                let offset = offset + bytes;

                $(
                if sig == $typ::signature() {
                    let (vbytes, v) = $typ::unmarshal(byteorder, buf, offset)?;
                    return Ok((bytes + vbytes, Self::$name(v)));
                }
                )+
                let vbytes = rustbus::wire::validate_raw::validate_marshalled(
                    byteorder, offset, buf, &sig
                ).map_err(|e| e.1)?;
                Ok((bytes + vbytes, Self::Catchall(sig)))
            }
        }
    };
}

#[test]
fn test_variant_macro() {
    use crate::Marshal;
    use crate::Unmarshal;

    // so the macro is able to use rustbus, like it would have to when importet into other crates
    use crate as rustbus;

    let mut buf = vec![];
    dbus_variant!(MyVariant, String => String; V2 => i32; Integer => u32);
    let v1 = MyVariant::String("ABCD".to_owned());
    let v2 = MyVariant::V2(0);
    let v3 = MyVariant::Integer(100);

    (&v1, &v2, &v3)
        .marshal(crate::ByteOrder::LittleEndian, &mut buf)
        .unwrap();
    // add a unknown variant here
    crate::message_builder::marshal_as_variant(0xFFFFu64, crate::ByteOrder::LittleEndian, &mut buf)
        .unwrap();

    let (bytes, (uv1, uv2, uv3)) = <(MyVariant, MyVariant, MyVariant) as Unmarshal>::unmarshal(
        crate::ByteOrder::LittleEndian,
        &buf,
        0,
    )
    .unwrap();
    assert_eq!(uv1, v1);
    assert_ne!(uv1, v2);
    assert_ne!(uv1, v3);

    assert_eq!(uv2, v2);
    assert_ne!(uv2, v3);

    assert_eq!(uv3, v3);

    let (_bytes, uv4) = MyVariant::unmarshal(crate::ByteOrder::LittleEndian, &buf, bytes).unwrap();
    assert_eq!(
        uv4,
        MyVariant::Catchall(crate::signature::Type::Base(crate::signature::Base::Uint64))
    );

    type Map = std::collections::HashMap<String, (i32, u8, (u64, MyVariant))>;
    type Struct = (u32, u32, MyVariant);
    dbus_variant!(MyVariant2, CaseMap => Map; CaseStruct => Struct);

    let mut map = Map::new();
    map.insert(
        "AAAA".into(),
        (100, 20, (300, MyVariant::String("BBBB".into()))),
    );
    map.insert("CCCC".into(), (400, 50, (600, MyVariant::V2(0))));
    map.insert("DDDD".into(), (500, 60, (700, MyVariant::Integer(10))));
    let v1 = MyVariant2::CaseMap(map);
    let v2 = MyVariant2::CaseStruct((10, 20, MyVariant::String("AAAAA".into())));
    let v3 = MyVariant2::CaseStruct((30, 40, MyVariant::V2(10)));
    let v4 = MyVariant2::CaseStruct((30, 40, MyVariant::Integer(20)));

    let mut buf = vec![];
    (&v1, &v2, &v3, &v4)
        .marshal(crate::ByteOrder::LittleEndian, &mut buf)
        .unwrap();
    let (_bytes, (uv1, uv2, uv3, uv4)) =
        <(MyVariant2, MyVariant2, MyVariant2, MyVariant2) as Unmarshal>::unmarshal(
            crate::ByteOrder::LittleEndian,
            &buf,
            0,
        )
        .unwrap();
    assert_eq!(uv1, v1);
    assert_ne!(uv1, v2);
    assert_ne!(uv1, v3);
    assert_ne!(uv1, v4);

    assert_eq!(uv2, v2);
    assert_ne!(uv2, v3);
    assert_ne!(uv2, v4);

    assert_eq!(uv3, v3);
    assert_ne!(uv3, v4);

    assert_eq!(uv4, v4);
}
