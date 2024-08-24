#[macro_export]
macro_rules! vk_enumerate_to_vec {
    ($vk_function:ident, $vk_struct:ty, $($var:expr,)*) => {{
        let mut len: u32 = 0;

        $vk_function(
            $($var,)* &mut len, std::ptr::null_mut()
        );

        let mut vec: Vec<$vk_struct> = Vec::with_capacity(len as usize);
        vec.set_len(len as usize);

        $vk_function(
            $($var,)* &mut len, vec.as_mut_ptr()
        );

        vec
    }};
}

#[macro_export]
macro_rules! prepare_extensions {
    ($supported_extensions:expr, $($extension:expr,)*) => {{
        let supported_extensions: Vec<String> = $supported_extensions.iter()
            .map(|x| {
                let extension_name_u8: Vec<u8> = x.extension_name.iter().map(|y| *y as u8).filter(|y| *y != 0).collect();
                let extension_name_slice: &[u8] = &extension_name_u8;
                return std::str::from_utf8(extension_name_slice).expect("Invalid extension name").to_string()

        }).collect();

        let mut extension_ptrs: Vec<*const c_char> = vec!();

        $(
            let extension = CString::new($extension).expect("Invalid user extension name!");

            if supported_extensions.iter().position(|x| CString::new(x.as_str()).unwrap() == extension).is_some() {
                let ptr = extension.as_ptr();

                std::mem::forget(extension);
                let _ = ptr;

                extension_ptrs.push(ptr);
            } else {dbg!("ignoring: {:?}", extension);}
        )*

        let extensions = extension_ptrs.as_ptr();

        let extensions_len = extension_ptrs.len() as u32;

        std::mem::forget(extension_ptrs);

        (extensions, extensions_len)
    }};
}
