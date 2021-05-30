# Project Final report
11811719 魏源泰 11811715 赵云天

[TOC]

[**The github reop link correspond to the project**](https://github.com/BigBillZhao/xv6-rust)

## Result analysis
- [x] Implement the xv6 rust system to let it able to boot and print information to the console

- [x] use the same assembly code as the C version did, like entry.S

- [x] rewrite start.c and other code in Rust to operate the registers to setup uart and machine mode

- [x] rewrite the string format function in Rust and implement the println! 

- memory management

  - - [x] realize the physical memory allocator(kalloc)

  - - [x] realize the kernel's page table and use direct map for kernel

  - - [x] realize other virtual memory technologies(vm)

- interrupt
  - - [x] realize code to initialize interrupt

- cache
  - - [ ] realize the cache list
- file system
  - - [ ] realize the open close read write of files

- scheduling
  - - [x] realize the process allocator
  - - [ ] realize the code to set up first user process

Furtherly implement other features so that the kernel can really work

- scheduling
  - - [ ] realize the scheduler to do scheduling
  - - [x] realize the fork, exec, exit, wait, sleep system calls (nightly build)

### Effects
> The first four will start up the riscv cpu and return to supervised mode(os mode) instead in machine mode, and as the print is ready, it will print out 'hello world' message.
> The memory management part will implement a global allocator(will be discussed later) and thus physical memory allocation will be taken care by rust intelligence pointer of rust based on global allocator. And then the page mapping will be performed and will print out corresponding message.
> interrupt will be turned on, and interrupt routine will be performed if there is interrupt.
> process allocator will have scheduling algorithm and if there is process, shcedule will work.
> system calls are implemented and will work properly if the nightly build is stable.

### Difficulty
Since the xv6 is new to us and we are only rust beginner, so we takes a lot of time to learn rust and learn the xv6 architecture. And during implementation, we found that our workload is really heavy and writing an OS within 30 day is not so easy. 
From 5.1 to 5.30, we took at least 1 day/per week to stuck on this project, and we still not able to finish all the tasks.
We also meet some deadly bugs which is related to the hardware and hard to fix.Since the only fast way to take xv6 to rust is to directly copy it without using any rust feature, this is opposite to the purpose of this project. So we just mainly focus on our code quality instead of code quantity.

> The cache list is for file system so it is not implemented.
> The file system is simply to 'massive' to implement.
> The set up of the first process need support of file system so it is not implemented.
> As the first process cannot be set up, so there is no process to schedule.

**To mention**, in the opening(design document), the file system is an optional task, but the process is heavily related with file system, which is not realized by us then. 

## Implementation:
This part we will give a comparison between the official c implementation of xv6-riscv and some representative opensource rust implementation, and our implemenation over all work we've done, thus providing the reason of our implemenation. The comparsion is for the following parts:
```rust
pub unsafe fn rust_main() -> ! {
    let cpuid = crate::proc::cpu_id();
    if cpuid == 0{
        // init print
        crate::console::console_init();
        println!();
        println!("hello world");
        // init memory allocation
        crate::memory::kalloc::kinit();
        // init kernel virtual memory
        crate::memory::kvminit();
        crate::memory::kvminithart();
        // init process
        crate::proc::proc_init();
        // init interrupt: plic and trap
        crate::plic::plic_init();
        crate::plic::plic_init_hart();
        crate::trap::trap_init_hart();
        panic!();
    } else {
        
    }
    loop {}
}
```

### Start up
The start up process of the xv6 on riscv basically includes three 'phases'. The first runs in machine mode and you will see this in `start.rs` in the root directory, this basically set some register which could only be modified in machine mode and set the `rmain` as the return valus when exiting machine mode. Then the second phase will be the initlization in the `rmain` and the third will be start the first process, then start the shell for interaction. **But in this sector, we only talk about the first part.** Some of the rest will be discussed in following sectors.

As the start will need to operate the registers as required by the riscv64 instructions, so there is no big differences inter programming languages. Thanks for riscv64 that is much easier than booting an x86 machine. The qume will put the start point of the program at address `0x80000000` so we will need a kernel.ld to let the linker know where to put the asm code to call the start function. the `.cargo/cargo.toml` will need to be specified to let quem run the binary target instead of default.

here is the part in `kernel.ld`:
```ld
  /*
   * ensure that entry.S / _entry is at 0x80000000,
   * where qemu's -kernel jumps.
   */
  . = 0x80000000;
```

and the corresponding code in `entry.S`
```asm
_entry:
	# set up a stack for Rust.
    # stack0 is declared below,
    # with a 8192-byte stack per CPU.
    # sp = stack0 + (hartid * 8192)
    la sp, stack0
    li a0, 1024*8
	csrr a1, mhartid
    addi a1, a1, 1
    mul a0, a0, a1
    add sp, sp, a0
	# jump to start() in start.rs
    call start
```

`start.rs` will do some register operation in this function:
```rust
#[no_mangle]
pub unsafe fn start() -> ! {
    //set M Previous Privilege mode to Supervisor, for mret.
    startup::mstatus();
    //set M Execption Program Counter to main , for mret.
    // requires gcc -mcmodel=medany
    mepc::write(rust_main as usize);
    //disable paging
    satp::write(0);
    //delegate all interrupts and exceptions to supervisor mode
    startup::to_supervisor();
    //ask for clock interrupts
    timerinit();
    //give cpu id
    startup::allo_cpu();
    //switch to supervisor mode and jump to rmain()
    llvm_asm!("mret"::::"volatile");
    loop {}
}
```

The above is our implementation and there is not big difference between some representative implementaiton, even the c version, except we have wrap the register operation well.

### Print
There are two way to print the output to the qemu console: using the uart to serial or use the VGA to directy put it to the screen. xv6 uses the uart as the output. To print, we need to write characters to a proper register. In the case of format, the C implementation is replacing the special formattor symbol to meaningful content, while in the rust we need to implement a marco for the `print!()` and the `println!()` function, these macros needs to take the string and the formatter as the input , and print them to the uart serial. 
Different from c , we do not need to write the formatter ourselves, the rust have already provide an official `core::fmt`.
To avoid race condition of writing to a same register(UART), the C version utilities a spin lock for mutual exclusion, and in rust we use the spin crate to do the mutual exclusiton. In the following parts of the implementation, the spin crate will be used as well.


The writer and mutex in rust is:
```rust
//use the mutex lock lib to wrap our writer
static WRITER: Mutex<Writer> = Mutex::new(Writer {});
```

and the `core::fmt` is trait is:
```rust
//implement fmt::Writer, so we can use the core format to avoid format the print input ourselves
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            console::consputc(byte);
        }
        Ok(())
    }
}
```

### memory allocator
The differences are:
- c version: a simplified version of linked list allocation, where memory are alloced and freed by pages.
- other rust version: some use similar to c version, some implement the global allocator trait(see sector Summary) of rust, thus can make use of intelligence pointer of rust.(will be discussed in our implementationpart)
- our implementation: we implement a global allocator, so that we will warp the intelligent pointer when memory allocation is called. the global allocator trait of rust is a function when rust need to dyanmically alloc and free memory, it will use this. If not implemented, it will use the default verison which requires system calls. The global allocator we implemented here is a simple version of bump allocator, which can be changed to other fancy memory allocation algorithms. The memory layout of xv6 is kernel is put at address `0x80000000` and ther will be a pre set `PHYSTOP`. The space from end of kernel to `PHYSTOP` is the free space to use. This will be in detial discussion in next sector.


So these are som code part corresponding to the difference:
C version
```c
// Free the page of physical memory pointed at by v,
// which normally should have been returned by a
// call to kalloc().  (The exception is when
// initializing the allocator; see kinit above.)
void
kfree(void *pa)
{
...
}

// Allocate one 4096-byte page of physical memory.
// Returns a pointer that the kernel can use.
// Returns 0 if the memory cannot be allocated.
void *
kalloc(void)
{
...
}
```

Our version
```rust
#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());
pub fn kinit(){
    extern "C" {
        fn end();
    }
    let heap_start = end as usize;
    let heap_start = align_up(heap_start, PGSIZE);
    let heap_size = PHYSTOP - heap_start;
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
    ...
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    ...
}
```
 the `end` in above code is the end of kernel mentioned above and is set in `kernel.ld`:
 ```ld
  . = 0x80000000;
  .text :
  {
    ...
  }
  .rodata :
  {
    ...
  }
  .data : {
    ...
  }
  .bss : {
    ...
  }
  PROVIDE(end = .);
 ```

### virtual memory
Virtual memory is the big topic of the project. Virtual memory is realized by paging, which is a combiantion of hardware and Operating system. The `satp` register hold the address of theroot page table(there are three level of page table and this will be discussed later in detail) and there is also TLB, so whenever the OS modifies the pagetable, the TLB should be updated(can be flushed for simplicity)

The paging applied is SV39 on riscv64, which means the virtual address is inpereted as following:
|reserved|index in PT1|index in PT2|index in PT3|offset|total|
|:-:|:-:|:-:|:-:|:-:|:-:|
|25 bits|9 bits|9 bits|9 bits|12 bits|64 bits|

And the page is of 4096 bytes, which can be indexed by 12 bits of offset. There are 512 items in each page table due to 9 bits for index, so the PTE is of 64 bit size. And the PTE should be inpreted as following:

|PPN or VPN|Bit Flags|Bit Flags|total|
|:-:|:-:|:-:|:-:|
|PPN or VPN|Reserved|D-A-G-U-R-W-X-V|total|
|52 bits|4 bits|8 bits|64 bits|

The PPN and VPN are abbrervation for physical page number and virtual page number, and are actually PPN here. This is because PTE is pointed either to a seconderay level page table or to a physical page, which are all 4096 bytes sized and are aligned by 4096 bytes, so we do not need the 64 bit address, but 52 bit is enough.

Here are the comparison:
- C version: the core function is `walk`, which takes in a virtual address and a pagetable, then return the address of the PTE in the leaf page table. If the pagetables are not exist yet, can be alloc to create then. For virtual address to physical address translation porpose, the PTE is just interpreted as a physical address. For map page perpose, the PTE can then be modified via the pointer. there are also some other functions based on this:
    - map pages: before the kernel really started, the code has already been loaded into the physical address of `0x80000000` and some I/O are at special address like UART. So this will need to be maped **after** they have been placed at the location, thus we will perform direct map. However, map pages are useful in later user process memory allocation. The map pages function's implementation is simply map a continous memory from virtual address to physical address specified by starting addresse, thus set the correspong page table. After mapping all the memory, the kvm initialization is finished.
    - user space memory allocation and deallocation: this is just map the require virtual space memory to a new allocated physical memory, thus modify the process' page table. the deallocation is just the reverse. To notice that deallcation is simply set the PTE to invalid.
    - user process free resources, thus will free all the virtual memory of the user process and the page tables shoulde be freeed as well.
- Other rust version: we don't refer them much in this part, so we do not discuss them here.
- Our implementation: the walk is devided into three functions, the first is to get the corresponding physical address of an given virtual address; then is the allocation and deallocation of pagetables, which will be used in following map pages and user space memory allocation and deallocation. the corresping functions of c version is:
    - mappages: which is similar to c version, but as a function of the pagetable structure
    - user space memory allocation and deallocation: this is also similar to c version, but as a function of virtual addresses
    - user process free resources: this is also similar to c version, but as a function of pagetable structure.

walk in c:
```c
// Return the address of the PTE in page table pagetable
// that corresponds to virtual address va.  If alloc!=0,
// create any required page-table pages.
//
// The risc-v Sv39 scheme has three levels of page-table
// pages. A page-table page contains 512 64-bit PTEs.
// A 64-bit virtual address is split into five fields:
//   39..63 -- must be zero.
//   30..38 -- 9 bits of level-2 index.
//   21..29 -- 9 bits of level-1 index.
//   12..20 -- 9 bits of level-0 index.
//    0..11 -- 12 bits of byte offset within the page.
pte_t *
walk(pagetable_t pagetable, uint64 va, int alloc)
{
  for(int level = 2; level > 0; level--) {
    pte_t *pte = &pagetable[PX(level, va)];
    if(*pte & PTE_V) {
      pagetable = (pagetable_t)PTE2PA(*pte);
    } else {
      if(!alloc || (pagetable = (pde_t*)kalloc()) == 0)
        return 0;
      memset(pagetable, 0, PGSIZE);
      *pte = PA2PTE(pagetable) | PTE_V;
    }
  }
  return &pagetable[PX(0, va)];
}
```

The 'walk' in Virtual address, and walk alloc and walk free in pagetable is similar:
```rust
    pub unsafe fn to_pa(&self, pagetable: &mut PageTable) -> Option<PhysAddr> {
        let mut pgt = pagetable as * mut PageTable;
        for level in (0..3).rev() {
            let pte = *self.resolve_level(pgt, level);
            if ! pte.is_valid() {
                return None
            }
            if level == 0 {
                return Some(pte.to_pa())
            }
            pgt = pte.to_pagetable();
        }
        None
    }
```

the mappages used in `kvm init` in rust:
```rust
pub unsafe fn kvminit() {
    // uart
    println!("map uart: ");
    KERNEL_PAGETABLE.map_pages(
        VirtualAddr::from(UART0),
        PhysAddr::from(UART0),
        PGSIZE,
        PteFlag::R | PteFlag::W
    );
    ...
    extern "C" {
        fn etext();
    }
    let etext = etext as usize;
    // kernel text executable and read-only
    println!("map kernel text executable and read-only: ");
    KERNEL_PAGETABLE.map_pages(
        VirtualAddr::from(KERNBASE),
        PhysAddr::from(KERNBASE),
        etext - KERNBASE,
        PteFlag::R | PteFlag::X
    );
    ...
    println!("proc_mapstacks: ");
    for i in 0..NPROC {
        let pa: Box<PageTable> = unsafe { Box::new_zeroed().assume_init() };
        KERNEL_PAGETABLE.map_pages(
            VirtualAddr::from(KSTACK(i)),
            PhysAddr::from(Box::into_raw(pa) as usize),
            PGSIZE,
            PteFlag::R | PteFlag::W
        );
    }
}
```

and `mappages` nad `recursive free` of our rust implementation:
```rust
    pub unsafe fn map_pages(
        &mut self, va: VirtualAddr, pa: PhysAddr, size: usize, perm: PteFlag
    ) -> Result<(), &'static str> {
        if ! pa.is_aligned() {
            panic!();
        }
        let mut va_iter: usize = va.round_down();
        let mut pa_iter: PhysAddr = pa; 
        let addr_end: usize = va.add(size).round_up();
        println!("    map_pages, va = {:#x}, end = {:#x}", va_iter, addr_end);
        while va_iter != addr_end {
            let ppte: * mut PTE = walk_alloc(self, VirtualAddr::from(va_iter));
            if (*ppte).is_valid() {
                panic!();
            }
            *ppte = PTE::from_pa(pa_iter, perm | PteFlag::V);
            va_iter += PGSIZE;
            pa_iter = pa_iter.add(PGSIZE); 
        }
        Ok(())
    }
    pub unsafe fn recursive_free(&mut self) {
        for i in (0..512) {
            let pte = self.data[i];
            if pte.is_valid() && (
                pte.get() & (PteFlag::R.bits()|PteFlag::W.bits()|PteFlag::X.bits()) == 0
            ) {
                let pchild: *mut PageTable = pte.to_pagetable();
                (*pchild).recursive_free();
                self.data[i].set(0);
            } else if pte.is_valid() {
                panic!();
            }
        }
        let ppa = self as * mut PageTable;
        drop(Box::from_raw(ppa));
    }
```

There are some other functions to realize the functionality of virtual memory based functions writtern above, so they will not be detialde discussed here and are in `memory` mod.

### process
The xv6 have a very simple structure `proc_list` to store all the process. The number of totall process is fixed to a size 64 (which means the xv6 can only have maximun of 64 processes),each process have a pid as identifier, a pagetable in uvm to store the data, a state to store the current state, a context and trapframe to store the cpu registers when doing context switch and dealing with trap and interrupts, and some other control data.
**The process behavior is similar to the xv6-c system.**
- init a process:
    all the process are init when the os start, each with a page as the root pagetable as the uvm, and use the uvmallc/ uvmdealloc method to get/free the pagetable. the state is `UNUSED` and all the other parameter are set to 0 or 1.
- allocating a process:
    first it will give a currently `UNUSED` process a `pid`, each time a pid is allocated, the next pid increase by 1, then set the state to `USED`, and allocate the context and trapframe for it.
- free a process:
    just like the initiate process, give all the struct stored value 0 or -1,then set the process state to `UNUSED`. Then free the pagetable recursively , sincne we only have the root pagetable stored in the process.
- other system calls:
the majority sys calls have only got a nighty version, since we do not implement the filesystem, so some of them can still not work properly. Like the fork will also deal with the file system, when creating the child process.

 
## Future direction:
The goal of this project in the official repo is to re-implement an xv6 in rust, which including the file system and shell, which are necessary for an operating system. However, due to the limitation of time, these are not included in the design document. So there are servals things to do in the future:
- the parts that havn't finished in the first part `Result Analyse`.
- the necessary parts to form an runnable operating system, which is mentioned above.
- refactor the code to fit the feature of Rust better: Rust do many analyse at complie time and little at run time to gunrantee its efficiency. The `Modern Operting systems` book quotes that usually there will be around ten bugs in a thousand lines of code, so it will be reasonable to use rust's powerful static analyze functionality as much as possible. So the following parts, like memory allocation, virtual memory and paging, process and file system should be writtern in more rust like way instead of original c style, or even pure literally translation.
- some limitation of xv6 itself: xv6 only allow 64 processes, and do not have the fancy feature. priority donation or MLQFS scheduling in previous project `pintos: threads`. Some data structure and algorithms which have been applied to modern operating systems can be added as well. So the following serval features can be added to be enhancement of xv6:
    - dynamic process numbers: pintos have very varfully designed its memory allocation and linked list. As memory allocation is already implemented in xv6, so this will not need much modification. But the well designed linked list for threads in pintos can be referenced here.
    - priority donation and MLQFS scheduling: the process scheduling is simply a combination of voluntary yielding CPU and timer interrupt based 'quantumn charges'. The priority based scheduling algorithm and priority donation can be added in xv6 and so do the MLQFS scheduling.
    - Some modern operating system will have fancy kernel data structures like `rbtree` in linus kernel to make kernel more efficient. These canbe added as well.
 
## Summary:
Summarize the main techniques learned through the project and the experience of teamwork.

### main techniques
- Rust feature
    - `ownership`
    this is a rust unique technique of memory management , which is safe and also efficient, please refer to https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html for more detail.
    This feature is useful but also bring some problems when coding.
    - trait
    trait is a interface, but it can have a defaut implememttion, since rust do not have inheritence, so we mainly use the trait to let it behave likes inheritence.
    - lock free programming
    rust variables have the `ownership` , so if the variable is not a `mut` variable, it's value cannot be changed.Also the variable passing to the functions have also pass its ownership to the function, so it can only modified in this function and will be deleted if the function end.
- OOAD
    - interface
    we use interface for some structure that have similar functions, like `PhysAddr` and `VirtAddr` share the same trait `Addr`, this is helpful for decoupling.
    - functional programming
    we implemet some of the static functions in c to the structures we define in rust to make them more ooad, and easy to read and safer to use. In c there is a lot of raw pointer which is unsafe and not allowed in rust, we modify them and try to use different structures to replace them.
    - design patterns
    we use several design patterns to make our code more flexible. We remove a bunch of macros in c and makes them to rust functions or structures.
### team work
- git
    Git is easy to use for version control, if we meet some bugs and not easy to fix it, we can choose to rollback if we have no option. It is also easy to fetch the code if we write seperately.
- pair programming
     Pair programming means two people using the same computer together and writing code together. When one people is coding , the other one will review his work. And we take turn every 1-2 hours. So we can code for more than 20 hours without getting tied. And each of us have the full view of the code, the code is less buggy because there is always another person reviewing the code. 
## Division of labor:
the majority of the part are finished by both of us since we use pair programming.
The following are the seperate part:
![](https://codimd.s3.shivering-isles.com/demo/uploads/upload_1118fa5a32dc4195dcaca9f758a3a9fa.png)
![](https://codimd.s3.shivering-isles.com/demo/uploads/upload_1447380cb0345cc906bd8bf12c653642.png)
**To notice**, since we are pair programming, so some code are coded by different people on the same computer, so the commit amount will be different. **But the overall contribut is equal contribute.**

## Acknowledgement

We would like thank Dr. Bo Tang for his dedicated teaching, and well prepared lectures and patient answering to questions and TA Haotian Liu for his lab guidence and DBGoupe for lab contents. We also want to thank Gogo for devoting much time to discuss detailed implementation with us.

## References
Text book and other useful materials
- Operating System Concepts
- Modern Operationg System
- The Rust Programming Languange

These are related xv6 repo
https://github.com/mit-pdos/xv6-riscv
https://github.com/skyzh/core-os-riscv
https://github.com/Jaic1/xv6-riscv-rust

Some other useful blogs 
https://os.phil-opp.com/
http://osblog.stephenmarz.com/index.html

Useful xv6 and rust document
https://doc.rust-lang.org/stable/book/
https://pdos.csail.mit.edu/6.828/2020/xv6/book-riscv-rev1.pdf




