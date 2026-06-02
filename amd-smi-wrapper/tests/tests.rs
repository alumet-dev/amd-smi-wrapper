use amd_smi_wrapper::{
    AmdInitFlags, AmdInterface, AmdSmi,
    handles::{ProcessorHandle, SocketHandle},
};

fn run_gpu_tests() -> bool {
    if std::env::var_os("TEST_GPU").is_some() {
        true
    } else {
        println!("test skipped because TEST_GPU is not set");
        false
    }
}

#[test]
fn list_devices() {
    if !run_gpu_tests() {
        return;
    }

    let amdsmi = AmdSmi::init(AmdInitFlags::AMDSMI_INIT_AMD_GPUS).unwrap();
    for socket in amdsmi.socket_handles().unwrap() {
        for proc in socket.processor_handles().unwrap() {
            let uuid = proc.device_uuid().unwrap();
            println!("found gpu: {uuid}");
        }
    }

    // automatic drop of amdsmi
}
