//! A bit more convenient ways to make containers
//!
//! These allow for easier construction of containers. Note that empty containers require you to specify the
//! signature.

use crate::params::message::Result;
use crate::params::*;
use crate::signature;

impl<'e, 'a: 'e> Container<'a, 'e> {
    pub fn push<P: Into<Param<'a, 'e>>>(&mut self, new: P) -> Result<()> {
        match self {
            Container::Struct(elements) => {
                elements.push(new.into());
                Ok(())
            }
            Container::Array(arr) => {
                let new = new.into();
                if arr.element_sig.eq(&new.sig()) {
                    arr.values.push(new);
                    Ok(())
                } else {
                    Err(crate::params::validation::Error::ArrayElementTypesDiffer.into())
                }
            }
            _ => Err(crate::params::validation::Error::InvalidSignature(
                signature::Error::InvalidSignature,
            )
            .into()),
        }
    }
    pub fn insert<K: Into<Base<'a>>, V: Into<Param<'a, 'e>>>(
        &mut self,
        key: K,
        val: V,
    ) -> Result<()> {
        match self {
            Container::Dict(dict) => {
                let key = key.into();
                let val = val.into();
                if !key.sig().eq(&signature::Type::Base(dict.key_sig)) {
                    return Err(crate::params::validation::Error::DictKeyTypesDiffer.into());
                }
                if !val.sig().eq(&dict.value_sig) {
                    return Err(crate::params::validation::Error::DictKeyTypesDiffer.into());
                }
                dict.map.insert(key, val);
                Ok(())
            }
            _ => Err(crate::params::validation::Error::InvalidSignature(
                signature::Error::InvalidSignature,
            )
            .into()),
        }
    }
}

