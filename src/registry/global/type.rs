use camelpaste::paste;
use crate::plc::interface::status::{BodyStatus, InterfaceStatus};
use crate::plc::interface::traits::DeferredBuilder;
use crate::plc::pou::db::db::Db;
use crate::plc::pou::fb::Fb;
use crate::plc::pou::fc::Fc;
use crate::plc::pou::ob::Ob;
use crate::plc::pou::udt::Udt;
use crate::registry::registry::Kernel;
use crate::container::broadcast::broadcast::Broadcast;
use crate::container::error::error::Stop;

pub enum GlobalType {
    Ob(Ob),
    Fb(Fb),
    Fc(Fc),
    Db(Db),
    Udt(Udt)
}

impl GlobalType {
    pub fn get_interface_status(&self) -> InterfaceStatus {
        match self {
            GlobalType::Ob(ob) => ob.get_interface_status(),
            GlobalType::Fb(fb) => fb.get_interface_status(),
            GlobalType::Fc(fc) => fc.get_interface_status(),
            GlobalType::Db(db) => match db {
                Db::Global(global) => global.get_interface_status(),
                Db::Instance(instance) => instance.get_interface_status()
            },
            GlobalType::Udt(udt) => udt.get_interface_status()
        }
    }

    pub fn get_body_status(&self) -> BodyStatus {
        match self {
            GlobalType::Ob(ob) => ob.get_body_status(),
            GlobalType::Fb(fb) => fb.get_body_status(),
            GlobalType::Fc(fc) => fc.get_body_status(),
            GlobalType::Db(db) => match db {
                Db::Global(global) => global.get_body_status(),
                Db::Instance(instance) => instance.get_body_status()
            },
            GlobalType::Udt(udt) => udt.get_body_status()
        }
    }

    pub fn build_interface(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<(), Stop> {
        match self {
            GlobalType::Ob(ob) => ob.build_interface(registry, channel),
            GlobalType::Fb(fb) => fb.build_interface(registry, channel),
            GlobalType::Fc(fc) => fc.build_interface(registry, channel),
            GlobalType::Db(db) =>
                match db {
                    Db::Global(global) => global.build_interface(registry, channel),
                    Db::Instance(instance) => instance.build_interface(registry, channel),
                },
            GlobalType::Udt(udt) => udt.build_interface(registry, channel)
        }
    }

    pub fn build_body(&mut self, registry: &Kernel, channel: &Broadcast) -> Result<(), Stop> {
        match self {
            GlobalType::Ob(ob) => { ob.build_body(registry, channel)?; Ok(()) },
            GlobalType::Fb(fb) => fb.build_body(registry, channel),
            GlobalType::Fc(fc) => fc.build_body(registry, channel),
            GlobalType::Db(db) =>
                match db {
                    Db::Global(global) => global.build_body(registry, channel),
                    Db::Instance(instance) => instance.build_body(registry, channel)
                },
            GlobalType::Udt(udt) => udt.build_body(registry, channel)
        }
    }
}

macro_rules! impl_all_global_types {
    ($({$primitive: ty, $global_type: ident}),+) => {
        paste! {
            impl GlobalType {
                $(
                    pub fn [<is_$primitive>](&self) -> bool {
                        match self {
                            GlobalType::$global_type(_) => true,
                            _ => false,
                        }
                    }
                )+
            }
        }
    };
}

impl_all_global_types!(
    { udt, Udt },
    { db, Db },
    { fc, Fc },
    { fb, Fb },
    { ob, Ob }
);
