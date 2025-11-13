use std::{
    env::{self, VarError},
    ffi::{CStr},
    path::{PathBuf},
    sync::{Arc},
};

use libc::{c_char};

use katsuba::cli::{
    HYPHEN,
    io::{InputsOutputs},
};

use katsuba::cmd::{
    bcd::{deserialize as rust_bcd_deserialize},
    cs::{KATSUBA_CLIENTSIG_PRIVATE_KEY, DEFAULT_OUTPUT_FILE, arg as rust_cs_arg, decrypt as rust_cs_decrypt},
    hash::{Algo, hash},
    nav::{deserialize_nav as rust_nav_deserialize, deserialize_zonenav as rust_zonenav_deserialize},
    op::{deserialize as rust_op_deserialize},
    op::guess,
    op::utils::{merge_type_lists},
    poi::{deserialize as rust_poi_deserialize},
    wad::{wad_pack as rust_wad_pack, wad_unpack as rust_wad_unpack},
};

use katsuba_object_property::serde;

use katsuba_types::{PropertyFlags, TypeList};

#[no_mangle]
pub extern "C" fn bcd_deserialize(input: *const c_char, output: *const c_char) -> bool {
    let default_path = PathBuf::from(HYPHEN);

    let rust_input = if input.is_null() {
        return false
    } else {
        match unsafe { CStr::from_ptr(input) }.to_str() {
            Ok(rust_str) => rust_str.to_owned(),
            Err(_) => return false,
        }
    };

    let rust_output = if output.is_null() {
        default_path
    } else {
        match unsafe { CStr::from_ptr(output) }.to_str() {
            Ok(rust_str) => PathBuf::from(rust_str),
            Err(_) => default_path,
        }
    };

    let io = InputsOutputs {
        input: rust_input,
        output: rust_output,
    };

    rust_bcd_deserialize(io).is_ok()
}

fn get_private_key_path_from_c(private_key_file: *const c_char) -> eyre::Result<PathBuf, VarError> {
    let private_key = if !private_key_file.is_null() {
        match unsafe { CStr::from_ptr(private_key_file) }.to_str() {
            Ok(rust_str) => rust_str,
            Err(_) => "",
        }
    } else {
        ""
    };

    // No private key file was given, try the environment variable instead
    if private_key == "" {
        match env::var(KATSUBA_CLIENTSIG_PRIVATE_KEY) {
            Ok(value) => Ok(PathBuf::from(value)),
            Err(error) => Err(error), // No value found for environment variable
        }
    } else {
        Ok(PathBuf::from(private_key))
    }
}

#[no_mangle]
pub extern "C" fn cs_arg(private_key_file: *const c_char) -> bool {
    let private_key = match get_private_key_path_from_c(private_key_file) {
        Ok(key) => key,
        Err(_) => return false,
    };

    rust_cs_arg(&private_key).is_ok()
}

#[no_mangle]
pub extern "C" fn cs_decrypt(private_key_file: *const c_char, path: *const c_char, output: *const c_char) -> bool {
    let private_key = match get_private_key_path_from_c(private_key_file) {
        Ok(key) => key,
        Err(_) => return false,
    };

    let rust_path = if path.is_null() {
        return false
    } else {
        match unsafe { CStr::from_ptr(path) }.to_str() {
            Ok(rust_str) => PathBuf::from(rust_str),
            Err(_) => return false,
        }
    };

    let rust_output = if output.is_null() {
        PathBuf::from(DEFAULT_OUTPUT_FILE)
    } else {
        match unsafe { CStr::from_ptr(output) }.to_str() {
            Ok(rust_str) => PathBuf::from(rust_str),
            Err(_) => PathBuf::from(DEFAULT_OUTPUT_FILE),
        }
    };

    rust_cs_decrypt(&private_key, rust_path, rust_output).is_ok()
}

