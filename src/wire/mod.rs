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

#[test]
fn test_variant_macro() {
    macro_rules! dbus_variant {
        ($vname: ident, $($name: ident => $typ: ident);+) => {
            dbus_variant_type!($vname, $(
                $name => $typ
            )+);

            impl marshal::traits::Signature for $vname {
                fn signature() -> crate::signature::Type {
                    crate::signature::Type::Container(crate::signature::Container::Variant)
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

    macro_rules! dbus_variant_type {
        ($vname: ident, $($name: ident => $typ: ident)+) => {
            #[derive(Eq, PartialEq, Debug)]
            pub enum $vname {
                $(
                    $name($typ),
                )+
                Catchall(crate::signature::Type)
            }
        };
    }

    macro_rules! dbus_variant_marshal {
        ($vname: ident, $($name: ident => $typ: ident)+) => {
            impl crate::Marshal for $vname {
                fn marshal(&self, byteorder: crate::ByteOrder, buf: &mut Vec<u8>) -> Result<(), crate::Error> {
                    use crate::Signature;

                    match self {
                        $(
                            Self::$name(v) => {
                                let mut sig_str = String::new();
                                $typ::signature().to_str(&mut sig_str);
                                crate::wire::marshal::base::marshal_base_param(
                                    byteorder,
                                    &crate::params::Base::Signature(sig_str),
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
    macro_rules! dbus_variant_unmarshal {
        ($vname: ident, $($name: ident => $typ: ident)+) => {
            impl<'ret, 'buf: 'ret> unmarshal::traits::Unmarshal<'ret, 'buf> for $vname {
                fn unmarshal(
                    byteorder: crate::ByteOrder,
                    buf: &'buf [u8],
                    offset: usize,
                ) -> unmarshal::UnmarshalResult<Self> {
                    use crate::Signature;

                    let (bytes, sig_str) = crate::wire::util::unmarshal_signature(&buf[offset..])?;
                    let mut sig = crate::signature::Type::parse_description(&sig_str)?;
                    let sig = if sig.len() == 1 {
                        sig.remove(0)
                    } else {
                        return Err(crate::wire::unmarshal::Error::WrongSignature);
                    };
                    let offset = offset + bytes;

                    $(
                    if sig == $typ::signature() {
                        let (vbytes, v) = $typ::unmarshal(byteorder, buf, offset)?;
                        return Ok((bytes + vbytes, Self::$name(v)));
                    }
                    )+
                    let (vbytes, _p) = crate::wire::unmarshal::container::unmarshal_with_sig(
                        byteorder, &sig, buf, offset,
                    )?;
                    Ok((bytes + vbytes, Self::Catchall(sig)))
                }
            }
        };
    }

    use crate::Marshal;
    use crate::Unmarshal;

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
}
