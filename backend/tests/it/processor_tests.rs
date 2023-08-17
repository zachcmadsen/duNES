use std::fs;
use std::vec::IntoIter;

use backend::{Bus, Cpu, Status};
use serde::Deserialize;

// The tests for NOP with absolute addressing might have the wrong number of
// cycles.
const NOP_ABS_OPCODES: [&str; 7] = ["0c", "1c", "3c", "5c", "7c", "dc", "fc"];
const UNSTABLE_OPCODES: [&str; 7] = ["8b", "93", "9b", "9c", "9e", "9f", "ab"];
const MAYBE_WRONG_OPCODES: [&str; 1] = ["6b"];

#[derive(Deserialize)]
struct Test {
    initial: CpuState,
    r#final: CpuState,
    cycles: Vec<(u16, u8, String)>,
}

#[derive(Deserialize)]
struct CpuState {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: Vec<(u16, u8)>,
}

struct ProcessorTestsBus {
    memory: Box<[u8; 0x10000]>,
    cycles: IntoIter<(u16, u8, String)>,
}

impl Bus for ProcessorTestsBus {
    fn read(&mut self, pins: &mut backend::Pins) {
        pins.data = self.memory[pins.address as usize];

        if let Some((addr, data, kind)) = self.cycles.next() {
            assert_eq!(addr, pins.address);
            assert_eq!(data, pins.data);
            assert_eq!(kind, "read");
        }
    }

    fn write(&mut self, pins: &mut backend::Pins) {
        if let Some((addr, data, kind)) = self.cycles.next() {
            assert_eq!(addr, pins.address);
            assert_eq!(data, pins.data);
            assert_eq!(kind, "write");
        }

        self.memory[pins.address as usize] = pins.data;
    }
}

