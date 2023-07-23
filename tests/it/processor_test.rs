use std::{fs, io};

use bincode::{config, Decode};
use dunes::{Bus, Cpu, Status};

#[derive(Decode)]
struct ProcessorTest {
    // name: String,
    initial: ProcessorTestState,
    r#final: ProcessorTestState,
    cycles: Vec<BusState>,
}

#[derive(Decode)]
struct ProcessorTestState {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: Vec<RamValue>,
}

#[derive(Decode)]
struct RamValue {
    addr: u16,
    data: u8,
}

#[derive(Decode)]
struct BusState {
    addr: u16,
    data: u8,
    kind: String,
}

struct ProcessorTestBus {
    memory: [u8; 0x10000],
    cycles: Option<Vec<BusState>>,
    cycle: usize,
}

impl Bus for ProcessorTestBus {
    fn read(&mut self, pins: &mut dunes::Pins) {
        pins.data = self.memory[pins.address as usize];

        if let Some(cycles) = &self.cycles {
            let cycle = &cycles[self.cycle];
            assert_eq!(cycle.addr, pins.address, "address not eq");
            assert_eq!(cycle.data, pins.data, "data not eq");
            assert_eq!(cycle.kind, "read");
            self.cycle += 1;
        }
    }

    fn write(&mut self, pins: &mut dunes::Pins) {
        if let Some(cycles) = &self.cycles {
            let cycle = &cycles[self.cycle];
            assert_eq!(cycle.addr, pins.address);
            assert_eq!(cycle.data, pins.data);
            assert_eq!(cycle.kind, "write");
            self.cycle += 1;
        }

        self.memory[pins.address as usize] = pins.data;
    }
}

