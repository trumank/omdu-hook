use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use element_ptr::element_ptr;
use simple_log::{error, info, LogConfigBuilder};
use windows::Win32::{
    Foundation::HMODULE,
    System::{
        LibraryLoader::GetModuleHandleW,
        SystemServices::*,
        Threading::{GetCurrentThread, QueueUserAPC},
    },
};

// x3daudio1_7.dll
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn X3DAudioCalculate() {}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn X3DAudioInitialize() {}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DllMain(dll_module: HMODULE, call_reason: u32, _: *mut ()) -> bool {
    unsafe {
        match call_reason {
            DLL_PROCESS_ATTACH => {
                QueueUserAPC(Some(init), GetCurrentThread(), 0);
            }
            DLL_PROCESS_DETACH => (),
            _ => (),
        }

        true
    }
}

unsafe extern "system" fn init(_: usize) {
    if let Ok(_bin_dir) = setup() {
        info!("dll_hook loaded",);

        if let Err(e) = patch() {
            error!("{e:#}");
        } else {
            info!("patches and hooks complete");
        }
    }
}

fn setup() -> Result<PathBuf> {
    let exe_path = std::env::current_exe()?;
    let bin_dir = exe_path.parent().context("could not find exe parent dir")?;
    let config = LogConfigBuilder::builder()
        .path(bin_dir.join("omdu_hook.txt").to_str().unwrap())
        .time_format("%Y-%m-%d %H:%M:%S.%f")
        .level("debug")
        .output_file()
        .size(u64::MAX)
        .build();
    simple_log::new(config).map_err(|e| anyhow!("{e}"))?;
    Ok(bin_dir.to_path_buf())
}

