use ash::version::{InstanceV1_0, DeviceV1_0};

impl Drop for super::Engine {
    fn drop(&mut self) {
        unsafe {
            if let Some((report, callback)) = self.debug_report_callback.take() {
                report.destroy_debug_report_callback(callback, None);
            }
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
