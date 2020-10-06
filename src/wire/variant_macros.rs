#[macro_export(local_inner_macros)]
/// This macro provides a convenient way to create enums to represent relatively simple Variants, with fitting marshal/unmarshal implementations.
/// It can be used like this:
/// ```rust, ignore
///    type Map = std::collections::HashMap<String, (i32, u8, (u64, MyVariant))>;
///    type Struct = (u32, u32, MyVariant);
///    dbus_variant_sig!(MyVariant, CaseMap => Map; CaseStruct => Struct);
/// ```
/// And it will generate an enum like this:
/// ```rust, ignore
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
/// 1. References like &str are not supported
macro_rules! dbus_variant_sig {
    ($vname: ident, $($name: ident => $typ: path);+) => {
        dbus_variant_sig_type!($vname, $(
            $name => $typ
        )+);

        impl rustbus::Signature for $vname {
            fn signature() -> rustbus::signature::Type {
                rustbus::signature::Type::Container(rustbus::signature::Container::Variant)
            }
            fn alignment() -> usize {
                1
            }
        }

        dbus_variant_sig_marshal!($vname, $(
            $name => $typ
        )+);
        dbus_variant_sig_unmarshal!($vname, $(
            $name => $typ
        )+);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dbus_variant_sig_type {
    ($vname: ident, $($name: ident => $typ: path)+) => {
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
macro_rules! dbus_variant_sig_marshal {
    ($vname: ident, $($name: ident => $typ: path)+) => {
        impl rustbus::Marshal for $vname {
            fn marshal(&self, ctx: &mut rustbus::wire::marshal::MarshalContext) -> Result<(), rustbus::Error> {
                use rustbus::Signature;

                match self {
                    $(
                        Self::$name(v) => {
                            let mut sig_str = String::new();
                            <$typ as Signature>::signature().to_str(&mut sig_str);
                            rustbus::wire::marshal::base::marshal_base_param(
                                &rustbus::params::Base::Signature(sig_str),
                                ctx,
                            )
                            .unwrap();
                            v.marshal(ctx)?;
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
macro_rules! dbus_variant_sig_unmarshal {
    ($vname: ident, $($name: ident => $typ: path)+) => {
        impl<'ret, 'buf: 'ret> rustbus::Unmarshal<'ret, 'buf> for $vname {
            fn unmarshal(
                byteorder: rustbus::ByteOrder,
                buf: &'buf [u8],
                offset: usize,
            ) -> rustbus::wire::unmarshal::UnmarshalResult<Self> {
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
                if sig == <$typ as Signature>::signature() {
                    let (vbytes, v) = <$typ as rustbus::Unmarshal>::unmarshal(byteorder, buf, offset)?;
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
fn test_variant_sig_macro() {
    use crate::Marshal;
    use crate::Unmarshal;

    use crate::wire::marshal::MarshalContext;

    let mut fds = Vec::new();
    let mut ctxbuf = Vec::new();
    let mut ctx = MarshalContext {
        buf: &mut ctxbuf,
        fds: &mut fds,
        byteorder: crate::ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;

    // so the macro is able to use rustbus, like it would have to when importet into other crates
    use crate as rustbus;

    dbus_variant_sig!(MyVariant, String => std::string::String; V2 => i32; Integer => u32);
    let v1 = MyVariant::String("ABCD".to_owned());
    let v2 = MyVariant::V2(0);
    let v3 = MyVariant::Integer(100);

    (&v1, &v2, &v3).marshal(ctx).unwrap();
    // add a unknown variant here
    crate::message_builder::marshal_as_variant(
        0xFFFFu64,
        crate::ByteOrder::LittleEndian,
        &mut ctx.buf,
        &mut ctx.fds,
    )
    .unwrap();

    let (bytes, (uv1, uv2, uv3)) = <(MyVariant, MyVariant, MyVariant) as Unmarshal>::unmarshal(
        crate::ByteOrder::LittleEndian,
        &ctx.buf,
        0,
    )
    .unwrap();
    assert_eq!(uv1, v1);
    assert_ne!(uv1, v2);
    assert_ne!(uv1, v3);

    assert_eq!(uv2, v2);
    assert_ne!(uv2, v3);

    assert_eq!(uv3, v3);

    let (_bytes, uv4) =
        MyVariant::unmarshal(crate::ByteOrder::LittleEndian, &ctx.buf, bytes).unwrap();
    assert_eq!(
        uv4,
        MyVariant::Catchall(crate::signature::Type::Base(crate::signature::Base::Uint64))
    );

    type Map = std::collections::HashMap<String, (i32, u8, (u64, MyVariant))>;
    type Struct = (u32, u32, MyVariant);
    dbus_variant_sig!(MyVariant2, CaseMap => Map; CaseStruct => Struct);

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

    ctx.buf.clear();
    (&v1, &v2, &v3, &v4).marshal(ctx).unwrap();
    let (_bytes, (uv1, uv2, uv3, uv4)) =
        <(MyVariant2, MyVariant2, MyVariant2, MyVariant2) as Unmarshal>::unmarshal(
            crate::ByteOrder::LittleEndian,
            &ctx.buf,
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

    // Test that catchall gets the right signatures
    ctx.buf.clear();
    crate::message_builder::marshal_as_variant(
        ("", "", 100u8),
        crate::ByteOrder::LittleEndian,
        &mut ctx.buf,
        &mut ctx.fds,
    )
    .unwrap();
    let (_bytes, uv) =
        <MyVariant2 as Unmarshal>::unmarshal(crate::ByteOrder::LittleEndian, &ctx.buf, 0).unwrap();
    assert_eq!(
        uv,
        MyVariant2::Catchall(crate::signature::Type::Container(
            crate::signature::Container::Struct(vec![
                crate::signature::Type::Base(crate::signature::Base::String),
                crate::signature::Type::Base(crate::signature::Base::String),
                crate::signature::Type::Base(crate::signature::Base::Byte),
            ])
        ))
    )
}

#[macro_export(local_inner_macros)]
/// This macro provides a convenient way to create enums to represent relatively simple Variants, with fitting marshal/unmarshal implementations.
/// It can be used like this:
/// ```rust, ignore
///    type Map<'buf> = std::collections::HashMap<String, (i32, u8, (u64, MyVariant<'buf>))>;
///    type Struct<'buf> = (u32, u32, MyVariant<'buf>);
///    dbus_variant_var!(MyVariant2, CaseMap => Map<'buf>; CaseStruct => Struct<'buf>);
/// ```
/// And it will generate an enum like this:
/// ```rust, ignore
/// enum MyVariant<'buf> {
///     CaseMap(Map<'buf>),
///     CaseStruct(Struct<'buf>),
///     Catchall(rustbus::wire::unmarshal::traits::Variant<'buf>),   
/// }
/// ```
/// The `Catchall` case is used for unmarshalling, when encountering a Value that did not match any of the other cases. **The generated marshal impl will
/// refuse to marshal the Catchall case!**.
///
/// ## Current limitations
/// 1. References like &str are supported, if you use a type def like this:
///     * `type StrRef<'buf> = &'buf str;`
///     * `dbus_variant_var!(MyVariant, String => StrRef<'buf>; V2 => i32; Integer => u32);`
macro_rules! dbus_variant_var {
    ($vname: ident, $($name: ident => $typ: path);+) => {
        dbus_variant_var_type!($vname, $(
            $name => $typ
        )+);

        impl<'buf> rustbus::Signature for $vname <'buf> {
            fn signature() -> rustbus::signature::Type {
                rustbus::signature::Type::Container(rustbus::signature::Container::Variant)
            }
            fn alignment() -> usize {
                1
            }
        }

        dbus_variant_var_marshal!($vname, $(
            $name => $typ
        )+);
        dbus_variant_var_unmarshal!($vname, $(
            $name => $typ
        )+);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dbus_variant_var_type {
    ($vname: ident, $($name: ident => $typ: path)+) => {
        #[derive(Debug)]
        pub enum $vname <'buf> {
            $(
                $name($typ),
            )+
            Catchall(rustbus::wire::unmarshal::traits::Variant<'buf>)
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dbus_variant_var_marshal {
    ($vname: ident, $($name: ident => $typ: path)+) => {
        impl<'buf> rustbus::Marshal for $vname <'buf> {
            fn marshal(&self, ctx: &mut rustbus::wire::marshal::MarshalContext) -> Result<(), rustbus::Error> {
                use rustbus::Signature;

                match self {
                    $(
                        Self::$name(v) => {
                            let mut sig_str = String::new();
                            <$typ as Signature>::signature().to_str(&mut sig_str);
                            rustbus::wire::marshal::base::marshal_base_param(
                                &rustbus::params::Base::Signature(sig_str),
                                ctx,
                            )
                            .unwrap();
                            v.marshal(ctx)?;
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
macro_rules! dbus_variant_var_unmarshal {
    ($vname: ident, $($name: ident => $typ: path)+) => {
        impl<'ret, 'buf: 'ret> rustbus::Unmarshal<'ret, 'buf> for $vname <'ret> {
            fn unmarshal(
                byteorder: rustbus::ByteOrder,
                buf: &'buf [u8],
                offset: usize,
            ) -> rustbus::wire::unmarshal::UnmarshalResult<Self> {
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
                if sig == <$typ as Signature>::signature() {
                    let (vbytes, v) = <$typ as rustbus::Unmarshal>::unmarshal(byteorder, buf, offset)?;
                    return Ok((bytes + vbytes, Self::$name(v)));
                }
                )+
                let vbytes = rustbus::wire::validate_raw::validate_marshalled(
                    byteorder, offset, buf, &sig
                ).map_err(|e| e.1)?;
                let var = rustbus::wire::unmarshal::traits::Variant {
                    byteorder,
                    buf,
                    offset,
                    sig,
                };
                Ok((bytes + vbytes, Self::Catchall(var)))
            }
        }
    };
}

#[test]
fn test_variant_var_macro() {
    use crate::Marshal;
    use crate::Unmarshal;

    use crate::wire::marshal::MarshalContext;

    let mut fds = Vec::new();
    let mut buf = Vec::new();
    let mut ctx = MarshalContext {
        buf: &mut buf,
        fds: &mut fds,
        byteorder: crate::ByteOrder::LittleEndian,
    };
    let ctx = &mut ctx;

    // so the macro is able to use rustbus, like it would have to when importet into other crates
    use crate as rustbus;

    type StrRef<'buf> = &'buf str;
    dbus_variant_var!(MyVariant, String => StrRef<'buf>; V2 => i32; Integer => u32);
    let v1 = MyVariant::String("ABCD");
    let v2 = MyVariant::V2(0);
    let v3 = MyVariant::Integer(100);

    (&v1, &v2, &v3).marshal(ctx).unwrap();
    // add a unknown variant here
    crate::message_builder::marshal_as_variant(
        0xFFFFu64,
        crate::ByteOrder::LittleEndian,
        &mut ctx.buf,
        &mut ctx.fds,
    )
    .unwrap();

    let (bytes, (uv1, uv2, uv3)) = <(MyVariant, MyVariant, MyVariant) as Unmarshal>::unmarshal(
        crate::ByteOrder::LittleEndian,
        &ctx.buf,
        0,
    )
    .unwrap();
    assert!(match uv1 {
        MyVariant::String(s) => s.eq("ABCD"),
        _ => false,
    });
    assert!(match uv2 {
        MyVariant::V2(s) => s.eq(&0),
        _ => false,
    });
    assert!(match uv3 {
        MyVariant::Integer(s) => s.eq(&100),
        _ => false,
    });

    let (_bytes, uv4) =
        MyVariant::unmarshal(crate::ByteOrder::LittleEndian, &ctx.buf, bytes).unwrap();
    assert!(match uv4 {
        MyVariant::Catchall(var) => {
            var.get::<u64>().unwrap() == 0xFFFFu64
        }
        _ => false,
    });

    type Map<'buf> = std::collections::HashMap<String, (i32, u8, (u64, MyVariant<'buf>))>;
    type Struct<'buf> = (u32, u32, MyVariant<'buf>);
    dbus_variant_var!(MyVariant2, CaseMap => Map<'buf>; CaseStruct => Struct<'buf>);

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

    ctx.buf.clear();
    (&v1, &v2, &v3, &v4).marshal(ctx).unwrap();
    let (_bytes, (uv1, uv2, uv3, uv4)) =
        <(MyVariant2, MyVariant2, MyVariant2, MyVariant2) as Unmarshal>::unmarshal(
            crate::ByteOrder::LittleEndian,
            &ctx.buf,
            0,
        )
        .unwrap();
    assert!(match uv1 {
        MyVariant2::CaseMap(map) => {
            {
                let a = map.get("AAAA").unwrap();
                assert!(a.0 == 100);
                assert!(a.1 == 20);
                assert!((a.2).0 == 300);
                assert!(match (a.2).1 {
                    MyVariant::String(s) => s.eq("BBBB"),
                    _ => false,
                });
            }
            {
                let c = map.get("CCCC").unwrap();
                assert!(c.0 == 400);
                assert!(c.1 == 50);
                assert!((c.2).0 == 600);
                assert!(match (c.2).1 {
                    MyVariant::V2(s) => s.eq(&0),
                    _ => false,
                });
            }
            {
                let d = map.get("DDDD").unwrap();
                assert!(d.0 == 500);
                assert!(d.1 == 60);
                assert!((d.2).0 == 700);
                assert!(match (d.2).1 {
                    MyVariant::Integer(s) => s.eq(&10),
                    _ => false,
                });
            }
            true
        }
        _ => false,
    });
    assert!(match uv2 {
        MyVariant2::CaseStruct(strct) => {
            assert!(strct.0 == 10);
            assert!(strct.1 == 20);
            assert!(match strct.2 {
                MyVariant::String(s) => s.eq("AAAAA"),
                _ => false,
            });
            true
        }
        _ => false,
    });
    assert!(match uv3 {
        MyVariant2::CaseStruct(strct) => {
            assert!(strct.0 == 30);
            assert!(strct.1 == 40);
            assert!(match strct.2 {
                MyVariant::V2(s) => s.eq(&10),
                _ => false,
            });
            true
        }
        _ => false,
    });
    assert!(match uv4 {
        MyVariant2::CaseStruct(strct) => {
            assert!(strct.0 == 30);
            assert!(strct.1 == 40);
            assert!(match strct.2 {
                MyVariant::Integer(s) => s.eq(&20),
                _ => false,
            });
            true
        }
        _ => false,
    });

    // Test that catchall gets the right signatures
    ctx.buf.clear();
    crate::message_builder::marshal_as_variant(
        ("testtext", "moretesttext", 100u8),
        crate::ByteOrder::LittleEndian,
        &mut ctx.buf,
        &mut ctx.fds,
    )
    .unwrap();
    let (_bytes, uv) =
        <MyVariant2 as Unmarshal>::unmarshal(crate::ByteOrder::LittleEndian, &ctx.buf, 0).unwrap();
    assert!(match uv {
        MyVariant2::Catchall(var) => {
            var.get::<(&str, &str, u8)>()
                .unwrap()
                .eq(&("testtext", "moretesttext", 100u8))
                && var.sig
                    == crate::signature::Type::Container(crate::signature::Container::Struct(vec![
                        crate::signature::Type::Base(crate::signature::Base::String),
                        crate::signature::Type::Base(crate::signature::Base::String),
                        crate::signature::Type::Base(crate::signature::Base::Byte),
                    ]))
        }
        _ => false,
    });
}