#[derive(Debug)]
#[repr(C)]
struct TArray<T> {
    data: *const T,
    num: i32,
    max: i32,
}
type FString = TArray<u16>;
impl<T> TArray<T> {
    fn new() -> Self {
        Self {
            data: std::ptr::null(),
            num: 0,
            max: 0,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
struct UEngine {
    vtable: *const UEngineVTable,
    padding1: [u8; 0x6d8],
    client: *const UClient,
}
#[derive(Debug)]
#[repr(C)]
struct UEngineVTable {
    padding2: [u8; 0x310],
    spawn_server_actors: unsafe extern "system" fn(this: *mut UEngine),
    construct_net_driver: unsafe extern "system" fn(this: *mut UEngine) -> *mut UNetDriver,
}

#[derive(Debug)]
#[repr(C)]
struct UClient {}

#[derive(Debug)]
#[repr(C)]
struct UWorld {
    padding1: [u8; 0x60],
    vtable_network_notify: *const FNetworkNotifyVTable,
    padding2: [u8; 0xb8],
    net_driver: *const UNetDriver,
}
#[derive(Debug)]
#[repr(C)]
struct AWorldInfo {
    #[cfg(feature = "manifest-4932913164832566208")]
    padding1: [u8; 0x638],
    #[cfg(feature = "manifest-808827202674972462")]
    padding1: [u8; 0x620],
    net_mode: u8,
    padding2: [u8; 0xbc],
    next_switch_countdown: f32,
}

#[derive(Debug)]
#[repr(C)]
struct UNetDriver {
    vtable: *const UNetDriverVTable,
    padding1: [u8; 0x60],
    vtable_net_object_notify: *const FNetworkNotifyVTable,
    padding2: [u8; 0x20],
    master_map: *const UPackageMap,
    padding3: [u8; 0x20],
    server_travel_pause: f32,
}
#[derive(Debug)]
#[repr(C)]
struct UNetDriverVTable {
    padding: [u8; 0x298],
    init_listen: unsafe extern "system" fn(
        this: *mut UNetDriver,
        notify: *const FNetworkNotify,
        url: *const FURL,
        error: *mut FString,
    ) -> bool,
}

#[derive(Debug)]
#[repr(C)]
struct UPackageMap {
    vtable: *const UPackageMapVTable,
}
#[derive(Debug)]
#[repr(C)]
struct UPackageMapVTable {
    padding: [u8; 0x290],
    add_net_packages: unsafe extern "system" fn(this: *mut UPackageMap) -> bool,
}

#[derive(Debug)]
#[repr(C)]
struct FNetworkNotify {
    vtable: *const FNetworkNotifyVTable,
}
#[derive(Debug)]
#[repr(C)]
struct FNetworkNotifyVTable {}

#[derive(Debug)]
#[repr(C)]
struct FNetObjectNotify {
    vtable: *const FNetObjectNotifyVTable,
}
#[derive(Debug)]
#[repr(C)]
struct FNetObjectNotifyVTable {}

#[derive(Debug)]
#[repr(C)]
struct FURL {}

//#[cfg(test)]
mod test {
    use super::*;
    const _: [u8; 0x60] = [0; std::mem::offset_of!(UWorld, vtable_network_notify)];
    const _: [u8; 0x120] = [0; std::mem::offset_of!(UWorld, net_driver)];

    #[cfg(feature = "manifest-4932913164832566208")]
    const _: [u8; 0x638] = [0; std::mem::offset_of!(AWorldInfo, net_mode)];
    #[cfg(feature = "manifest-4932913164832566208")]
    const _: [u8; 0x6f8] = [0; std::mem::offset_of!(AWorldInfo, next_switch_countdown)];

    #[cfg(feature = "manifest-808827202674972462")]
    const _: [u8; 0x620] = [0; std::mem::offset_of!(AWorldInfo, net_mode)];
    #[cfg(feature = "manifest-808827202674972462")]
    const _: [u8; 0x6e0] = [0; std::mem::offset_of!(AWorldInfo, next_switch_countdown)];

    const _: [u8; 0x6e0] = [0; std::mem::offset_of!(UEngine, client)];
    const _: [u8; 0x310] = [0; std::mem::offset_of!(UEngineVTable, spawn_server_actors)];
    const _: [u8; 0x318] = [0; std::mem::offset_of!(UEngineVTable, construct_net_driver)];
    const _: [u8; 0x298] = [0; std::mem::offset_of!(UNetDriverVTable, init_listen)];
    const _: [u8; 0x68] = [0; std::mem::offset_of!(UNetDriver, vtable_net_object_notify)];
    const _: [u8; 0x90] = [0; std::mem::offset_of!(UNetDriver, master_map)];
    const _: [u8; 0xb8] = [0; std::mem::offset_of!(UNetDriver, server_travel_pause)];
    const _: [u8; 0x290] = [0; std::mem::offset_of!(UPackageMapVTable, add_net_packages)];
}

retour::static_detour! {
    static HookUWorldListen: unsafe extern "system" fn(*mut UWorld, *const FURL, *mut FString) -> bool;
    static HookIsValidPlayerGUID: unsafe extern "system" fn(*const (), i32, *const ()) -> bool;
}

type FnAddItem =
    unsafe extern "system" fn(*mut TArray<*const FNetObjectNotify>, *const *const FNetObjectNotify);
type FnUWorldGetWorldInfo = unsafe extern "system" fn(
    this: *mut UWorld,
    check_streaming_persistence: bool,
) -> *mut AWorldInfo;

#[derive(Debug)]
struct Addresses {
    is_player_guid_valid: usize,
    u_world_listen: usize,
    g_engine: usize,
    g_use_seek_free_package_map: usize,
    add_item: usize,
    u_package_net_object_notfies: usize,
    u_world_get_world_info: usize,
}

unsafe fn patch() -> Result<()> {
    #[cfg(feature = "manifest-4932913164832566208")]
    let addresses = Addresses {
        is_player_guid_valid: 0xf216c0,
        u_world_listen: 0x7f6540,
        g_engine: 0x26761e0,
        g_use_seek_free_package_map: 0x2539738,
        add_item: 0x677860,
        u_package_net_object_notfies: 0x2559850,
        u_world_get_world_info: 0x7f8cf0,
    };

    #[cfg(feature = "manifest-808827202674972462")]
    let addresses = Addresses {
        is_player_guid_valid: 0xdd21d0,
        u_world_listen: 0x7f8340,
        g_engine: 0x238a200,
        g_use_seek_free_package_map: 0x224e580,
        add_item: 0xd14290,
        u_package_net_object_notfies: 0x226d800,
        u_world_get_world_info: 0x7fab90,
    };

    let base_address = GetModuleHandleW(None).unwrap().0 as usize;
    info!("base_address = {:x}", base_address);

    HookIsValidPlayerGUID.initialize(
        std::mem::transmute(base_address + addresses.is_player_guid_valid),
        move |_, _, _| true,
    )?;
    HookIsValidPlayerGUID.enable()?;

    HookUWorldListen.initialize(
        std::mem::transmute(base_address + addresses.u_world_listen),
        move |world, url, error| {
            info!("uworld listen hooked");

            let net_driver = element_ptr!(world => .net_driver.*);

            info!("net_driver = {net_driver:?}");

            let gengine = *std::mem::transmute::<usize, *const *mut UEngine>(
                base_address + addresses.g_engine,
            );
            info!("gengine {gengine:?}");
            let vtable = element_ptr!(gengine => .vtable.* as UEngineVTable);
            info!("vtable {vtable:?}");
            let net_driver = (element_ptr!(vtable => .construct_net_driver.*))(gengine);

            if !net_driver.is_null() {
                info!("constructed net_driver = {net_driver:?}");
                *element_ptr!(world => .net_driver) = net_driver;

                let notify = element_ptr!(world => .vtable_network_notify as FNetworkNotify);
                info!("notify = {notify:?}");

                let mut error = FString::new();
                let f = element_ptr!(net_driver => .vtable.*.init_listen.*);
                info!("init function = {f:?}");
                let code = f(net_driver, notify, url, &mut error);
                info!("listen ret code = {code}");

                let g_use_seek_free_package_map = *std::mem::transmute::<usize, *const u32>(
                    base_address + addresses.g_use_seek_free_package_map,
                );
                info!("GUseSeekFreePackageMap = {g_use_seek_free_package_map}");

                if g_use_seek_free_package_map == 0 {
                    let master_map = element_ptr!(net_driver => .master_map.*);
                    info!("master map = {master_map:?}");
                    info!(
                        "master map vtable = {:?}",
                        element_ptr!(master_map => .vtable.*)
                    );
                    element_ptr!(master_map => .vtable.*.add_net_packages.*)(master_map as *mut _);
                }
                info!("adding net object notify");
                let add_item =
                    std::mem::transmute::<usize, FnAddItem>(base_address + addresses.add_item);
                let notify =
                    element_ptr!(net_driver => .vtable_net_object_notify as FNetObjectNotify);
                info!("NetDriver object notify = {notify:?}");
                let object_notifies =
                    std::mem::transmute::<usize, *mut TArray<*const FNetObjectNotify>>(
                        base_address + addresses.u_package_net_object_notfies,
                    );
                add_item(object_notifies, &(notify as *const _));

                (element_ptr!(vtable => .spawn_server_actors.*))(gengine);

                let get_world_info = std::mem::transmute::<usize, FnUWorldGetWorldInfo>(
                    base_address + addresses.u_world_get_world_info,
                );
                let world_info = get_world_info(world, false);
                info!(
                    "world_info = {world_info:?} net_mode = {}",
                    element_ptr!(world_info => .net_mode.*)
                );
                let net_mode = if element_ptr!(gengine => .client.*).is_null() {
                    1
                } else {
                    2
                };
                *element_ptr!(world_info => .net_mode) = net_mode;
                //*element_ptr!(world_info => .next_switch_countdown) =
                //    *element_ptr!(net_driver => .server_travel_pause);
                info!("net_mode is now {net_mode}");
            } else {
                info!("failed to construct net driver");
            }

            HookUWorldListen.call(world, url, error);
            info!("returning");
            return true;
        },
    )?;
    HookUWorldListen.enable()?;
    Ok(())
}
