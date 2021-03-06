#![no_std]
#![feature(alloc)]

/// This crate contains an implementation of an in-memory filesystem backed by MappedPages from the memory crate
/// Currently, the read and write operations of the RamFile follows the interface of the std::io read/write operations of the Rust standard library

#[macro_use] extern crate log;
extern crate alloc;
extern crate spin;
extern crate fs_node;
extern crate memory;
extern crate irq_safety;

use alloc::vec::Vec;
use core::ops::DerefMut;
use alloc::string::String;
use fs_node::{StrongAnyDirRef, WeakDirRef, File, FileDirectory};
use memory::{MappedPages, FRAME_ALLOCATOR};
use memory::EntryFlags;


pub struct InMemoryFile {
    /// The name of the file
    name: String,
    // The size of the file in bytes
    size: usize,
    /// The string contents as a file: this primitive can be changed into a more complex struct as files become more complex
    contents: MappedPages,
    /// A weak reference to the parent directory
    parent: WeakDirRef,
}

impl InMemoryFile {
    /// Combines file creation and file write into one operation
    pub fn new(name: String, contents: &mut [u8], parent: WeakDirRef) -> Result<InMemoryFile, &'static str> {
        // Obtain the active kernel page table
        let kernel_mmi_ref = memory::get_kernel_mmi_ref().ok_or("create_contiguous_mapping(): KERNEL_MMI was not yet initialized!")?;
        if let memory::PageTable::Active(ref mut active_table) = kernel_mmi_ref.lock().page_table {
            let mut allocator = try!(FRAME_ALLOCATOR.try().ok_or("Couldn't get Frame Allocator")).lock(); 
            // Allocate and map the least number of pages we need to store the information contained in the buffer
            let pages = memory::allocate_pages_by_bytes(contents.len()).ok_or("could not allocate pages")?;
            let mut mapped_pages = active_table.map_allocated_pages(pages,  EntryFlags::WRITABLE, allocator.deref_mut())?;            

            { // scoped this so that the mutable borrow on mapped_pages ends as soon as possible
                // Gets a mutuable reference to the byte portion of the newly mapped pages
                let mut dest_slice = mapped_pages.as_slice_mut::<u8>(0, contents.len())?;
                dest_slice.copy_from_slice(contents); // writes the desired contents into the correct area in the mapped page
            }
            // create and return the newly create InMemoryFile
            return Ok(InMemoryFile {
                name: name, 
                size: contents.len(),
                contents: mapped_pages,
                parent: parent
            })
        }
        return Err("could not get active table");
    }
}

impl File for InMemoryFile {
    /// To read the contents of a file, query the size of the file 
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, &'static str> {
        let num_bytes_read = self.size; // this is the number of bytes of actual information stored in the MappedPage
        // Copies the information from the MappedPage (or at least the section containing the relevant information) to the read-buffer
        buf.copy_from_slice(self.contents.as_slice_mut(0, num_bytes_read)?); 
        return Ok(num_bytes_read);
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, &'static str> {
        // Get the active table
        let kernel_mmi_ref = memory::get_kernel_mmi_ref().ok_or("create_contiguous_mapping(): KERNEL_MMI was not yet initialized!")?;
        if let memory::PageTable::Active(ref mut active_table) = kernel_mmi_ref.lock().page_table {
            let mut allocator = try!(FRAME_ALLOCATOR.try().ok_or("Couldn't get Frame Allocator")).lock();
            // Allocate and map the least number of pages we need to store the information contained in the buffer
            let pages = memory::allocate_pages_by_bytes(buf.len()).ok_or("could not allocate pages")?;
            let mapped_pages = active_table.map_allocated_pages(pages,  EntryFlags::WRITABLE, allocator.deref_mut())?;
            { // scoped this so that the mutable borrow on mapped_pages ends as soon as possible
                // Gets a mutuable reference to the byte portion of the newly mapped pages
                let mut dest_slice = mapped_pages.as_slice_mut::<u8>(0, contents.len())?;
                dest_slice.copy_from_slice(contents); // writes the desired contents into the correct area in the mapped page
            }
            // Overwrites the previous file's contents (MappedPage) with the new buffer's contents (in the newly mapped page)
            self.contents = mapped_pages;
            return Ok(self.contents.size_in_bytes())
        }
        return Err("could not get active table");

    }

    fn seek(&self) { unimplemented!(); }
    fn delete(&self) { unimplemented!(); }
    fn size(&self) -> usize {
        return self.size;
    }
}

impl FileDirectory for InMemoryFile {
    fn get_name(&self) -> String {
        self.name.clone()
    }
    
    /// Returns a pointer to the parent if it exists
    fn get_parent_dir(&self) -> Result<StrongAnyDirRef, &'static str> {
        return match self.parent.upgrade() {
            Some(parent) => Ok(parent),
            None => Err("could not upgrade parent")
        }
    }

    fn set_parent(&mut self, parent_pointer: WeakDirRef) {
        self.parent = parent_pointer
    }
}