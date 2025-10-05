use super::Opcode;

#[test]
fn decode_recode() {
    for i in 0..u8::MAX {
        let opcode = Opcode::lookup(i);
        let byte = opcode.byte();
        assert_eq!(
            i, byte,
            "testing if an opcode can recover the byte that made it up"
        );
    }
}
