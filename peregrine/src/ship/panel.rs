use strum::FromRepr;
use tethys::prelude::Object;

use super::PartLoader;


#[repr(usize)]
#[derive(Copy, Clone, Debug, FromRepr)]
pub enum PanelModel {
    Metal,
}

#[derive(Clone, Copy, Debug)]
pub struct Panel {

}
impl Panel {
    pub(crate) fn get_objects(&self, loader: PartLoader, layout: PanelLayout) -> Object {
        unimplemented!()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PanelLayout {

}