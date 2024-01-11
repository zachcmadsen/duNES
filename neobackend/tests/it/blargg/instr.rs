use crate::blargg::blargg_test;

blargg_test!(basics, "instr_test-v5/01-basics.nes");
blargg_test!(implied, "instr_test-v5/02-implied.nes");
// blargg_test!(immediate, "instr_test-v5/03-immediate.nes");
blargg_test!(zero_page, "instr_test-v5/04-zero_page.nes");
blargg_test!(zp_xy, "instr_test-v5/05-zp_xy.nes");
blargg_test!(absolute, "instr_test-v5/06-absolute.nes");
blargg_test!(abs_xy, "instr_test-v5/07-abs_xy.nes");
blargg_test!(ind_x, "instr_test-v5/08-ind_x.nes");
blargg_test!(ind_y, "instr_test-v5/09-ind_y.nes");
blargg_test!(branches, "instr_test-v5/10-branches.nes");
blargg_test!(stack, "instr_test-v5/11-stack.nes");
blargg_test!(jmp_jsr, "instr_test-v5/12-jmp_jsr.nes");
blargg_test!(rts, "instr_test-v5/13-rts.nes");
blargg_test!(rti, "instr_test-v5/14-rti.nes");
blargg_test!(brk, "instr_test-v5/15-brk.nes");
blargg_test!(special, "instr_test-v5/16-special.nes");
