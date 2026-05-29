use amd_smi_wrapper_sys::load::{MultiVersionLib, VersionedLib};
use amd_smi_wrapper_sys::versions::stable::AMDSMI_STATUS_SUCCESS;
use amd_smi_wrapper_sys::{load, versions};
use std::mem::MaybeUninit;

fn run_gpu_tests() -> bool {
    if std::env::var_os("TEST_GPU").is_some() {
        true
    } else {
        println!("test skipped because TEST_GPU is not set");
        false
    }
}

#[test]
fn load_multiversion() {
    if !run_gpu_tests() {
        return;
    }

    let lib = load("libamd_smi.so", true).expect("load failure");
    println!("AMD SMI version: {:?}", lib.version);

    // Call the version-specific amdsmi_get_lib_version(*amdsmi_version_t) and check that we get coherent data.
    unsafe fn get_version<T>(lib: &MultiVersionLib) -> T {
        let mut version = MaybeUninit::<T>::zeroed();
        let res = unsafe {
            lib.lib_stable
                .amdsmi_get_lib_version(version.as_mut_ptr() as _)
        };
        assert_eq!(res, AMDSMI_STATUS_SUCCESS, "get_lib_version failed");
        unsafe { version.assume_init() }
    }

    match lib.lib_versioned {
        VersionedLib::V6_3_0(_) => {
            use versions::v6_3_0::amdsmi_version_t;
            let version = unsafe { get_version::<amdsmi_version_t>(&lib) };
            assert_eq!(
                lib.version.smi_version,
                [version.year, version.major, version.minor, version.release]
            );
        }
        VersionedLib::V6_4_0(_) => {
            use versions::v6_4_0::amdsmi_version_t;
            let version = unsafe { get_version::<amdsmi_version_t>(&lib) };
            assert_eq!(
                lib.version.smi_version,
                [version.year, version.major, version.minor, version.release]
            );
        }
        VersionedLib::V6_4_2(_) => {
            use versions::v6_4_2::amdsmi_version_t;
            let version = unsafe { get_version::<amdsmi_version_t>(&lib) };
            assert_eq!(
                lib.version.smi_version,
                [version.year, version.major, version.minor, version.release]
            );
        }
        VersionedLib::V7_0_0(_) => {
            use versions::v7_0_0::amdsmi_version_t;
            let version = unsafe { get_version::<amdsmi_version_t>(&lib) };
            assert_eq!(
                lib.version.smi_version,
                [version.major, version.minor, version.release, 0]
            );
        }
        VersionedLib::V7_2_0(_) => {
            use versions::v7_2_0::amdsmi_version_t;
            let version = unsafe { get_version::<amdsmi_version_t>(&lib) };
            assert_eq!(
                lib.version.smi_version,
                [version.major, version.minor, version.release, 0]
            );
        }
    }
}