/// The hash algorithm to apply. Duplicate of Algo enum.
///
/// This enum is accessible from C.
#[repr(C)]
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CAlgo {
    /// The KingsIsle string ID algorithm.
    StringId,
    /// The DJB2 algorithm.
    Djb2,
}
impl From<&CAlgo> for Algo {
    fn from(algo: &CAlgo) -> Self {
        match algo {
            CAlgo::StringId => Algo::StringId,
            CAlgo::Djb2 => Algo::Djb2,
        }
    }
}

#[no_mangle]
pub extern "C" fn hash_c(input: *const c_char, algo: CAlgo) -> bool {
    let rust_input = if input.is_null() {
        return false
    } else {
        match unsafe { CStr::from_ptr(input) }.to_str() {
            Ok(rust_str) => rust_str.to_owned(),
            Err(_) => return false,
        }
    };

    let rust_algo = Algo::from(&algo);

    hash(&rust_input, rust_algo).is_ok()
}

#[no_mangle]
pub extern "C" fn nav_deserialize(
    input: *const c_char,
    output: *const c_char,
) -> bool {
    let default_path = PathBuf::from(HYPHEN);

    let rust_input = if input.is_null() {
        return false
    } else {
        match unsafe { CStr::from_ptr(input) }.to_str() {
            Ok(rust_str) => rust_str.to_owned(),
            Err(_) => return false,
        }
    };

    let rust_output = if output.is_null() {
        default_path
    } else {
        match unsafe { CStr::from_ptr(output) }.to_str() {
            Ok(rust_str) => PathBuf::from(rust_str),
            Err(_) => default_path,
        }
    };

    let io = InputsOutputs {
        input: rust_input,
        output: rust_output,
    };

    rust_nav_deserialize(io).is_ok()
}

#[no_mangle]
pub extern "C" fn zonenav_deserialize(
    input: *const c_char,
    output: *const c_char,
) -> bool {
    let default_path = PathBuf::from(HYPHEN);

    let rust_input = if input.is_null() {
        return false
    } else {
        match unsafe { CStr::from_ptr(input) }.to_str() {
            Ok(rust_str) => rust_str.to_owned(),
            Err(_) => return false,
        }
    };

    let rust_output = if output.is_null() {
        default_path
    } else {
        match unsafe { CStr::from_ptr(output) }.to_str() {
            Ok(rust_str) => PathBuf::from(rust_str),
            Err(_) => default_path,
        }
    };

    let io = InputsOutputs {
        input: rust_input,
        output: rust_output,
    };

    rust_zonenav_deserialize(io).is_ok()
}

fn get_type_lists_from_c(type_lists: *const *const c_char) -> eyre::Result<Arc<TypeList>> {
    let mut type_list_paths = Vec::new();
    let mut i = 0;
    loop {
        let current_type_list = unsafe { *type_lists.add(i) };
        if current_type_list.is_null() {
            break // Null pointer indicates end of array
        }

        match unsafe { CStr::from_ptr(current_type_list) }.to_str() {
            Ok(rust_str) => type_list_paths.push(PathBuf::from(rust_str)),
            Err(_) => continue,
        };

        i += 1;
    }

    match merge_type_lists(type_list_paths) {
        Ok(combined_type_list) => Ok(Arc::new(combined_type_list)),
        Err(report) => Err(report),
    }
}

#[no_mangle]
pub extern "C" fn op_deserialize(
    input: *const c_char,
    output: *const c_char,
    type_lists: *const *const c_char,
    flags: u32,
    mask: u32,
    shallow: bool,
    manual_compression: bool,
    djb2_only: bool,
    ignore_unknown_types: bool,
) -> bool {
    let default_path = PathBuf::from(HYPHEN);

    if input.is_null() || type_lists.is_null() {
        return false
    }

    // Create the InputsOutputs
    let rust_input = match unsafe { CStr::from_ptr(input) }.to_str() {
        Ok(rust_str) => rust_str.to_owned(),
        Err(_) => return false,
    };

    let rust_output = if output.is_null() {
        default_path
    } else {
        match unsafe { CStr::from_ptr(output) }.to_str() {
            Ok(rust_str) => PathBuf::from(rust_str),
            Err(_) => default_path,
        }
    };

    let io = InputsOutputs {
        input: rust_input,
        output: rust_output,
    };

    // Create the type_list
    let type_list = match get_type_lists_from_c(type_lists) {
        Ok(list) => list,
        Err(_) => return false,
    };

    // Set the options
    let options = serde::SerializerOptions {
        flags: serde::SerializerFlags::from_bits_truncate(flags),
        property_mask: PropertyFlags::from_bits_truncate(mask),
        shallow: shallow,
        manual_compression: manual_compression,
        djb2_only: djb2_only,
        ..Default::default()
    };

    rust_op_deserialize(io, type_list, options, ignore_unknown_types).is_ok()
}

