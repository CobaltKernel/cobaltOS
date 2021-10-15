use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{PhysAddr, structures::paging::{FrameAllocator, PhysFrame, Size4KiB}};

pub const PAGE_SIZE: usize = 4096;

pub struct BootFrameAllocator {
    memory_map: &'static MemoryMap,
    next_frame: usize,
}

impl BootFrameAllocator {
    pub fn new(memory_map: &'static MemoryMap) -> Self {
        Self {memory_map, next_frame: 0}
    }

    pub fn free_frame_count(&self) -> usize {
        self.usable_frames().count() - self.next_frame_index()
    }

    pub fn total_frame_count(&self) -> usize {
        self.total_frames().count()
    }

    pub fn next_frame_index(&self) -> usize {
        self.next_frame
    }

    pub fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let valid_regions = regions.filter(
            |region| region.region_type == MemoryRegionType::Usable
        );

        let address_ranges =  valid_regions.map(
            |region| region.range.start_addr()..region.range.end_addr()
        );

        let addresses = address_ranges.flat_map(
            |range| range.step_by(PAGE_SIZE)
        );

        let frames = addresses.map(
            |addr| PhysFrame::containing_address(PhysAddr::new(addr))
        );



        frames
    }

    pub fn total_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let valid_regions = regions;

        let address_ranges =  valid_regions.map(
            |region| region.range.start_addr()..region.range.end_addr()
        );

        let addresses = address_ranges.flat_map(
            |range| range.step_by(PAGE_SIZE)
        );

        let frames = addresses.map(
            |addr| PhysFrame::containing_address(PhysAddr::new(addr))
        );



        frames
    }

    pub fn get_mem_size(&self) -> u64 {
        let regions = self.total_frames();
        let mut sum: u64 = 0;
        for r in regions {
            sum += r.size() as u64
        }
        sum
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next_frame);
        self.next_frame += 1;
        frame
    }
}



/// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}