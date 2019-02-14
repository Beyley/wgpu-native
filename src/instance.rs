use crate::hub::HUB;
use crate::{
    WeaklyStored,
    AdapterHandle, DeviceHandle, SurfaceHandle,
    AdapterId, InstanceId,
};
#[cfg(feature = "local")]
use crate::{DeviceId, SurfaceId};

use hal::{self, Instance as _Instance, PhysicalDevice as _PhysicalDevice};


#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum PowerPreference {
    Default = 0,
    LowPower = 1,
    HighPerformance = 2,
}

#[repr(C)]
pub struct AdapterDescriptor {
    pub power_preference: PowerPreference,
}

#[repr(C)]
pub struct Extensions {
    pub anisotropic_filtering: bool,
}

#[repr(C)]
pub struct DeviceDescriptor {
    pub extensions: Extensions,
}

pub fn wgpu_create_instance_impl() -> ::back::Instance {
    ::back::Instance::create("wgpu", 1)
}

#[no_mangle]
#[cfg(feature = "local")]
pub extern "C" fn wgpu_create_instance() -> InstanceId {
    let inst = wgpu_create_instance_impl();
    HUB.instances.register(inst)
}

#[cfg(feature = "local")]
#[no_mangle]
pub extern "C" fn wgpu_instance_create_surface_from_winit(
    instance_id: InstanceId,
    window: &winit::Window,
) -> SurfaceId {
    let raw = HUB.instances
        .read()
        .get(instance_id)
        .create_surface(window);
    let surface = SurfaceHandle {
        raw,
    };

    HUB.surfaces.register(surface)
}

#[allow(unused_variables)]
pub fn wgpu_instance_create_surface_from_xlib_impl(
    instance_id: InstanceId,
    display: *mut *const std::ffi::c_void,
    window: u64,
) -> SurfaceHandle {
    #[cfg(not(all(unix, feature = "gfx-backend-vulkan")))]
    unimplemented!();

    #[cfg(all(unix, feature = "gfx-backend-vulkan"))]
    SurfaceHandle {
        raw: HUB.instances
            .read()
            .get(instance_id)
            .create_surface_from_xlib(display, window),
    }
}

#[cfg(feature = "local")]
#[no_mangle]
pub extern "C" fn wgpu_instance_create_surface_from_xlib(
    instance_id: InstanceId,
    display: *mut *const std::ffi::c_void,
    window: u64,
) -> SurfaceId {
    let surface = wgpu_instance_create_surface_from_xlib_impl(instance_id, display, window);
    HUB.surfaces.register(surface)
}

#[allow(unused_variables)]
pub fn wgpu_instance_create_surface_from_macos_layer_impl(
    instance_id: InstanceId,
    layer: *mut std::ffi::c_void,
) -> SurfaceHandle {
    #[cfg(not(feature = "gfx-backend-metal"))]
    unimplemented!();

    #[cfg(feature = "gfx-backend-metal")]
    SurfaceHandle {
        raw: HUB.instances
            .read()
            .get(instance_id)
            .create_surface_from_layer(layer as *mut _),
    }
}

#[cfg(feature = "local")]
#[no_mangle]
pub extern "C" fn wgpu_instance_create_surface_from_macos_layer(
    instance_id: InstanceId,
    layer: *mut std::ffi::c_void,
) -> SurfaceId {
    let surface = wgpu_instance_create_surface_from_macos_layer_impl(instance_id, layer);
    HUB.surfaces.register(surface)
}

#[allow(unused_variables)]
pub fn wgpu_instance_create_surface_from_windows_hwnd_impl(
    instance_id: InstanceId,
    hinstance: *mut std::ffi::c_void,
    hwnd: *mut std::ffi::c_void,
) -> SurfaceHandle {
    #[cfg(not(target_os = "windows"))]
    let raw = unimplemented!();

    #[cfg(any(feature = "gfx-backend-dx11", feature = "gfx-backend-dx12"))]
    let raw = HUB.instances
        .read()
        .get(instance_id)
        .create_surface_from_hwnd(hwnd);

    #[cfg(all(target_os = "windows", feature = "gfx-backend-vulkan"))]
    let raw = HUB.instances
        .read()
        .get(instance_id)
        .create_surface_from_hwnd(hinstance, hwnd);

    #[cfg_attr(not(target_os = "windows"), allow(unreachable_code))]
    SurfaceHandle {
        raw,
    }
}

#[cfg(feature = "local")]
#[no_mangle]
pub extern "C" fn wgpu_instance_create_surface_from_windows_hwnd(
    instance_id: InstanceId,
    hinstance: *mut std::ffi::c_void,
    hwnd: *mut std::ffi::c_void,
) -> SurfaceId {
    let surface = wgpu_instance_create_surface_from_windows_hwnd_impl(instance_id, hinstance, hwnd);
    HUB.surfaces.register(surface)
}

pub fn wgpu_instance_get_adapter_impl(
    instance_id: InstanceId,
    desc: &AdapterDescriptor,
) -> AdapterHandle {
    let instance_guard = HUB.instances.read();
    let instance = instance_guard.get(instance_id);
    let (mut low, mut high, mut other) = (None, None, None);
    for adapter in instance.enumerate_adapters() {
        match adapter.info.device_type {
            hal::adapter::DeviceType::IntegratedGpu => low = Some(adapter),
            hal::adapter::DeviceType::DiscreteGpu => high = Some(adapter),
            _ => other = Some(adapter),
        }
    }

    let some = match desc.power_preference {
        PowerPreference::LowPower => low.or(high),
        PowerPreference::HighPerformance | PowerPreference::Default => high.or(low),
    };
    some.or(other).unwrap()
}

#[cfg(feature = "local")]
#[no_mangle]
pub extern "C" fn wgpu_instance_get_adapter(
    instance_id: InstanceId,
    desc: &AdapterDescriptor,
) -> AdapterId {
    let adapter = wgpu_instance_get_adapter_impl(instance_id, desc);
    HUB.adapters.register(adapter)
}

pub fn wgpu_adapter_create_device_impl(
    adapter_id: AdapterId,
    _desc: &DeviceDescriptor,
) -> DeviceHandle {
    let mut adapter_guard = HUB.adapters.write();
    let adapter = adapter_guard.get_mut(adapter_id);
    let (raw, queue_group) = adapter.open_with::<_, hal::General>(1, |_qf| true).unwrap();
    let mem_props = adapter.physical_device.memory_properties();
    DeviceHandle::new(raw, WeaklyStored(adapter_id), queue_group, mem_props)
}

#[cfg(feature = "local")]
#[no_mangle]
pub extern "C" fn wgpu_adapter_create_device(
    adapter_id: AdapterId,
    desc: &DeviceDescriptor,
) -> DeviceId {
    let device = wgpu_adapter_create_device_impl(adapter_id, desc);
    HUB.devices.register(device)
}
