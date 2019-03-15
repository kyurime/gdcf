use crate::{error::ValueError, Parse};
use gdcf_model::level::data::{
    ids,
    portal::{PortalData, PortalType},
    text::TextData,
    trigger::ColorTriggerData,
    ObjectData,
};

impl Parse for ObjectData {
    fn parse<'a, I, F>(iter: I, mut f: F) -> Result<Self, ValueError<'a>>
    where
        I: Iterator<Item = (&'a str, &'a str)> + Clone,
        F: FnMut(&'a str, &'a str) -> Result<(), ValueError<'a>>,
    {
        let id = iter
            .clone()
            .find(|(idx, _)| idx == &"1")
            .map(|(_, id)| id)
            .ok_or(ValueError::NoValue("1"))?;

        match id {
            ids::S_SLOW_PORTAL | ids::S_NORMAL_PORTAL | ids::S_MEDIUM_PORTAL | ids::S_FAST_PORTAL | ids::S_VERY_FAST_PORTAL =>
                Ok(ObjectData::Portal(PortalData::parse(iter, f)?)),
            // .. all the other types of metadata, which might have proper parsers ...
            _ => {
                // We aren't delegating further, so we gotta drive the iterator to completion
                for (idx, value) in iter {
                    f(idx, value)?
                }

                Ok(ObjectData::None)
            },
        }
    }
}

parser! {
    PortalData => {
        checked(index = 13),
        portal_type(custom = PortalType::from_id_str, depends_on = [id]),
    },
    id(^index = 1, noparse),
}

parser! {
    ColorTriggerData => {
        r(index = 7),
        g(index = 8),
        b(index = 9),
        blending_enabled(index = 17),
    }
}

parser! {
    TextData => {
        text(index = 31),
    }
}