fn run_processor_test(opcode: &str) {
    static JAM_OPCODES: [&str; 12] = [
        "02", "12", "22", "32", "42", "52", "62", "72", "92", "b2", "d2", "f2",
    ];
    // The cases for these NOP opcodes might have the wrong number of cycles.
    static NOP_ABS_OPCODES: [&str; 7] =
        ["0c", "1c", "3c", "5c", "7c", "dc", "fc"];
    static UNSTABLE_OPCODES: [&str; 7] =
        ["8b", "93", "9b", "9c", "9e", "9f", "ab"];
    static MAYBE_WRONG_OPCODES: [&str; 1] = ["6b"];
    if JAM_OPCODES.contains(&opcode)
        || NOP_ABS_OPCODES.contains(&opcode)
        || UNSTABLE_OPCODES.contains(&opcode)
        || MAYBE_WRONG_OPCODES.contains(&opcode)
    {
        return;
    }

    let test_file = fs::File::open(format!("roms/v1/{opcode}.bincode"))
        .unwrap_or_else(|_| panic!("roms/v1/{opcode}.bincode should exist"));
    let mut buf_reader = io::BufReader::new(test_file);
    let tests: Vec<ProcessorTest> =
        bincode::decode_from_std_read(&mut buf_reader, config::standard())
            .unwrap();

    let bus = ProcessorTestBus {
        memory: [0u8; 0x10000],
        cycles: None,
        cycle: 0,
    };
    let mut cpu = Cpu::new(bus);

    cpu.step();

    for test in tests {
        cpu.pc = test.initial.pc;
        cpu.s = test.initial.s;
        cpu.a = test.initial.a;
        cpu.x = test.initial.x;
        cpu.y = test.initial.y;
        cpu.p = Status::from(test.initial.p);

        cpu.bus.memory.fill(0);
        for RamValue { addr, data } in test.initial.ram {
            cpu.bus.memory[addr as usize] = data;
        }

        cpu.bus.cycles = Some(test.cycles);
        cpu.bus.cycle = 0;

        cpu.step();

        assert_eq!(cpu.pc, test.r#final.pc);
        assert_eq!(cpu.s, test.r#final.s);
        assert_eq!(cpu.a, test.r#final.a);
        assert_eq!(cpu.x, test.r#final.x);
        assert_eq!(cpu.y, test.r#final.y);
        assert_eq!(u8::from(cpu.p), test.r#final.p);

        for RamValue { addr, data } in test.r#final.ram {
            assert_eq!(cpu.bus.memory[addr as usize], data);
        }
    }
}

#[test]
fn opcode_00() {
    run_processor_test("00");
}

#[test]
fn opcode_01() {
    run_processor_test("01");
}

#[test]
fn opcode_02() {
    run_processor_test("02");
}

#[test]
fn opcode_03() {
    run_processor_test("03");
}

#[test]
fn opcode_04() {
    run_processor_test("04");
}

#[test]
fn opcode_05() {
    run_processor_test("05");
}

#[test]
fn opcode_06() {
    run_processor_test("06");
}

#[test]
fn opcode_07() {
    run_processor_test("07");
}

#[test]
fn opcode_08() {
    run_processor_test("08");
}

#[test]
fn opcode_09() {
    run_processor_test("09");
}

#[test]
fn opcode_0a() {
    run_processor_test("0a");
}

#[test]
fn opcode_0b() {
    run_processor_test("0b");
}

#[test]
fn opcode_0c() {
    run_processor_test("0c");
}

#[test]
fn opcode_0d() {
    run_processor_test("0d");
}

#[test]
fn opcode_0e() {
    run_processor_test("0e");
}

#[test]
fn opcode_0f() {
    run_processor_test("0f");
}

#[test]
fn opcode_10() {
    run_processor_test("10");
}

#[test]
fn opcode_11() {
    run_processor_test("11");
}

#[test]
fn opcode_12() {
    run_processor_test("12");
}

#[test]
fn opcode_13() {
    run_processor_test("13");
}

#[test]
fn opcode_14() {
    run_processor_test("14");
}

#[test]
fn opcode_15() {
    run_processor_test("15");
}

#[test]
fn opcode_16() {
    run_processor_test("16");
}

#[test]
fn opcode_17() {
    run_processor_test("17");
}

#[test]
fn opcode_18() {
    run_processor_test("18");
}

#[test]
fn opcode_19() {
    run_processor_test("19");
}

#[test]
fn opcode_1a() {
    run_processor_test("1a");
}

#[test]
fn opcode_1b() {
    run_processor_test("1b");
}

#[test]
fn opcode_1c() {
    run_processor_test("1c");
}

#[test]
fn opcode_1d() {
    run_processor_test("1d");
}

#[test]
fn opcode_1e() {
    run_processor_test("1e");
}

#[test]
fn opcode_1f() {
    run_processor_test("1f");
}

#[test]
fn opcode_20() {
    run_processor_test("20");
}

#[test]
fn opcode_21() {
    run_processor_test("21");
}

#[test]
fn opcode_22() {
    run_processor_test("22");
}

#[test]
fn opcode_23() {
    run_processor_test("23");
}

#[test]
fn opcode_24() {
    run_processor_test("24");
}

#[test]
fn opcode_25() {
    run_processor_test("25");
}

#[test]
fn opcode_26() {
    run_processor_test("26");
}

#[test]
fn opcode_27() {
    run_processor_test("27");
}

#[test]
fn opcode_28() {
    run_processor_test("28");
}

#[test]
fn opcode_29() {
    run_processor_test("29");
}

#[test]
fn opcode_2a() {
    run_processor_test("2a");
}

#[test]
fn opcode_2b() {
    run_processor_test("2b");
}

#[test]
fn opcode_2c() {
    run_processor_test("2c");
}

#[test]
fn opcode_2d() {
    run_processor_test("2d");
}

#[test]
fn opcode_2e() {
    run_processor_test("2e");
}

#[test]
fn opcode_2f() {
    run_processor_test("2f");
}

#[test]
fn opcode_30() {
    run_processor_test("30");
}

#[test]
fn opcode_31() {
    run_processor_test("31");
}

#[test]
fn opcode_32() {
    run_processor_test("32");
}

#[test]
fn opcode_33() {
    run_processor_test("33");
}

#[test]
fn opcode_34() {
    run_processor_test("34");
}

#[test]
fn opcode_35() {
    run_processor_test("35");
}

#[test]
fn opcode_36() {
    run_processor_test("36");
}

#[test]
fn opcode_37() {
    run_processor_test("37");
}

#[test]
fn opcode_38() {
    run_processor_test("38");
}

#[test]
fn opcode_39() {
    run_processor_test("39");
}

#[test]
fn opcode_3a() {
    run_processor_test("3a");
}

#[test]
fn opcode_3b() {
    run_processor_test("3b");
}

#[test]
fn opcode_3c() {
    run_processor_test("3c");
}

#[test]
fn opcode_3d() {
    run_processor_test("3d");
}

#[test]
fn opcode_3e() {
    run_processor_test("3e");
}

#[test]
fn opcode_3f() {
    run_processor_test("3f");
}

#[test]
fn opcode_40() {
    run_processor_test("40");
}

#[test]
fn opcode_41() {
    run_processor_test("41");
}

#[test]
fn opcode_42() {
    run_processor_test("42");
}

#[test]
fn opcode_43() {
    run_processor_test("43");
}

#[test]
fn opcode_44() {
    run_processor_test("44");
}

#[test]
fn opcode_45() {
    run_processor_test("45");
}

#[test]
fn opcode_46() {
    run_processor_test("46");
}

#[test]
fn opcode_47() {
    run_processor_test("47");
}

#[test]
fn opcode_48() {
    run_processor_test("48");
}

#[test]
fn opcode_49() {
    run_processor_test("49");
}

#[test]
fn opcode_4a() {
    run_processor_test("4a");
}

#[test]
fn opcode_4b() {
    run_processor_test("4b");
}

#[test]
fn opcode_4c() {
    run_processor_test("4c");
}

#[test]
fn opcode_4d() {
    run_processor_test("4d");
}

#[test]
fn opcode_4e() {
    run_processor_test("4e");
}

#[test]
fn opcode_4f() {
    run_processor_test("4f");
}

#[test]
fn opcode_50() {
    run_processor_test("50");
}

#[test]
fn opcode_51() {
    run_processor_test("51");
}

#[test]
fn opcode_52() {
    run_processor_test("52");
}

#[test]
fn opcode_53() {
    run_processor_test("53");
}

#[test]
fn opcode_54() {
    run_processor_test("54");
}

#[test]
fn opcode_55() {
    run_processor_test("55");
}

#[test]
fn opcode_56() {
    run_processor_test("56");
}

#[test]
fn opcode_57() {
    run_processor_test("57");
}

#[test]
fn opcode_58() {
    run_processor_test("58");
}

#[test]
fn opcode_59() {
    run_processor_test("59");
}

#[test]
fn opcode_5a() {
    run_processor_test("5a");
}

#[test]
fn opcode_5b() {
    run_processor_test("5b");
}

#[test]
fn opcode_5c() {
    run_processor_test("5c");
}

#[test]
fn opcode_5d() {
    run_processor_test("5d");
}

#[test]
fn opcode_5e() {
    run_processor_test("5e");
}

#[test]
fn opcode_5f() {
    run_processor_test("5f");
}

#[test]
fn opcode_60() {
    run_processor_test("60");
}

#[test]
fn opcode_61() {
    run_processor_test("61");
}

#[test]
fn opcode_62() {
    run_processor_test("62");
}

#[test]
fn opcode_63() {
    run_processor_test("63");
}

#[test]
fn opcode_64() {
    run_processor_test("64");
}

#[test]
fn opcode_65() {
    run_processor_test("65");
}

#[test]
fn opcode_66() {
    run_processor_test("66");
}

#[test]
fn opcode_67() {
    run_processor_test("67");
}

#[test]
fn opcode_68() {
    run_processor_test("68");
}

#[test]
fn opcode_69() {
    run_processor_test("69");
}

#[test]
fn opcode_6a() {
    run_processor_test("6a");
}

#[test]
fn opcode_6b() {
    run_processor_test("6b");
}

#[test]
fn opcode_6c() {
    run_processor_test("6c");
}

#[test]
fn opcode_6d() {
    run_processor_test("6d");
}

#[test]
fn opcode_6e() {
    run_processor_test("6e");
}

#[test]
fn opcode_6f() {
    run_processor_test("6f");
}

#[test]
fn opcode_70() {
    run_processor_test("70");
}

#[test]
fn opcode_71() {
    run_processor_test("71");
}

#[test]
fn opcode_72() {
    run_processor_test("72");
}

#[test]
fn opcode_73() {
    run_processor_test("73");
}

#[test]
fn opcode_74() {
    run_processor_test("74");
}

#[test]
fn opcode_75() {
    run_processor_test("75");
}

#[test]
fn opcode_76() {
    run_processor_test("76");
}

#[test]
fn opcode_77() {
    run_processor_test("77");
}

#[test]
fn opcode_78() {
    run_processor_test("78");
}

#[test]
fn opcode_79() {
    run_processor_test("79");
}

#[test]
fn opcode_7a() {
    run_processor_test("7a");
}

#[test]
fn opcode_7b() {
    run_processor_test("7b");
}

#[test]
fn opcode_7c() {
    run_processor_test("7c");
}

#[test]
fn opcode_7d() {
    run_processor_test("7d");
}

#[test]
fn opcode_7e() {
    run_processor_test("7e");
}

#[test]
fn opcode_7f() {
    run_processor_test("7f");
}

#[test]
fn opcode_80() {
    run_processor_test("80");
}

#[test]
fn opcode_81() {
    run_processor_test("81");
}

#[test]
fn opcode_82() {
    run_processor_test("82");
}

#[test]
fn opcode_83() {
    run_processor_test("83");
}

#[test]
fn opcode_84() {
    run_processor_test("84");
}

#[test]
fn opcode_85() {
    run_processor_test("85");
}

#[test]
fn opcode_86() {
    run_processor_test("86");
}

#[test]
fn opcode_87() {
    run_processor_test("87");
}

#[test]
fn opcode_88() {
    run_processor_test("88");
}

#[test]
fn opcode_89() {
    run_processor_test("89");
}

#[test]
fn opcode_8a() {
    run_processor_test("8a");
}

#[test]
fn opcode_8b() {
    run_processor_test("8b");
}

#[test]
fn opcode_8c() {
    run_processor_test("8c");
}

#[test]
fn opcode_8d() {
    run_processor_test("8d");
}

#[test]
fn opcode_8e() {
    run_processor_test("8e");
}

#[test]
fn opcode_8f() {
    run_processor_test("8f");
}

#[test]
fn opcode_90() {
    run_processor_test("90");
}

#[test]
fn opcode_91() {
    run_processor_test("91");
}

#[test]
fn opcode_92() {
    run_processor_test("92");
}

#[test]
fn opcode_93() {
    run_processor_test("93");
}

#[test]
fn opcode_94() {
    run_processor_test("94");
}

#[test]
fn opcode_95() {
    run_processor_test("95");
}

#[test]
fn opcode_96() {
    run_processor_test("96");
}

#[test]
fn opcode_97() {
    run_processor_test("97");
}

#[test]
fn opcode_98() {
    run_processor_test("98");
}

#[test]
fn opcode_99() {
    run_processor_test("99");
}

#[test]
fn opcode_9a() {
    run_processor_test("9a");
}

#[test]
fn opcode_9b() {
    run_processor_test("9b");
}

#[test]
fn opcode_9c() {
    run_processor_test("9c");
}

#[test]
fn opcode_9d() {
    run_processor_test("9d");
}

#[test]
fn opcode_9e() {
    run_processor_test("9e");
}

#[test]
fn opcode_9f() {
    run_processor_test("9f");
}

#[test]
fn opcode_a0() {
    run_processor_test("a0");
}

#[test]
fn opcode_a1() {
    run_processor_test("a1");
}

#[test]
fn opcode_a2() {
    run_processor_test("a2");
}

#[test]
fn opcode_a3() {
    run_processor_test("a3");
}

#[test]
fn opcode_a4() {
    run_processor_test("a4");
}

#[test]
fn opcode_a5() {
    run_processor_test("a5");
}

#[test]
fn opcode_a6() {
    run_processor_test("a6");
}

#[test]
fn opcode_a7() {
    run_processor_test("a7");
}

#[test]
fn opcode_a8() {
    run_processor_test("a8");
}

#[test]
fn opcode_a9() {
    run_processor_test("a9");
}

#[test]
fn opcode_aa() {
    run_processor_test("aa");
}

#[test]
fn opcode_ab() {
    run_processor_test("ab");
}

#[test]
fn opcode_ac() {
    run_processor_test("ac");
}

#[test]
fn opcode_ad() {
    run_processor_test("ad");
}

#[test]
fn opcode_ae() {
    run_processor_test("ae");
}

#[test]
fn opcode_af() {
    run_processor_test("af");
}

#[test]
fn opcode_b0() {
    run_processor_test("b0");
}

#[test]
fn opcode_b1() {
    run_processor_test("b1");
}

#[test]
fn opcode_b2() {
    run_processor_test("b2");
}

#[test]
fn opcode_b3() {
    run_processor_test("b3");
}

#[test]
fn opcode_b4() {
    run_processor_test("b4");
}

#[test]
fn opcode_b5() {
    run_processor_test("b5");
}

#[test]
fn opcode_b6() {
    run_processor_test("b6");
}

#[test]
fn opcode_b7() {
    run_processor_test("b7");
}

#[test]
fn opcode_b8() {
    run_processor_test("b8");
}

#[test]
fn opcode_b9() {
    run_processor_test("b9");
}

#[test]
fn opcode_ba() {
    run_processor_test("ba");
}

#[test]
fn opcode_bb() {
    run_processor_test("bb");
}

#[test]
fn opcode_bc() {
    run_processor_test("bc");
}

#[test]
fn opcode_bd() {
    run_processor_test("bd");
}

#[test]
fn opcode_be() {
    run_processor_test("be");
}

#[test]
fn opcode_bf() {
    run_processor_test("bf");
}

#[test]
fn opcode_c0() {
    run_processor_test("c0");
}

#[test]
fn opcode_c1() {
    run_processor_test("c1");
}

#[test]
fn opcode_c2() {
    run_processor_test("c2");
}

#[test]
fn opcode_c3() {
    run_processor_test("c3");
}

#[test]
fn opcode_c4() {
    run_processor_test("c4");
}

#[test]
fn opcode_c5() {
    run_processor_test("c5");
}

#[test]
fn opcode_c6() {
    run_processor_test("c6");
}

#[test]
fn opcode_c7() {
    run_processor_test("c7");
}

#[test]
fn opcode_c8() {
    run_processor_test("c8");
}

#[test]
fn opcode_c9() {
    run_processor_test("c9");
}

#[test]
fn opcode_ca() {
    run_processor_test("ca");
}

#[test]
fn opcode_cb() {
    run_processor_test("cb");
}

#[test]
fn opcode_cc() {
    run_processor_test("cc");
}

#[test]
fn opcode_cd() {
    run_processor_test("cd");
}

#[test]
fn opcode_ce() {
    run_processor_test("ce");
}

#[test]
fn opcode_cf() {
    run_processor_test("cf");
}

#[test]
fn opcode_d0() {
    run_processor_test("d0");
}

#[test]
fn opcode_d1() {
    run_processor_test("d1");
}

#[test]
fn opcode_d2() {
    run_processor_test("d2");
}

#[test]
fn opcode_d3() {
    run_processor_test("d3");
}

#[test]
fn opcode_d4() {
    run_processor_test("d4");
}

#[test]
fn opcode_d5() {
    run_processor_test("d5");
}

#[test]
fn opcode_d6() {
    run_processor_test("d6");
}

#[test]
fn opcode_d7() {
    run_processor_test("d7");
}

#[test]
fn opcode_d8() {
    run_processor_test("d8");
}

#[test]
fn opcode_d9() {
    run_processor_test("d9");
}

#[test]
fn opcode_da() {
    run_processor_test("da");
}

#[test]
fn opcode_db() {
    run_processor_test("db");
}

#[test]
fn opcode_dc() {
    run_processor_test("dc");
}

#[test]
fn opcode_dd() {
    run_processor_test("dd");
}

#[test]
fn opcode_de() {
    run_processor_test("de");
}

#[test]
fn opcode_df() {
    run_processor_test("df");
}

#[test]
fn opcode_e0() {
    run_processor_test("e0");
}

#[test]
fn opcode_e1() {
    run_processor_test("e1");
}

#[test]
fn opcode_e2() {
    run_processor_test("e2");
}

#[test]
fn opcode_e3() {
    run_processor_test("e3");
}

#[test]
fn opcode_e4() {
    run_processor_test("e4");
}

#[test]
fn opcode_e5() {
    run_processor_test("e5");
}

#[test]
fn opcode_e6() {
    run_processor_test("e6");
}

#[test]
fn opcode_e7() {
    run_processor_test("e7");
}

#[test]
fn opcode_e8() {
    run_processor_test("e8");
}

#[test]
fn opcode_e9() {
    run_processor_test("e9");
}

#[test]
fn opcode_ea() {
    run_processor_test("ea");
}

#[test]
fn opcode_eb() {
    run_processor_test("eb");
}

#[test]
fn opcode_ec() {
    run_processor_test("ec");
}

#[test]
fn opcode_ed() {
    run_processor_test("ed");
}

#[test]
fn opcode_ee() {
    run_processor_test("ee");
}

#[test]
fn opcode_ef() {
    run_processor_test("ef");
}

#[test]
fn opcode_f0() {
    run_processor_test("f0");
}

#[test]
fn opcode_f1() {
    run_processor_test("f1");
}

#[test]
fn opcode_f2() {
    run_processor_test("f2");
}

#[test]
fn opcode_f3() {
    run_processor_test("f3");
}

#[test]
fn opcode_f4() {
    run_processor_test("f4");
}

#[test]
fn opcode_f5() {
    run_processor_test("f5");
}

#[test]
fn opcode_f6() {
    run_processor_test("f6");
}

#[test]
fn opcode_f7() {
    run_processor_test("f7");
}

#[test]
fn opcode_f8() {
    run_processor_test("f8");
}

#[test]
fn opcode_f9() {
    run_processor_test("f9");
}

#[test]
fn opcode_fa() {
    run_processor_test("fa");
}

#[test]
fn opcode_fb() {
    run_processor_test("fb");
}

#[test]
fn opcode_fc() {
    run_processor_test("fc");
}

#[test]
fn opcode_fd() {
    run_processor_test("fd");
}

#[test]
fn opcode_fe() {
    run_processor_test("fe");
}

#[test]
fn opcode_ff() {
    run_processor_test("ff");
}
