use x86_64::{VirtAddr, structures::paging::Page};

pub fn get_page_containing(addr: VirtAddr) -> Page {
    Page::containing_address(addr)
}
