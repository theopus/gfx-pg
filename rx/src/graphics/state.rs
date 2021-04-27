use std::mem::ManuallyDrop;

use hal::{
    adapter::{Gpu, PhysicalDevice},
    Backend,
    queue::{QueueFamily, QueueGroup}, window::Surface,
};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use winit::window::Window;

use crate::hal::adapter::Adapter;

pub struct HalStateV2<B: Backend> {
    pub(crate) device: ManuallyDrop<B::Device>,
    pub(crate) _adapter: hal::adapter::Adapter<B>,
    pub(crate) _surface: ManuallyDrop<B::Surface>,
    pub(crate) _instance: Option<ManuallyDrop<B::Instance>>,
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

    pub fn new(
        _window: &Window,
        instance: Option<<B as Backend>::Instance>,
        surface: B::Surface,
        adapters: Vec<Adapter<B>>,
    ) -> Result<(Self, QueueGroup<B>), &'static str> {
        let adapter = adapters
            .into_iter()
            .find(|a| {
                a.queue_families.iter().any(|qf| {
                    qf.queue_type().supports_graphics() && surface.supports_queue_family(qf)
                })
            })
            .ok_or("Couldn't find a graphical Adapter!")?;
        info!("{:?}", adapter);
        //device stuff
        let (device, queue_group) = {
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
                _instance: match instance {
                    None => None,
                    Some(i) => Some(ManuallyDrop::new(i)),
                },
            },
            queue_group,
        ))
    }
}

impl<B: Backend> Drop for HalStateV2<B> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.device);

//            let surface_ptr = ptr::read(&self._surface);
//            if self._instance.is_some() {
//                self._instance.as_mut()
//                    .destroy_surface(ManuallyDrop::into_inner(surface_ptr));
//                ManuallyDrop::drop(&mut self._instance.unwrap());
//            }
        }
    }
}
