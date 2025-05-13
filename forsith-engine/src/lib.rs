pub const ENGINE_NAME: fn() -> CString = || CString::new("forsith").unwrap();

pub const ENGINE_VERSION: fn() -> VkVersion = || {
    let [major, minor, patch] = env!("CARGO_PKG_VERSION").split('.')
        .map(|num| num.parse::<u32>().expect("incorrect crate version format (incorrect num)"))
        .collect::<Vec<u32>>()
        .try_into().expect("incorrect crate version format (to much nums)");

    return vk_version(major, minor, patch);
};


#[cfg(test)]
#[test]
fn test_engine_version() {assert_eq!(ENGINE_VERSION(), vk_version(0, 1, 0))}
