use std::arch::asm;
use std::boxed::Box;

pub fn step(instruction: u32, registers: [i32; 31]) -> [i32; 31] {
    #[allow(unused_mut)]
    #[allow(unused_variables)]
    #[allow(unused_assignments)]
    let [mut x1, mut x2, mut x3, mut x4, mut x5, mut x6, mut x7, mut x8, mut x9, mut x10, mut x11, mut x12, mut x13, mut x14, mut x15, mut x16, mut x17, mut x18, mut x19, mut x20, mut x21, mut x22, mut x23, mut x24, mut x25, mut x26, mut x27, mut x28, mut x29, mut x30, mut x31] =
        registers;

    //TODO little parser of instruction to generate the number for instruction[0], only allow
    //temporary registers to be used. Except x31 & x5 since we use ourselves, in the worng way really.
    let instruction: [u32; 2] = [
        0b00000000101000110000001100010011, //addi x6, x6, 10
        0b00000000000000001000000001100111, //jalr x0, 0(x1)
    ];

    let instruction_pointer: *const [u32; 2] = &instruction;
    println!("instruction pointer {:?}", instruction_pointer);

    let testing_second_instruction_pointer: *const u32 = &instruction[1];
    println!("Points to second instruction? {:?}", testing_second_instruction_pointer);

    //Maybe the problem is they are on the stack and it fucks things?
    let boxed_instr = Box::new(instruction.clone());
    let box_ptr = Box::into_raw(boxed_instr);
    println!("boxed {:?}", box_ptr);

    x31 = instruction_pointer as i32;
    println!("x31 {:x}", x31);

    unsafe {
        asm!(
        "add x5, x1, x0", //Save return address
        "lw x30, 0(x31)", //TODO remove now retrieve word see, if is instruction[0].... ehm
                          //yes..... it is, wtf?
        //TODO error now"jalr x1, 0(x31)", //Set return address in x1, jump to x31 Our dynamic instruction
        "add x1, x5, x0", //Set return address back
        // Only the temporaries allow input, looks like we take off two from six, pfew.... Maybe
        // misuse argument registers?
        out("x1") x1,
        //out("x2") x2,
        //out("x3") x3,
        //out("x4") x4,
        inout("x5") x5,
        inout("x6") x6,
        inout("x7") x7,
        //out("x8") x8,
        //out("x9") x9,
        out("x10") x10,
        out("x11") x11,
        out("x12") x12,
        out("x13") x13,
        out("x14") x14,
        out("x15") x15,
        out("x16") x16,
        out("x17") x17,
        out("x18") x18,
        out("x19") x19,
        out("x20") x20,
        out("x21") x21,
        out("x22") x22,
        out("x23") x23,
        out("x24") x24,
        out("x25") x25,
        out("x26") x26,
        out("x27") x27,
        inout("x28") x28,
        inout("x29") x29,
        inout("x30") x30,
        inout("x31") x31,
        )
    }

    [
        x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15, x16, x17, x18, x19, x20,
        x21, x22, x23, x24, x25, x26, x27, x28, x29, x30, x31,
    ]
}
