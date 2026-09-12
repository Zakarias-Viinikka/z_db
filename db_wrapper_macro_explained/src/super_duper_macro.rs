#[macro_export]
macro_rules! db_wrapper_method_generator {
    () => {
        method_attrs!();
        pub fn get_thing(
            &self,
            data: method_input_type!(GetThingIn),
        ) -> method_return_type!(GetThingOut) {
            let data: GetThingIn = decode_input!(data);
            let conn = unwrap_or_bail!(self.get_conn());
            let raw = unwrap_or_bail!(db_get_thing(conn, data.id));
            finish_output!(GetThingOut { value: raw })
        }
    };
}
