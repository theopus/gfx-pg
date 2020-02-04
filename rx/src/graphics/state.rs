use core::ptr;
use std::mem::ManuallyDrop;

use arrayvec::ArrayVec;
use hal::{
    adapter::{Gpu, PhysicalDevice},
    queue::{QueueFamily, QueueGroup},
    window::Surface,
    Backend, Instance,
};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::window::Window;

use crate::graphics::api::DepthImage;

pub struct HalStateV2<B: Backend> {
    pub(crate) device: ManuallyDrop<B::Device>,
    pub(crate) _adapter: hal::adapter::Adapter<B>,
    pub(crate) _surface: ManuallyDrop<B::Surface>,
    pub(crate) _instance: ManuallyDrop<B::Instance>,
}

impl<B: Backend> HalStateV2<B> {
    pub fn device_ref(&self) -> &B::Device {
        &self.device
    }
    pub fn adapter_ref(&self) -> &hal::adapter::Adapter<B> {
        &self._adapter
    }
    pub fn surface_ref(&mut self) -> &mut B::Surface {
        &mut self._surface
    }
    pub fn instance_ref(&self) -> &B::Instance {
        &self._instance
    }

    pub fn new(
        window: &Window,
        instance: <B as Backend>::Instance,
        mut surface: B::Surface,
    ) -> Result<(Self, QueueGroup<B>), &'static str> {
        let adapter = instance
            .enumerate_adapters()
            .into_iter()
            .find(|a| {
                a.queue_families.iter().any(|qf| {
                    qf.queue_type().supports_graphics() && surface.supports_queue_family(qf)
                })
            })
            .ok_or("Couldn't find a graphical Adapter!")?;
        info!("{:?}", adapter);
        //device stuff
        let (mut device, mut queue_group) = {
            let queue_family = adapter
                .queue_families
                .iter()
                .find(|qf| qf.queue_type().supports_graphics() && surface.supports_queue_family(qf))
                .ok_or("Couldn't find a QueueFamily with graphics!")?;

            let Gpu {
                device,
                mut queue_groups,
            } = unsafe {
                adapter
                    .physical_device
                    .open(&[(&queue_family, &[1.0; 1])], hal::Features::empty())
                    .map_err(|_| "Couldn't open the PhysicalDevice!")?
            };

            let queue_group = queue_groups.pop().unwrap();
            let _ = if queue_group.queues.len() > 0 {
                Ok(())
            } else {
                Err("The QueueGroup did not have any CommandQueues available!")
            }?;
            (device, queue_group)
        };
        Ok((
            HalStateV2 {
                device: ManuallyDrop::new(device),
                _adapter: adapter,
                _surface: ManuallyDrop::new(surface),
                _instance: ManuallyDrop::new(instance),
            },
            queue_group,
        ))
    }
}

impl<B: Backend> Drop for HalStateV2<B> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.device);

            let surface_ptr = ptr::read(&self._surface);
            self._instance
                .destroy_surface(ManuallyDrop::into_inner(surface_ptr));

            ManuallyDrop::drop(&mut self._instance);
        }
    }
}
