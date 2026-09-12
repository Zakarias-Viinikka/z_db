#[macro_export]
macro_rules! create_the_entire_universe {
    ($type:ty $(, $attr:meta)*) => {
        $(#[$attr])*
        impl $type {
            pub fn get_thing(
                &self,
                data: method_input_type!(GetThingIn),
            ) -> method_return_type!(GetThingOut) {
                let data: GetThingIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                let raw = unwrap_or_bail!($crate::db_operations::get_thing(&*conn, data.id));
                finish_output!(GetThingOut { value: raw })
            }

            pub fn get_combined(
                &self,
                data: method_input_type!(GetCombinedIn),
            ) -> method_return_type!(CombinedOut) {
                let data: GetCombinedIn = decode_input!(data);
                let conn = unwrap_or_bail!(self.get_conn());
                let value = unwrap_or_bail!($crate::db_operations::get_thing(&*conn, data.id));
                let other = unwrap_or_bail!($crate::db_operations::get_other_thing(&*conn));
                finish_output!(CombinedOut { value, other })
            }
        }
    };
}
