use crate::{
    level_data::{ids, speed::Speed},
    util::{int_to_bool, parse},
    Parse,
};
use gdcf::error::ValueError;

#[derive(Debug)]
pub enum ObjectMetadata {
    None,
    Portal(PortalMetadata),
}

#[derive(Debug)]
pub struct PortalMetadata {
    pub checked: bool,
    pub portal_type: PortalType,
}

#[derive(Debug)]
pub enum PortalType {
    Nonsense,
    Speed(Speed),
    // .. all the other portals ..
}

impl PortalType {
    pub fn from_id(id: u16) -> PortalType {
        match id {
            ids::SLOW_PORTAL => PortalType::Speed(Speed::Slow),
            ids::NORMAL_PORTAL => PortalType::Speed(Speed::Normal),
            ids::MEDIUM_PORTAL => PortalType::Speed(Speed::Medium),
            ids::FAST_PORTAL => PortalType::Speed(Speed::Fast),
            ids::VERY_FAST_PORTAL => PortalType::Speed(Speed::VeryFast),
            _ => PortalType::Nonsense,
        }
    }
}

impl Parse for ObjectMetadata {
    fn parse<'a, I, F>(mut iter: I, mut f: F) -> Result<Self, ValueError<'a>>
    where
        I: Iterator<Item = (&'a str, &'a str)> + Clone,
        F: FnMut(&'a str, &'a str) -> Result<(), ValueError<'a>>,
    {
        let id = match iter.clone().find(|(idx, _)| idx == &"1") {
            Some((_, id)) => parse(1, id)?.ok_or(ValueError::NoValue(1))?,
            None => return Err(ValueError::NoValue(1)),
        };

        match id {
            ids::SLOW_PORTAL | ids::NORMAL_PORTAL | ids::MEDIUM_PORTAL | ids::FAST_PORTAL | ids::VERY_FAST_PORTAL =>
                Ok(ObjectMetadata::Portal(PortalMetadata::parse(iter, f)?)),
            // .. all the other types of metadata, which might have proper parsers ...
            _ => {
                // We aren't delegating further, so we gotta drive the iterator to completion
                for (idx, value) in iter {
                    f(idx, value)?
                }

                Ok(ObjectMetadata::None)
            },
        }
    }
}

parser! {
    PortalMetadata => {
        checked(index = 13, with = int_to_bool),
        portal_type(custom = PortalType::from_id, depends_on = [id]),
    },
    id(index = 1),
}