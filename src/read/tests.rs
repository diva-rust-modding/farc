use super::*;
use nom::dbg_dmp;

const INPUT: &[u8] = include_bytes!("../../assets/robmot_PV626.farc");
const COMP: &[u8] = include_bytes!("../../assets/gm_module_tbl.farc");
const FARC: &[u8] = include_bytes!("../../assets/pv_721_common.farc");
const FUTURE: &[u8] = include_bytes!("../../assets/lenitm027.farc");

#[test]
fn read_base() {
    let (_, farc) = BaseArchive::read(INPUT).unwrap();
    let entry = BaseEntry::Memory(MemoryEntry {
        name: "mot_PV626.bin".into(),
        data: INPUT[0x22..][..15305208].into(),
    });
    assert_eq!(entry, farc.entries[0]);
}
#[test]
fn read_compressed() {
    let (_, farc) = CompressArchive::read(COMP).unwrap();
    let entry: Compressor = CompressedEntry {
        entry: MemoryEntry {
            name: "gm_module_id.bin".into(),
            data: COMP[41..][..3827].into(),
        },
        original_len: 21050,
    }
    .into();
    assert_eq!(entry, farc.entries[0]);
}
#[test]
fn read_extended_encrypt_compres() {
    let (_, farc) = ExtendArchive::<Encryptor<Compressor<'_>>>::read(FARC).unwrap();
    for entry in &farc.0.entries {
        println!("{}", &entry.name());
    }
    //pv_721_mouth.dsc
    //pv_721_scene.dsc
    //pv_721_success_mouth.dsc
    //pv_721_success_scene.dsc
    //pv_721_system.dsc
    assert_eq!(farc.0.entries[0].name(), "pv_721_mouth.dsc");
    assert_eq!(farc.0.entries[1].name(), "pv_721_scene.dsc");
    assert_eq!(farc.0.entries[2].name(), "pv_721_success_mouth.dsc");
    assert_eq!(farc.0.entries[3].name(), "pv_721_success_scene.dsc");
    assert_eq!(farc.0.entries[4].name(), "pv_721_system.dsc");
}
#[test]
fn read_future_compressed() {
    let (_, farc) = dbg_dmp(FutureArchive::<CompressedEntry<'_>>::read, "future")(FUTURE).unwrap();
    for entry in &farc.0.entries {
        println!("{} {:#X}", entry.name(), entry.original_len);
    }
    assert_eq!(farc.0.entries[0].name(), "lenitm027_obj.bin");
    assert_eq!(farc.0.entries[1].name(), "lenitm027_tex.bin");
}
