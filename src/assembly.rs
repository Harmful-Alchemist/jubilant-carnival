use std::arch::asm;

pub fn step(instruction: u32, registers: [i32; 31]) -> [i32; 31] {
    let [mut x1, mut x2, mut x3, mut x4, mut x5, mut x6, mut x7, mut x8, mut x9, mut x10, mut x11, mut x12, mut x13, mut x14, mut x15, mut x16, mut x17, mut x18, mut x19, mut x20, mut x21, mut x22, mut x23, mut x24, mut x25, mut x26, mut x27, mut x28, mut x29, mut x30, mut x31] =
        registers;

    unsafe {
        asm!(
        "addi x5, x5, 5",
        // Add "useless" instructions so all registers are used, and the compiler is happy
        "add x1, x1, x0",
        //"add x2, x2, x0", Some registers are not allowed as operands.
        //"add x3, x3, x0",
        //"add x4, x4, x0",
        "add x5, x5, x0",
        "add x6, x6, x0",
        "add x7, x7, x0",
        //"add x8, x8, x0",
        //"add x9, x9, x0",
        "add x10, x10, x0",
        "add x11, x11, x0",
        "add x12, x12, x0",
        "add x13, x13, x0",
        "add x14, x14, x0",
        "add x15, x15, x0",
        "add x16, x16, x0",
        "add x17, x17, x0",
        "add x18, x18, x0",
        "add x19, x19, x0",
        "add x20, x20, x0",
        "add x21, x21, x0",
        "add x22, x22, x0",
        "add x23, x23, x0",
        "add x24, x24, x0",
        "add x25, x25, x0",
        "add x26, x26, x0",
        "add x27, x27, x0",
        "add x28, x28, x0",
        "add x29, x29, x0",
        "add x30, x30, x0",
        "add x31, x31, x0",
        // Only the temporaries allow input
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