#[no_mangle]
pub extern "C" fn op_guess(
    path: *const c_char,
    type_lists: *const *const c_char,
    flags: u32,
    mask: u32,
    shallow: bool,
    manual_compression: bool,
    djb2_only: bool,
    quiet: bool,
) -> bool {

    // Set the options
    let options = serde::SerializerOptions {
        flags: serde::SerializerFlags::from_bits_truncate(flags),
        property_mask: PropertyFlags::from_bits_truncate(mask),
        shallow: shallow,
        manual_compression: manual_compression,
        djb2_only: djb2_only,
        ..Default::default()
    };

    // Create the type_list
    let type_list = match get_type_lists_from_c(type_lists) {
        Ok(list) => list,
        Err(_) => return false,
    };

    let rust_path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(rust_str) => PathBuf::from(rust_str),
        Err(_) => return false,
    };

    guess::guess(options, type_list, rust_path, quiet).is_ok()
}

#[no_mangle]
pub extern "C" fn poi_deserialize(
    input: *const c_char,
    output: *const c_char,
) -> bool {
    let default_path = PathBuf::from(HYPHEN);

    let rust_input = if input.is_null() {
        return false
    } else {
        match unsafe { CStr::from_ptr(input) }.to_str() {
            Ok(rust_str) => rust_str.to_owned(),
            Err(_) => return false,
        }
    };

    let rust_output = if output.is_null() {
        default_path
    } else {
        match unsafe { CStr::from_ptr(output) }.to_str() {
            Ok(rust_str) => PathBuf::from(rust_str),
            Err(_) => default_path,
        }
    };

    let io = InputsOutputs {
        input: rust_input,
        output: rust_output,
    };

    rust_poi_deserialize(io).is_ok()
}

#[no_mangle]
pub extern "C" fn wad_pack(
    input: *const c_char,
    flags: u8,
    output: *const c_char,
) -> bool {
    let input_path = if input.is_null() {
        return false
    } else {
        match unsafe { CStr::from_ptr(input) }.to_str() {
            Ok(rust_str) => PathBuf::from(rust_str),
            Err(_) => return false,
        }
    };

    let output_path = if output.is_null() {
        None
    } else {
        match unsafe { CStr::from_ptr(output) }.to_str() {
            Ok(rust_str) => Some(PathBuf::from(rust_str)),
            Err(_) => None,
        }
    };

    rust_wad_pack(input_path, flags, output_path).is_ok()
}

#[no_mangle]
pub extern "C" fn wad_unpack(
    input: *const c_char,
    output: *const c_char,
) -> bool {
    let rust_input = if input.is_null() {
        return false
    } else {
        match unsafe { CStr::from_ptr(input) }.to_str() {
            Ok(rust_str) => rust_str.to_owned(),
            Err(_) => return false,
        }
    };

    let default_path = PathBuf::from(HYPHEN);

    let rust_output = if output.is_null() {
        default_path
    } else {
        match unsafe { CStr::from_ptr(output) }.to_str() {
            Ok(rust_str) => PathBuf::from(rust_str),
            Err(_) => default_path,
        }
    };

    let io = InputsOutputs {
        input: rust_input,
        output: rust_output,
    };

    rust_wad_unpack(io).is_ok()
}
