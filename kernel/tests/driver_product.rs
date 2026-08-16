use aweos_kernel::drivers::{AdapterState,AndroidDriverAdapter,AndroidLayer,CoreError,DeviceContract,DeviceId,DmaPolicy,DriverAbi,DriverAdapter,DriverIdentity,DriverOs,HardwareAbstraction,HardwareInfo,InterruptMode,MmioRegion,WindowsDriverAdapter,WindowsLayer};

#[derive(Clone,Copy)]
struct TestAdapter{os:DriverOs,abi:DriverAbi,api:u16,vendor:u16,device:u16,signed:bool}
impl DriverAdapter for TestAdapter{
 fn identity(&self)->DriverIdentity{DriverIdentity{os:self.os,abi:self.abi,api_version:self.api,vendor:self.vendor,device:self.device,signed:self.signed}}
 fn probe(&mut self,_:&HardwareInfo)->Result<(),CoreError>{Ok(())}
 fn start(&mut self,_:&HardwareInfo)->Result<(),CoreError>{Ok(())}
 fn stop(&mut self,_:&HardwareInfo)->Result<(),CoreError>{Ok(())}
 fn remove(&mut self,_:&HardwareInfo)->Result<(),CoreError>{Ok(())}
}
impl WindowsDriverAdapter for TestAdapter{fn windows_api_version(&self)->u32{self.api as u32}fn windows_driver_name(&self)->&'static str{"awe-wdm"}}
impl AndroidDriverAdapter for TestAdapter{fn android_hal_version(&self)->u32{self.api as u32}fn android_interface_name(&self)->&'static str{"vendor/awe-hal"}}

#[derive(Default)]
struct TestHal{value:u32,dma_bytes:u64,irq_acks:u32}
impl HardwareAbstraction for TestHal{
 fn mmio_read32(&self,_:&HardwareInfo,_:u64)->Result<u32,CoreError>{Ok(self.value)}
 fn mmio_write32(&mut self,_:&HardwareInfo,_:u64,value:u32)->Result<(),CoreError>{self.value=value;Ok(())}
 fn irq_ack(&mut self,_:&HardwareInfo)->Result<(),CoreError>{self.irq_acks+=1;Ok(())}
 fn dma_submit(&mut self,_:&HardwareInfo,bytes:u64)->Result<(),CoreError>{self.dma_bytes=bytes;Ok(())}
}

fn hardware()->HardwareInfo{HardwareInfo{id:DeviceId{vendor:0x1234,device:0x5678,class:1,revision:1},mmio_base:0x1000,mmio_length:0x100,irq:5,dma_bits:64}}
fn contract()->DeviceContract<1>{DeviceContract{vendor:0x1234,device:0x5678,class_code:1,mmio:[Some(MmioRegion{base:0x1000,length:0x100})],interrupt:InterruptMode::Msi,dma:DmaPolicy{max_bytes:4096,address_bits:64,coherent:true}}}

#[test]
fn windows_product_io_path_is_end_to_end(){
 let mut layer=WindowsLayer::new(TestAdapter{os:DriverOs::Windows,abi:DriverAbi::WindowsCompat,api:7,vendor:0x1234,device:0x5678,signed:true});
 let mut hal=TestHal::default();
 let value=layer.io_cycle_contract(&hardware(),&contract(),&mut hal,0x10,0xa5a55a5a,512,32).unwrap();
 assert_eq!(value,0xa5a55a5a);assert_eq!(hal.dma_bytes,512);assert_eq!(layer.slot.state,AdapterState::Running);
 layer.stop(&hardware()).unwrap();layer.remove(&hardware()).unwrap();assert_eq!(layer.slot.state,AdapterState::Removed);
}

#[test]
fn android_product_io_path_is_end_to_end(){
 let mut layer=AndroidLayer::new(TestAdapter{os:DriverOs::Android,abi:DriverAbi::AndroidHal,api:7,vendor:0x1234,device:0x5678,signed:true});
 let mut hal=TestHal::default();
 let value=layer.io_cycle_contract(&hardware(),&contract(),&mut hal,0x20,0x55aa33cc,256,32).unwrap();
 assert_eq!(value,0x55aa33cc);assert_eq!(hal.dma_bytes,256);assert_eq!(layer.slot.state,AdapterState::Running);
 layer.stop(&hardware()).unwrap();layer.remove(&hardware()).unwrap();assert_eq!(layer.slot.state,AdapterState::Removed);
}

#[test]
fn unsigned_windows_driver_is_blocked_before_probe(){
 let layer=WindowsLayer::new(TestAdapter{os:DriverOs::Windows,abi:DriverAbi::WindowsCompat,api:7,vendor:0x1234,device:0x5678,signed:false});
 assert_eq!(layer.validate(&hardware()),Err(CoreError::PolicyDenied));
}

#[test]
fn unsigned_android_driver_is_blocked_before_probe(){
 let layer=AndroidLayer::new(TestAdapter{os:DriverOs::Android,abi:DriverAbi::AndroidHal,api:7,vendor:0x1234,device:0x5678,signed:false});
 assert_eq!(layer.validate(&hardware()),Err(CoreError::PolicyDenied));
}

#[test]
fn contract_blocks_dma_over_limit(){
 let mut layer=WindowsLayer::new(TestAdapter{os:DriverOs::Windows,abi:DriverAbi::WindowsCompat,api:7,vendor:0x1234,device:0x5678,signed:true});
 let mut hal=TestHal::default();
 assert_eq!(layer.io_cycle_contract(&hardware(),&contract(),&mut hal,0x10,1,4097,32),Err(CoreError::DmaDenied));
 assert_eq!(layer.slot.state,AdapterState::Running);
}

#[test]
fn contract_blocks_mmio_outside_device_window(){
 let mut layer=AndroidLayer::new(TestAdapter{os:DriverOs::Android,abi:DriverAbi::AndroidHal,api:7,vendor:0x1234,device:0x5678,signed:true});
 let mut hal=TestHal::default();
 assert_eq!(layer.io_cycle_contract(&hardware(),&contract(),&mut hal,0x100,1,16,32),Err(CoreError::MmioDenied));
 assert_eq!(layer.slot.state,AdapterState::New);
}
