**Theseus Virtual Filesystem**

This virtual filesystem emulates the Unix hierarchical filesystem. It allows kernel information to be organized into directories and supports in-memory file storage. 

The filesystem is implemented as a cyclic tree with strong pointers from the parent nodes to child nodes and weak pointers from child nodes to the parent nodes. 

The filesystem heavily incorporates Traits from Rust, which define a generic interface (in the fs_node crate) that allow multiple different implementations of Files and Directories. 


Currently, there are three implementations of Files:
- InMemoryFile: allows byte arrays to be stored within MappedPages in memory, similar to Linux's RamFS
- VFSFile: a generic concrete implementation of the file to allow for string storage of kernel information
- TaskFile: contains information about the running tasks within the kernel in a human-readable form

There are also two implementations of Directories:
- VFSDirectory: a generic implementation of the directory to allow for fast and lightweight organization
- TaskDirectory: a lazily generated directory which contains TaskFiles for all processes in the kernel, similar to Linux's procfs


**Note** This repository is meant as a code sample. This code was written in contribution to and as a part of the Theseus Operating System of Rice Efficient Computing Group. 
This code will not compile as a standalone application. 