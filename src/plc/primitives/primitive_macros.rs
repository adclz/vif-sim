#[macro_export]
/// Automatic creation of any plc primitive.
///
/// This will handle all cases as far as you don't have a specific behavior to implement.
macro_rules! impl_primitive_all {
    ($primitive: ident, $inner_type: ident) => {
        crate::impl_primitive_base!($primitive, $inner_type);
        crate::impl_primitive_type_name!($primitive, $inner_type);
        crate::impl_primitive_raw_mut!($primitive, $inner_type);
        crate::impl_primitive_display!($primitive, $inner_type);
        crate::impl_primitive_serialize!($primitive, $inner_type);
    };
}

#[macro_export]
macro_rules! impl_primitive_traits {
    ($atype: ident, { $($for_type: expr,
    [$(direct $boolean: ident)? $(self.$available: expr)?],
    [$($(get_mut $get_mut: ident),+)? $(stop $stop: expr)?],
    [$($(get $get: ident),+)? $(none $none: expr)?]
    ),+ }) => {
        paste! {
            impl Primitive for $atype {
                $(
                    fn [<is_$for_type>](&self) -> bool { $($boolean)? $(self.$available())? }
                    fn [<as_$for_type>](&self, channel: &Broadcast) -> Result<$for_type, Stop> {
                        $(
                            $(
                                if let Ok(a) = self.$get() {
                                    return a.get(channel)
                                };
                            )+
                            Err(error!(format!("0")))
                        )?
                        $($none)?
                    }
                )+
            }
            impl AsMutPrimitive for $atype {
                $(
                     fn [<set_$for_type>](&mut self, other: $for_type, channel: &Broadcast) -> Result<(), Stop> {
                         $(
                            $(
                                if let Ok(a) = self.$get_mut() {
                                    return a.set(other, channel)
                                };
                            )+
                            Err(error!(format!("0")))
                         )?
                         $($stop)?
                     }
                     fn [<set_default_$for_type>](&mut self, other: $for_type) -> Result<(), Stop> {
                         $(
                            $(
                                if let Ok(a) = self.$get_mut() {
                                    return a.set_default(other)
                                };
                            )+
                            Err(error!(format!("0")))
                         )?
                         $($stop)?
                     }
                )+
            }
        }
    };
}

#[macro_export]
macro_rules! impl_primitive_type_name {
    ($primitive: ident, $inner_type: ident) => {
        impl MetaData for $primitive {
            fn name(&self) -> &'static str {
                &stringify!($primitive)
            }

            fn get_alias_str<'a>(&self, kernel: &'a Kernel) -> Option<&'a String> {
                match self.alias {
                    Some(a) => kernel.get_type_alias_str(a),
                    None => None
                }
            }

            fn get_alias_id(&self, kernel: & Kernel) -> Option<usize> {
                self.alias
            }

            fn is_read_only(&self) -> bool {
                self.read_only
            }
        }
        
        impl SetMetaData for $primitive {
            fn set_alias(&mut self, alias: &str, kernel: &Kernel) {
                self.alias = kernel.get_type_alias_id(alias)
            }
            fn set_read_only(&mut self, value: bool) {
                self.read_only = value;
            }
        }
    };
}

#[macro_export]
macro_rules! impl_primitive_raw_mut {
    ($primitive: ident, $inner_type: ident) => {
        impl RawMut for $primitive {
            fn reset_ptr(&mut self, channel: &Broadcast) {
                self.reset(channel)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_primitive_display {
    ($primitive: ident, $inner_type: ident) => {
        impl Display for $primitive {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self.value)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_primitive_serialize {
    ($primitive: ident, $inner_type: ident) => {
        impl Serialize for $primitive {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut data = serializer.serialize_struct("data", 3)?;
                data.serialize_field("ty", &format!("{}", stringify!($primitive)))?;
                data.serialize_field("id", &self.id)?;
                data.serialize_field("value", &format!("{:?}", self.value))?;
                data.end()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_primitive_base {
    ($primitive: ident, $inner_type: ident) => {
        impl ToggleMonitor for $primitive {
            fn set_monitor(&mut self, activate: bool) {
                self.monitor = activate
            }
        }

        impl PrimitiveTrait for $primitive {
            type Native = $inner_type;
            type PlcPrimitive = $primitive;

            fn new(value: &$inner_type) -> Result<Self::PlcPrimitive, Stop> {
                Ok(Self {
                    default: value.clone(),
                    value: value.clone(),
                    id: get_id(),
                    monitor: false,
                    read_only: false,
                    alias: None
                })
            }

            fn get(&self, channel: &Broadcast) -> Result<$inner_type, Stop> {
                Ok(self.value)
            }

            fn set(&mut self, value: $inner_type, channel: &Broadcast) -> Result<(), Stop> {
                self.value = value;
                self.monitor(channel);
                Ok(())
            }

            fn set_default(&mut self, value: $inner_type) -> Result<(), Stop> {
                self.default = value;
                self.value = self.default;
                Ok(())
            }

            fn reset(&mut self, channel: &Broadcast) {
                self.value = self.default;
                self.monitor(channel)
            }

            fn monitor(&self, channel: &Broadcast) {
                channel.add_monitor_change(&MonitorChange::new(self.id, format!("{}", self)))
            }

            fn get_id(&self) -> usize {
                self.id
            }

            fn get_type_id(&self) -> TypeId {
                self.value.type_id()
            }
        }
    };
}