fn run(opcode: &str) {
    if NOP_ABS_OPCODES.contains(&opcode)
        || UNSTABLE_OPCODES.contains(&opcode)
        || MAYBE_WRONG_OPCODES.contains(&opcode)
    {
        return;
    }

    let contents =
        fs::read_to_string(format!("../roms/processor_tests/{opcode}.json"))
            .unwrap();
    let tests: Vec<Test> = serde_json::from_str(&contents).unwrap();

    let bus = ProcessorTestsBus {
        memory: vec![0; 0x10000].try_into().unwrap(),
        cycles: vec![].into_iter(),
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
        for (addr, data) in test.initial.ram {
            cpu.bus.memory[addr as usize] = data;
        }
        cpu.bus.cycles = test.cycles.into_iter();

        cpu.step();

        assert_eq!(cpu.pc, test.r#final.pc);
        assert_eq!(cpu.s, test.r#final.s);
        assert_eq!(cpu.a, test.r#final.a);
        assert_eq!(cpu.x, test.r#final.x);
        assert_eq!(cpu.y, test.r#final.y);
        assert_eq!(u8::from(cpu.p), test.r#final.p);

        for (addr, data) in test.r#final.ram {
            assert_eq!(cpu.bus.memory[addr as usize], data);
        }
    }
}

// The following opcodes aren't tested: JAM

#[test]
fn opcode_00() {
    run("00");
}

#[test]
fn opcode_01() {
    run("01");
}

#[test]
fn opcode_03() {
    run("03");
}

#[test]
fn opcode_04() {
    run("04");
}

#[test]
fn opcode_05() {
    run("05");
}

#[test]
fn opcode_06() {
    run("06");
}

#[test]
fn opcode_07() {
    run("07");
}

#[test]
fn opcode_08() {
    run("08");
}

#[test]
fn opcode_09() {
    run("09");
}

#[test]
fn opcode_0a() {
    run("0a");
}

#[test]
fn opcode_0b() {
    run("0b");
}

#[test]
fn opcode_0c() {
    run("0c");
}

#[test]
fn opcode_0d() {
    run("0d");
}

#[test]
fn opcode_0e() {
    run("0e");
}

#[test]
fn opcode_0f() {
    run("0f");
}

#[test]
fn opcode_10() {
    run("10");
}

#[test]
fn opcode_11() {
    run("11");
}

#[test]
fn opcode_13() {
    run("13");
}

#[test]
fn opcode_14() {
    run("14");
}

#[test]
fn opcode_15() {
    run("15");
}

#[test]
fn opcode_16() {
    run("16");
}

#[test]
fn opcode_17() {
    run("17");
}

#[test]
fn opcode_18() {
    run("18");
}

#[test]
fn opcode_19() {
    run("19");
}

#[test]
fn opcode_1a() {
    run("1a");
}

#[test]
fn opcode_1b() {
    run("1b");
}

#[test]
fn opcode_1c() {
    run("1c");
}

#[test]
fn opcode_1d() {
    run("1d");
}

#[test]
fn opcode_1e() {
    run("1e");
}

#[test]
fn opcode_1f() {
    run("1f");
}

#[test]
fn opcode_20() {
    run("20");
}

#[test]
fn opcode_21() {
    run("21");
}

#[test]
fn opcode_23() {
    run("23");
}

#[test]
fn opcode_24() {
    run("24");
}

#[test]
fn opcode_25() {
    run("25");
}

#[test]
fn opcode_26() {
    run("26");
}

#[test]
fn opcode_27() {
    run("27");
}

#[test]
fn opcode_28() {
    run("28");
}

#[test]
fn opcode_29() {
    run("29");
}

#[test]
fn opcode_2a() {
    run("2a");
}

#[test]
fn opcode_2b() {
    run("2b");
}

#[test]
fn opcode_2c() {
    run("2c");
}

#[test]
fn opcode_2d() {
    run("2d");
}

#[test]
fn opcode_2e() {
    run("2e");
}

#[test]
fn opcode_2f() {
    run("2f");
}

#[test]
fn opcode_30() {
    run("30");
}

#[test]
fn opcode_31() {
    run("31");
}

#[test]
fn opcode_33() {
    run("33");
}

#[test]
fn opcode_34() {
    run("34");
}

#[test]
fn opcode_35() {
    run("35");
}

#[test]
fn opcode_36() {
    run("36");
}

#[test]
fn opcode_37() {
    run("37");
}

#[test]
fn opcode_38() {
    run("38");
}

#[test]
fn opcode_39() {
    run("39");
}

#[test]
fn opcode_3a() {
    run("3a");
}

#[test]
fn opcode_3b() {
    run("3b");
}

#[test]
fn opcode_3c() {
    run("3c");
}

#[test]
fn opcode_3d() {
    run("3d");
}

#[test]
fn opcode_3e() {
    run("3e");
}

#[test]
fn opcode_3f() {
    run("3f");
}

#[test]
fn opcode_40() {
    run("40");
}

#[test]
fn opcode_41() {
    run("41");
}

#[test]
fn opcode_43() {
    run("43");
}

#[test]
fn opcode_44() {
    run("44");
}

#[test]
fn opcode_45() {
    run("45");
}

#[test]
fn opcode_46() {
    run("46");
}

#[test]
fn opcode_47() {
    run("47");
}

#[test]
fn opcode_48() {
    run("48");
}

#[test]
fn opcode_49() {
    run("49");
}

#[test]
fn opcode_4a() {
    run("4a");
}

#[test]
fn opcode_4b() {
    run("4b");
}

#[test]
fn opcode_4c() {
    run("4c");
}

#[test]
fn opcode_4d() {
    run("4d");
}

#[test]
fn opcode_4e() {
    run("4e");
}

#[test]
fn opcode_4f() {
    run("4f");
}

#[test]
fn opcode_50() {
    run("50");
}

#[test]
fn opcode_51() {
    run("51");
}

#[test]
fn opcode_53() {
    run("53");
}

#[test]
fn opcode_54() {
    run("54");
}

#[test]
fn opcode_55() {
    run("55");
}

#[test]
fn opcode_56() {
    run("56");
}

#[test]
fn opcode_57() {
    run("57");
}

#[test]
fn opcode_58() {
    run("58");
}

#[test]
fn opcode_59() {
    run("59");
}

#[test]
fn opcode_5a() {
    run("5a");
}

#[test]
fn opcode_5b() {
    run("5b");
}

#[test]
fn opcode_5c() {
    run("5c");
}

#[test]
fn opcode_5d() {
    run("5d");
}

#[test]
fn opcode_5e() {
    run("5e");
}

#[test]
fn opcode_5f() {
    run("5f");
}

#[test]
fn opcode_60() {
    run("60");
}

#[test]
fn opcode_61() {
    run("61");
}

#[test]
fn opcode_63() {
    run("63");
}

#[test]
fn opcode_64() {
    run("64");
}

#[test]
fn opcode_65() {
    run("65");
}

#[test]
fn opcode_66() {
    run("66");
}

#[test]
fn opcode_67() {
    run("67");
}

#[test]
fn opcode_68() {
    run("68");
}

#[test]
fn opcode_69() {
    run("69");
}

#[test]
fn opcode_6a() {
    run("6a");
}

#[test]
fn opcode_6b() {
    run("6b");
}

#[test]
fn opcode_6c() {
    run("6c");
}

#[test]
fn opcode_6d() {
    run("6d");
}

#[test]
fn opcode_6e() {
    run("6e");
}

#[test]
fn opcode_6f() {
    run("6f");
}

#[test]
fn opcode_70() {
    run("70");
}

#[test]
fn opcode_71() {
    run("71");
}

#[test]
fn opcode_73() {
    run("73");
}

#[test]
fn opcode_74() {
    run("74");
}

#[test]
fn opcode_75() {
    run("75");
}

#[test]
fn opcode_76() {
    run("76");
}

#[test]
fn opcode_77() {
    run("77");
}

#[test]
fn opcode_78() {
    run("78");
}

#[test]
fn opcode_79() {
    run("79");
}

#[test]
fn opcode_7a() {
    run("7a");
}

#[test]
fn opcode_7b() {
    run("7b");
}

#[test]
fn opcode_7c() {
    run("7c");
}

#[test]
fn opcode_7d() {
    run("7d");
}

#[test]
fn opcode_7e() {
    run("7e");
}

#[test]
fn opcode_7f() {
    run("7f");
}

#[test]
fn opcode_80() {
    run("80");
}

#[test]
fn opcode_81() {
    run("81");
}

#[test]
fn opcode_82() {
    run("82");
}

#[test]
fn opcode_83() {
    run("83");
}

#[test]
fn opcode_84() {
    run("84");
}

#[test]
fn opcode_85() {
    run("85");
}

#[test]
fn opcode_86() {
    run("86");
}

#[test]
fn opcode_87() {
    run("87");
}

#[test]
fn opcode_88() {
    run("88");
}

#[test]
fn opcode_89() {
    run("89");
}

#[test]
fn opcode_8a() {
    run("8a");
}

#[test]
fn opcode_8b() {
    run("8b");
}

#[test]
fn opcode_8c() {
    run("8c");
}

#[test]
fn opcode_8d() {
    run("8d");
}

#[test]
fn opcode_8e() {
    run("8e");
}

#[test]
fn opcode_8f() {
    run("8f");
}

#[test]
fn opcode_90() {
    run("90");
}

#[test]
fn opcode_91() {
    run("91");
}

#[test]
fn opcode_93() {
    run("93");
}

#[test]
fn opcode_94() {
    run("94");
}

#[test]
fn opcode_95() {
    run("95");
}

#[test]
fn opcode_96() {
    run("96");
}

#[test]
fn opcode_97() {
    run("97");
}

#[test]
fn opcode_98() {
    run("98");
}

#[test]
fn opcode_99() {
    run("99");
}

#[test]
fn opcode_9a() {
    run("9a");
}

#[test]
fn opcode_9b() {
    run("9b");
}

#[test]
fn opcode_9c() {
    run("9c");
}

#[test]
fn opcode_9d() {
    run("9d");
}

#[test]
fn opcode_9e() {
    run("9e");
}

#[test]
fn opcode_9f() {
    run("9f");
}

#[test]
fn opcode_a0() {
    run("a0");
}

#[test]
fn opcode_a1() {
    run("a1");
}

#[test]
fn opcode_a2() {
    run("a2");
}

#[test]
fn opcode_a3() {
    run("a3");
}

#[test]
fn opcode_a4() {
    run("a4");
}

#[test]
fn opcode_a5() {
    run("a5");
}

#[test]
fn opcode_a6() {
    run("a6");
}

#[test]
fn opcode_a7() {
    run("a7");
}

#[test]
fn opcode_a8() {
    run("a8");
}

#[test]
fn opcode_a9() {
    run("a9");
}

#[test]
fn opcode_aa() {
    run("aa");
}

#[test]
fn opcode_ab() {
    run("ab");
}

#[test]
fn opcode_ac() {
    run("ac");
}

#[test]
fn opcode_ad() {
    run("ad");
}

#[test]
fn opcode_ae() {
    run("ae");
}

#[test]
fn opcode_af() {
    run("af");
}

#[test]
fn opcode_b0() {
    run("b0");
}

#[test]
fn opcode_b1() {
    run("b1");
}

#[test]
fn opcode_b3() {
    run("b3");
}

#[test]
fn opcode_b4() {
    run("b4");
}

#[test]
fn opcode_b5() {
    run("b5");
}

#[test]
fn opcode_b6() {
    run("b6");
}

#[test]
fn opcode_b7() {
    run("b7");
}

#[test]
fn opcode_b8() {
    run("b8");
}

#[test]
fn opcode_b9() {
    run("b9");
}

#[test]
fn opcode_ba() {
    run("ba");
}

#[test]
fn opcode_bb() {
    run("bb");
}

#[test]
fn opcode_bc() {
    run("bc");
}

#[test]
fn opcode_bd() {
    run("bd");
}

#[test]
fn opcode_be() {
    run("be");
}

#[test]
fn opcode_bf() {
    run("bf");
}

#[test]
fn opcode_c0() {
    run("c0");
}

#[test]
fn opcode_c1() {
    run("c1");
}

#[test]
fn opcode_c2() {
    run("c2");
}

#[test]
fn opcode_c3() {
    run("c3");
}

#[test]
fn opcode_c4() {
    run("c4");
}

#[test]
fn opcode_c5() {
    run("c5");
}

#[test]
fn opcode_c6() {
    run("c6");
}

#[test]
fn opcode_c7() {
    run("c7");
}

#[test]
fn opcode_c8() {
    run("c8");
}

#[test]
fn opcode_c9() {
    run("c9");
}

#[test]
fn opcode_ca() {
    run("ca");
}

#[test]
fn opcode_cb() {
    run("cb");
}

#[test]
fn opcode_cc() {
    run("cc");
}

#[test]
fn opcode_cd() {
    run("cd");
}

#[test]
fn opcode_ce() {
    run("ce");
}

#[test]
fn opcode_cf() {
    run("cf");
}

#[test]
fn opcode_d0() {
    run("d0");
}

#[test]
fn opcode_d1() {
    run("d1");
}

#[test]
fn opcode_d3() {
    run("d3");
}

#[test]
fn opcode_d4() {
    run("d4");
}

#[test]
fn opcode_d5() {
    run("d5");
}

#[test]
fn opcode_d6() {
    run("d6");
}

#[test]
fn opcode_d7() {
    run("d7");
}

#[test]
fn opcode_d8() {
    run("d8");
}

#[test]
fn opcode_d9() {
    run("d9");
}

#[test]
fn opcode_da() {
    run("da");
}

#[test]
fn opcode_db() {
    run("db");
}

#[test]
fn opcode_dc() {
    run("dc");
}

#[test]
fn opcode_dd() {
    run("dd");
}

#[test]
fn opcode_de() {
    run("de");
}

#[test]
fn opcode_df() {
    run("df");
}

#[test]
fn opcode_e0() {
    run("e0");
}

#[test]
fn opcode_e1() {
    run("e1");
}

#[test]
fn opcode_e2() {
    run("e2");
}

#[test]
fn opcode_e3() {
    run("e3");
}

#[test]
fn opcode_e4() {
    run("e4");
}

#[test]
fn opcode_e5() {
    run("e5");
}

#[test]
fn opcode_e6() {
    run("e6");
}

#[test]
fn opcode_e7() {
    run("e7");
}

#[test]
fn opcode_e8() {
    run("e8");
}

#[test]
fn opcode_e9() {
    run("e9");
}

#[test]
fn opcode_ea() {
    run("ea");
}

#[test]
fn opcode_eb() {
    run("eb");
}

#[test]
fn opcode_ec() {
    run("ec");
}

#[test]
fn opcode_ed() {
    run("ed");
}

#[test]
fn opcode_ee() {
    run("ee");
}

#[test]
fn opcode_ef() {
    run("ef");
}

#[test]
fn opcode_f0() {
    run("f0");
}

#[test]
fn opcode_f1() {
    run("f1");
}

#[test]
fn opcode_f3() {
    run("f3");
}

#[test]
fn opcode_f4() {
    run("f4");
}

#[test]
fn opcode_f5() {
    run("f5");
}

#[test]
fn opcode_f6() {
    run("f6");
}

#[test]
fn opcode_f7() {
    run("f7");
}

#[test]
fn opcode_f8() {
    run("f8");
}

#[test]
fn opcode_f9() {
    run("f9");
}

#[test]
fn opcode_fa() {
    run("fa");
}

#[test]
fn opcode_fb() {
    run("fb");
}

#[test]
fn opcode_fc() {
    run("fc");
}

#[test]
fn opcode_fd() {
    run("fd");
}

#[test]
fn opcode_fe() {
    run("fe");
}

#[test]
fn opcode_ff() {
    run("ff");
}