impl<'e, 'a: 'e> Container<'a, 'e> {
    pub fn make_struct<P: Into<Param<'a, 'e>>>(elements: Vec<P>) -> Container<'a, 'e> {
        Container::Struct(elements.into_iter().map(std::convert::Into::into).collect())
    }
    pub fn make_struct_ref(elements: &'a [Param<'a, 'e>]) -> Container<'a, 'e> {
        Container::StructRef(elements)
    }
    pub fn make_struct1<P: Into<Param<'a, 'e>>>(e1: P) -> Container<'a, 'e> {
        Container::Struct(vec![e1.into()])
    }
    pub fn make_struct2<P1: Into<Param<'a, 'e>>, P2: Into<Param<'a, 'e>>>(
        e1: P1,
        e2: P2,
    ) -> Container<'a, 'e> {
        Container::Struct(vec![e1.into(), e2.into()])
    }
    pub fn make_struct3<
        P1: Into<Param<'a, 'e>>,
        P2: Into<Param<'a, 'e>>,
        P3: Into<Param<'a, 'e>>,
    >(
        e1: P1,
        e2: P2,
        e3: P3,
    ) -> Container<'a, 'e> {
        Container::Struct(vec![e1.into(), e2.into(), e3.into()])
    }

    pub fn make_variant<P: Into<Param<'a, 'e>>>(element: P) -> Container<'a, 'e> {
        let param: Param = element.into();

        Container::Variant(Box::new(Variant {
            sig: param.sig(),
            value: param,
        }))
    }

    pub fn make_array_ref(
        element_sig: &str,
        elements: &'a [Param<'a, 'e>],
    ) -> Result<Container<'a, 'e>> {
        let mut sigs = signature::Type::parse_description(element_sig)?;

        if sigs.len() != 1 {
            return Err(crate::signature::Error::TooManyTypes.into());
        }

        let sig = sigs.remove(0);
        Self::make_array_ref_with_sig(sig, elements)
    }

    pub fn make_array_ref_with_sig(
        element_sig: signature::Type,
        elements: &'a [Param<'a, 'e>],
    ) -> Result<Container<'a, 'e>> {
        let arr: ArrayRef<'a, 'e> = ArrayRef {
            element_sig,
            values: elements,
        };

        validate_array(&arr.values, &arr.element_sig)?;

        Ok(Container::ArrayRef(arr))
    }

    pub fn make_array<P: Into<Param<'a, 'e>>, I: Iterator<Item = P>>(
        element_sig: &str,
        elements: I,
    ) -> Result<Container<'a, 'e>> {
        let mut sigs = signature::Type::parse_description(element_sig)?;

        if sigs.len() != 1 {
            return Err(crate::signature::Error::TooManyTypes.into());
        }

        let sig = sigs.remove(0);
        Self::make_array_with_sig(sig, elements)
    }

    pub fn make_array_with_sig<P: Into<Param<'a, 'e>>, I: Iterator<Item = P>>(
        element_sig: signature::Type,
        elements: I,
    ) -> Result<Container<'a, 'e>> {
        let arr: Array<'a, 'e> = Array {
            element_sig,
            values: elements.map(std::convert::Into::into).collect(),
        };

        validate_array(&arr.values, &arr.element_sig)?;

        Ok(Container::Array(arr))
    }

    pub fn make_dict<K: Into<Base<'e>>, V: Into<Param<'a, 'e>>, I: Iterator<Item = (K, V)>>(
        key_sig: &str,
        val_sig: &str,
        map: I,
    ) -> Result<Container<'a, 'e>> {
        let mut valsigs = signature::Type::parse_description(val_sig)?;

        if valsigs.len() != 1 {
            return Err(crate::signature::Error::TooManyTypes.into());
        }

        let value_sig = valsigs.remove(0);
        let mut keysigs = signature::Type::parse_description(key_sig)?;

        if keysigs.len() != 1 {
            return Err(crate::signature::Error::TooManyTypes.into());
        }
        let key_sig = keysigs.remove(0);
        let key_sig = if let signature::Type::Base(sig) = key_sig {
            sig
        } else {
            return Err(crate::signature::Error::ShouldBeBaseType.into());
        };

        Self::make_dict_with_sig(key_sig, value_sig, map)
    }

    pub fn make_dict_with_sig<
        K: Into<Base<'e>>,
        V: Into<Param<'a, 'e>>,
        I: Iterator<Item = (K, V)>,
    >(
        key_sig: signature::Base,
        value_sig: signature::Type,
        map: I,
    ) -> Result<Container<'a, 'e>> {
        let dict = Dict {
            key_sig,
            value_sig,
            map: map.map(|(k, v)| (k.into(), v.into())).collect(),
        };

        validate_dict(&dict.map, dict.key_sig, &dict.value_sig)?;

        Ok(Container::Dict(dict))
    }
    pub fn make_dict_ref(
        key_sig: &str,
        val_sig: &str,
        map: &'a DictMap,
    ) -> Result<Container<'a, 'e>> {
        let mut valsigs = signature::Type::parse_description(val_sig)?;

        if valsigs.len() != 1 {
            return Err(crate::signature::Error::TooManyTypes.into());
        }

        let value_sig = valsigs.remove(0);
        let mut keysigs = signature::Type::parse_description(key_sig)?;

        if keysigs.len() != 1 {
            return Err(crate::signature::Error::TooManyTypes.into());
        }
        let key_sig = keysigs.remove(0);
        let key_sig = if let signature::Type::Base(sig) = key_sig {
            sig
        } else {
            return Err(crate::signature::Error::ShouldBeBaseType.into());
        };

        Self::make_dict_ref_with_sig(key_sig, value_sig, map)
    }

    pub fn make_dict_ref_with_sig(
        key_sig: signature::Base,
        value_sig: signature::Type,
        map: &'a DictMap,
    ) -> Result<Container<'a, 'e>> {
        let dict = DictRef {
            key_sig,
            value_sig,
            map,
        };

        validate_dict(&dict.map, dict.key_sig, &dict.value_sig)?;

        Ok(Container::DictRef(dict))
    }
}
