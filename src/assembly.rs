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

//    let testing_second_instruction_pointer: *const u32 = &instruction[1];
//    println!("Points to second instruction? {:?}", testing_second_instruction_pointer);
//
//    //Maybe the problem is they are on the stack and it fucks things?
//    let boxed_instr = Box::new(instruction.clone());
//    let box_ptr = Box::into_raw(boxed_instr);
//    println!("boxed {:?}", box_ptr);
//
    x30 = instruction_pointer as i32;
    //
    //x29 = 0b00000000000000001000000001100111; //jalr x0, 0(x1)
    //x30 = 0b00000000101000110000001100010011; //addi x6, x6, 10
    println!("x30 {:x}", x30);

    unsafe {
        asm!(
 

        "add x5, x1, x0",
        "jalr x1, 0(x30)",
        "add x1, x5, x0",


        // Maybe we can get the PC register in another way. Only numeric labels allowed... Then
        // says unknow register 2..... Oh needs f or b for forwards or backwards. Needs a register
        // not a label.. add?
 //       "sw x30, 2f(x0)", //Not the right syntax..
   //     "jalr x1, 2f(x0)", //Jump to our instructions
     //   "2:",
       // "addi x1, x1, 0", //Delete our instructions

        // Looks like we don't have access to the pc register this way, compile error.    

            // Still instruction access fault. Stack looks like I would expect based on this...
//        "addi sp, sp, -8", //Room for our two instructions
//        "sw x29, 4(sp)", //Store inststructions in the stack.
//        "sw x30, 0(sp)",
//        "jalr x1, 0(sp)", //Jump to our instructions
//        "addi sp, sp, 8", //Delete our instructions
//
        // Looks like we don't have access to the pc register this way, compile error.    
        //"sw x30, 4(pc)", // overwrite next instruction?
        //"add x0, x0, x0", //NOP overwritten by previous?


        //old tries below    
        //"add x5, x1, x0", //Save return address
        

        // fails but shows crash report with MSTATUS: 0x00001881 bit 11 & 12 should be 0x11 for machine mode. Looks like they are. 
        // So says: https://www.espressif.com/sites/default/files/documentation/esp32-c3_technical_reference_manual_en.pdf
        // error was: Guru Meditation Error: Core  0 panic'ed (Load access fault). Exception was unhandled.
        //"addi x28, x0, 0x300",
        //"lw x29, 0(x28)", // Read mstatus register of esp32 c3 0x300 = 0d768
        

        //"addi x30, x0, 0x3A0", //pmpcfg0 nope trapped
        //"lw x31, 0(x30)",


        //"la x29, MSTATUS", Linker (ld) error

        //"lw x30, 0(x31)", //TODO remove now retrieve word see, if is instruction[0].... ehm
                          //yes..... it is, wtf?

        //Guru Meditation Error: Core  0 panic'ed (Instruction access fault). Exception was unhandled.
        //MTVAL does show the addr, is x31 MTVAL register? Nah just in T5 now when printing.
        // Oh is it PMP, Physical Memory protection? In that case not easily turned off in M mode
        // even. ("so that even M-mode software cannot change them without a system reset")
        //"jalr x1, 0(x30)", //Set return address in x1z, jump to x31 Our dynamic instruction
        //"add x1, x5, x0", //Set return address back
        
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

        //out("pmpcfg0") pmpcfg0, hmm nope unknown
        )
    }

    [
        x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15, x16, x17, x18, x19, x20,
        x21, x22, x23, x24, x25, x26, x27, x28, x29, x30, x31,
    ]
}
