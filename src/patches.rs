
use rand::{
    rngs::StdRng,
    seq::SliceRandom,
    SeedableRng,
    Rng,
    distributions::{Distribution,Uniform},
};

use encoding::{
    all::WINDOWS_1252,
    Encoding,
    EncoderTrap,
};
use serde::Deserialize;

use crate::{
    custom_asset_ids,
    dol_patcher::DolPatcher,
    ciso_writer::CisoWriter,
    elevators::{ELEVATORS, Elevator, SpawnRoom},
    gcz_writer::GczWriter,
    memmap,
    mlvl_wrapper,
    pickup_meta::{self, PickupType},
    door_meta::{DoorType, DoorLocation, Weights, World},
    reader_writer,
    patcher::{PatcherState, PrimePatcher},
    structs,
    GcDiscLookupExtensions,
    ResourceData,
};

use generated::{mp1_symbol, resource_info, ResourceInfo};
use ppcasm::ppcasm;

use reader_writer::{
    generic_array::GenericArray,
    typenum::U3,
    CStrConversionExtension,
    FourCC,
    LCow,
    Reader,
    Writable,
};

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    ffi::CString,
    fmt,
    fs::File,
    io::Write,
    iter,
    mem,
};

const ARTIFACT_OF_TRUTH_REQ_LAYER: u32 = 24;
const ALWAYS_MODAL_HUDMENUS: &[usize] = &[23, 50, 63];


// When changing a pickup, we need to give the room a copy of the resources/
// assests used by the pickup. Create a cache of all the resources needed by
// any pickup.
fn collect_pickup_resources<'r>(gc_disc: &structs::GcDisc<'r>)
    -> HashMap<(u32, FourCC), structs::Resource<'r>>
{
    // Get list of all dependencies patcher needs //
    let mut looking_for: HashSet<_> = PickupType::iter()
        .flat_map(|pt| pt.dependencies().iter().cloned())
        .chain(PickupType::iter().map(|pt| (pt.hudmemo_strg(), b"STRG".into())))
        .collect();

    // Dependencies read from paks and custom assets will go here //
    let mut found = HashMap::with_capacity(looking_for.len());

    // Remove extra assets from dependency search since they won't appear     //
    // in any pak. Instead add them to the output resource pool. These assets //
    // are provided as external files checked into the repository.            //
    let extra_assets = pickup_meta::extra_assets();
    for res in extra_assets {
        looking_for.remove(&(res.file_id, res.fourcc()));
        assert!(found.insert((res.file_id, res.fourcc()), res.clone()).is_none());
    }

    // Iterate through all paks //
    for pak_name in pickup_meta::PICKUP_LOCATIONS.iter().map(|(name, _)| name) {

        // Get pak //
        let file_entry = gc_disc.find_file(pak_name).unwrap();
        let pak = match *file_entry.file().unwrap() {
            structs::FstEntryFile::Pak(ref pak) => Cow::Borrowed(pak),
            structs::FstEntryFile::Unknown(ref reader) => Cow::Owned(reader.clone().read(())),
            _ => panic!(),
        };

        // Iterate through all resources in pak //
        for res in pak.resources.iter() {
            // If this resource is a dependency needed by the patcher, add the resource to the output list //
            let key = (res.file_id, res.fourcc());
            if looking_for.remove(&key) {
                assert!(found.insert(key, res.into_owned()).is_none());
            }
        }
    }

    // Finally, we need to add the assets which are generated rather than read from a file locally //
    
    // Generate assets for Nothing and Phazon Suit //
    let mut new_assets = vec![];
    new_assets.extend_from_slice(&create_suit_icon_cmdl_and_ancs(
        &found,
        custom_asset_ids::NOTHING_CMDL,
        custom_asset_ids::NOTHING_ANCS,
        custom_asset_ids::NOTHING_TXTR,
        custom_asset_ids::PHAZON_SUIT_TXTR2,
    ));
    new_assets.extend_from_slice(&create_suit_icon_cmdl_and_ancs(
        &found,
        custom_asset_ids::PHAZON_SUIT_CMDL,
        custom_asset_ids::PHAZON_SUIT_ANCS,
        custom_asset_ids::PHAZON_SUIT_TXTR1,
        custom_asset_ids::PHAZON_SUIT_TXTR2,
    ));
    new_assets.extend_from_slice(&create_item_scan_strg_pair(
        custom_asset_ids::PHAZON_SUIT_SCAN,
        custom_asset_ids::PHAZON_SUIT_STRG,
        "Phazon Suit\0",
    ));
    new_assets.extend_from_slice(&create_item_scan_strg_pair(
        custom_asset_ids::NOTHING_SCAN,
        custom_asset_ids::NOTHING_SCAN_STRG,
        "???\0",
    ));
    new_assets.push(pickup_meta::build_resource(
        custom_asset_ids::NOTHING_ACQUIRED_HUDMEMO_STRG,
        structs::ResourceKind::Strg(structs::Strg::from_strings(vec![
            "&just=center;Nothing acquired!\0".to_owned(),
        ])),
    ));
    new_assets.extend_from_slice(&create_item_scan_strg_pair(
        custom_asset_ids::THERMAL_VISOR_SCAN,
        custom_asset_ids::THERMAL_VISOR_STRG,
        "Thermal Visor\0",
    ));
    new_assets.extend_from_slice(&create_item_scan_strg_pair(
        custom_asset_ids::SCAN_VISOR_SCAN,
        custom_asset_ids::SCAN_VISOR_SCAN_STRG,
        "Scan Visor\0",
    ));
    new_assets.push(pickup_meta::build_resource(
        custom_asset_ids::SCAN_VISOR_ACQUIRED_HUDMEMO_STRG,
        structs::ResourceKind::Strg(structs::Strg::from_strings(vec![
            "&just=center;Scan Visor acquired!\0".to_owned(),
        ])),
    ));
    new_assets.extend_from_slice(&create_item_scan_strg_pair(
        custom_asset_ids::SHINY_MISSILE_SCAN,
        custom_asset_ids::SHINY_MISSILE_SCAN_STRG,
        "Shiny Missile\0",
    ));
    new_assets.extend_from_slice(&create_shiny_missile_assets(&found));
    new_assets.push(pickup_meta::build_resource(
        custom_asset_ids::SHINY_MISSILE_ACQUIRED_HUDMEMO_STRG,
        structs::ResourceKind::Strg(structs::Strg::from_strings(vec![
            "&just=center;Shiny Missile acquired!\0".to_owned(),
        ])),
    ));

    // Add the newly generated resources //
    for res in new_assets {
        let key = (res.file_id, res.fourcc());
        if looking_for.remove(&key) {
            assert!(found.insert(key, res).is_none());
        }
    }

    assert!(looking_for.is_empty());

    if !looking_for.is_empty()
    {
        println!("error - still looking for {:?}", looking_for);
    }

    found
}

// Door assets are not shared across all areas either,
// so we have to make a cache for them as well.
fn collect_door_resources<'r>(gc_disc: &structs::GcDisc<'r>)
    -> HashMap<(u32, FourCC), structs::Resource<'r>>
{   
    // Get list of all dependencies needed by custom doors //
    let mut looking_for: HashSet<_> = DoorType::iter()
        .flat_map(|pt| pt.dependencies().into_iter())
        .collect();
    
    // Dependencies read from paks and custom assets will go here //
    let mut found = HashMap::with_capacity(looking_for.len());

    // Remove extra assets from dependency search since they won't appear     //
    // in any pak. Instead add them to the output resource pool. These assets //
    // are provided as external files checked into the repository.            //
    let extra_assets = pickup_meta::extra_assets_doors();
    for res in extra_assets {
        looking_for.remove(&(res.file_id, res.fourcc()));
        assert!(found.insert((res.file_id, res.fourcc()), res.clone()).is_none());
    }

    // Iterate through all paks and add add any dependencies to the resource pool //
    for pak_name in pickup_meta::PICKUP_LOCATIONS.iter().map(|(name, _)| name) { // for all paks

        // get the pak //
        let file_entry = gc_disc.find_file(pak_name).unwrap();
        let pak = match *file_entry.file().unwrap() {
            structs::FstEntryFile::Pak(ref pak) => Cow::Borrowed(pak),
            structs::FstEntryFile::Unknown(ref reader) => Cow::Owned(reader.clone().read(())),
            _ => panic!(),
        };

        // Iterate through all resources in the pak //
        for res in pak.resources.iter() {
            let key = (res.file_id, res.fourcc());
            if looking_for.remove(&key) { // If it's one of our dependencies
                assert!(found.insert(key, res.into_owned()).is_none()); // collect it
            }
        }
    }

    // Generate custom assets (new door variants) //
    let mut new_assets = vec![];

    for door_type in DoorType::iter() {
        if door_type.shield_cmdl() >= 0xDEAF0000 {
            new_assets.push(create_custom_door_cmdl(&found, door_type));
        }
    }

    // Add the newly generated resources //
    for res in new_assets {
        let key = (res.file_id, res.fourcc());
        if looking_for.remove(&key) {
            assert!(found.insert(key, res).is_none());
        }
    }

    if !looking_for.is_empty()
    {
        println!("error - still looking for {:?}", looking_for);
    }

    assert!(looking_for.is_empty());

    found
}

fn create_custom_door_cmdl<'r>(
    resources: &HashMap<(u32, FourCC), structs::Resource<'r>>,
    door_type: DoorType,
) -> structs::Resource<'r>
{
    let new_cmdl_id: u32 = door_type.shield_cmdl();
    let new_txtr_id: u32 = door_type.holorim_texture();

    let new_door_cmdl = {
        // Find and read the blue door CMDL
        let blue_door_cmdl = {
            if door_type.is_vertical() {
                ResourceData::new(&resources[&resource_info!("18D0AEE6.CMDL").into()]) // actually white door but who cares
            } else {
                ResourceData::new(&resources[&resource_info!("blueShield_v1.CMDL").into()])
            }
        };

        // Deserialize the blue door CMDL into a new mutable CMDL
        let blue_door_cmdl_bytes = blue_door_cmdl.decompress().into_owned();
        let mut new_cmdl = Reader::new(&blue_door_cmdl_bytes[..]).read::<structs::Cmdl>(());
        
        // Modify the new CMDL to make it unique
        new_cmdl.material_sets.as_mut_vec()[0].texture_ids.as_mut_vec()[0] = new_txtr_id;
        
        // Re-serialize the CMDL //
        let mut new_cmdl_bytes = vec![];
        new_cmdl.write_to(&mut new_cmdl_bytes).unwrap();

        // Pad length to multiple of 32 bytes //
        let len = new_cmdl_bytes.len();
        new_cmdl_bytes.extend(reader_writer::pad_bytes(32, len).iter());

        // Assemble into a proper resource object
        pickup_meta::build_resource(
            new_cmdl_id, // Custom ids start with 0xDEAFxxxx
            structs::ResourceKind::External(new_cmdl_bytes, b"CMDL".into())
        )
    };
    
    new_door_cmdl
}

fn create_suit_icon_cmdl_and_ancs<'r>(
    resources: &HashMap<(u32, FourCC), structs::Resource<'r>>,
    new_cmdl_id: u32,
    new_ancs_id: u32,
    new_txtr1: u32,
    new_txtr2: u32,
) -> [structs::Resource<'r>; 2]
{
    let new_suit_cmdl = {
        let grav_suit_cmdl = ResourceData::new(
            &resources[&resource_info!("Node1_11.CMDL").into()]
        );
        let cmdl_bytes = grav_suit_cmdl.decompress().into_owned();
        let mut cmdl = Reader::new(&cmdl_bytes[..]).read::<structs::Cmdl>(());

        cmdl.material_sets.as_mut_vec()[0].texture_ids.as_mut_vec()[0] = new_txtr1;
        cmdl.material_sets.as_mut_vec()[0].texture_ids.as_mut_vec()[3] = new_txtr2;

        let mut new_cmdl_bytes = vec![];
        cmdl.write_to(&mut new_cmdl_bytes).unwrap();

        // Ensure the length is a multiple of 32
        let len = new_cmdl_bytes.len();
        new_cmdl_bytes.extend(reader_writer::pad_bytes(32, len).iter());

        pickup_meta::build_resource(
            new_cmdl_id,
            structs::ResourceKind::External(new_cmdl_bytes, b"CMDL".into())
        )
    };
    let new_suit_ancs = {
        let grav_suit_ancs = ResourceData::new(
            &resources[&resource_info!("Node1_11.ANCS").into()]
        );
        let ancs_bytes = grav_suit_ancs.decompress().into_owned();
        let mut ancs = Reader::new(&ancs_bytes[..]).read::<structs::Ancs>(());

        ancs.char_set.char_info.as_mut_vec()[0].cmdl = new_cmdl_id;

        let mut new_ancs_bytes = vec![];
        ancs.write_to(&mut new_ancs_bytes).unwrap();

        // Ensure the length is a multiple of 32
        let len = new_ancs_bytes.len();
        new_ancs_bytes.extend(reader_writer::pad_bytes(32, len).iter());

        pickup_meta::build_resource(
            new_ancs_id,
            structs::ResourceKind::External(new_ancs_bytes, b"ANCS".into())
        )
    };
    [new_suit_cmdl, new_suit_ancs]
}

fn create_shiny_missile_assets<'r>(
    resources: &HashMap<(u32, FourCC), structs::Resource<'r>>,
) -> [structs::Resource<'r>; 4]
{
    let shiny_missile_cmdl = {
        let shiny_missile_cmdl = ResourceData::new(
            &resources[&resource_info!("Node1_36_0.CMDL").into()]
        );
        let cmdl_bytes = shiny_missile_cmdl.decompress().into_owned();
        let mut cmdl = Reader::new(&cmdl_bytes[..]).read::<structs::Cmdl>(());

        // println!("{:#?}", cmdl);
        cmdl.material_sets.as_mut_vec()[0].texture_ids = vec![
            custom_asset_ids::SHINY_MISSILE_TXTR0,
            custom_asset_ids::SHINY_MISSILE_TXTR1,
            custom_asset_ids::SHINY_MISSILE_TXTR2,
        ].into();

        let mut new_cmdl_bytes = vec![];
        cmdl.write_to(&mut new_cmdl_bytes).unwrap();

        // Ensure the length is a multiple of 32
        let len = new_cmdl_bytes.len();
        new_cmdl_bytes.extend(reader_writer::pad_bytes(32, len).iter());

        pickup_meta::build_resource(
            custom_asset_ids::SHINY_MISSILE_CMDL,
            structs::ResourceKind::External(new_cmdl_bytes, b"CMDL".into())
        )
    };
    let shiny_missile_ancs = {
        let shiny_missile_ancs = ResourceData::new(
            &resources[&resource_info!("Node1_37_0.ANCS").into()]
        );
        let ancs_bytes = shiny_missile_ancs.decompress().into_owned();
        let mut ancs = Reader::new(&ancs_bytes[..]).read::<structs::Ancs>(());

        ancs.char_set.char_info.as_mut_vec()[0].cmdl = custom_asset_ids::SHINY_MISSILE_CMDL;
        ancs.char_set.char_info.as_mut_vec()[0].particles.part_assets = vec![
            resource_info!("healthnew.PART").res_id
        ].into();
        if let Some(animation_resources) = &mut ancs.anim_set.animation_resources {
            animation_resources.as_mut_vec()[0].evnt = custom_asset_ids::SHINY_MISSILE_EVNT;
            animation_resources.as_mut_vec()[0].anim = custom_asset_ids::SHINY_MISSILE_ANIM;
        }

        match &mut ancs.anim_set.animations.as_mut_vec()[..] {
            [structs::Animation { meta: structs::MetaAnimation::Play(play), .. }] => {
                play.get_mut().anim = custom_asset_ids::SHINY_MISSILE_ANIM;
            },
            _ => panic!(),
        }

        let mut new_ancs_bytes = vec![];
        ancs.write_to(&mut new_ancs_bytes).unwrap();

        // Ensure the length is a multiple of 32
        let len = new_ancs_bytes.len();
        new_ancs_bytes.extend(reader_writer::pad_bytes(32, len).iter());

        pickup_meta::build_resource(
            custom_asset_ids::SHINY_MISSILE_ANCS,
            structs::ResourceKind::External(new_ancs_bytes, b"ANCS".into())
        )
    };
    let shiny_missile_evnt = {
        let mut evnt = resources[&resource_info!("Missile_Launcher_ready.EVNT").into()]
            .kind.as_evnt()
            .unwrap().into_owned();


        evnt.effect_events.as_mut_vec()[0].effect_file_id = resource_info!("healthnew.PART").res_id;
        evnt.effect_events.as_mut_vec()[1].effect_file_id = resource_info!("healthnew.PART").res_id;

        pickup_meta::build_resource(
            custom_asset_ids::SHINY_MISSILE_EVNT,
            structs::ResourceKind::Evnt(evnt)
        )
    };
    let shiny_missile_anim = {
        let shiny_missile_anim = ResourceData::new(
            &resources[&resource_info!("Missile_Launcher_ready.ANIM").into()]
        );
        let mut anim_bytes = shiny_missile_anim.decompress().into_owned();
        custom_asset_ids::SHINY_MISSILE_EVNT.write_to(&mut std::io::Cursor::new(&mut anim_bytes[8..])).unwrap();
        let len = anim_bytes.len();
        anim_bytes.extend(reader_writer::pad_bytes(32, len).iter());
        pickup_meta::build_resource(
            custom_asset_ids::SHINY_MISSILE_ANIM,
            structs::ResourceKind::External(anim_bytes, b"ANIM".into())
        )
    };
    [shiny_missile_cmdl, shiny_missile_ancs, shiny_missile_evnt, shiny_missile_anim]
}

fn create_item_scan_strg_pair<'r>(
    new_scan: u32,
    new_strg: u32,
    contents: &str,
) -> [structs::Resource<'r>; 2]
{
    let scan = pickup_meta::build_resource(
        new_scan,
        structs::ResourceKind::Scan(structs::Scan {
            frme: 0xFFFFFFFF,
            strg: new_strg,
            scan_speed: 0,
            category: 0,
            icon_flag: 0,
            images: Default::default(),
            padding: [255; 23].into(),
            _dummy: std::marker::PhantomData,
        }),
    );
    let strg = pickup_meta::build_resource(
        new_strg,
        structs::ResourceKind::Strg(structs::Strg::from_strings(vec![contents.to_owned()])),
    );
    [scan, strg]
}


fn artifact_layer_change_template<'r>(instance_id: u32, pickup_kind: u32)
    -> structs::SclyObject<'r>
{
    let layer = if pickup_kind > 29 {
        pickup_kind - 28
    } else {
        assert!(pickup_kind == 29);
        ARTIFACT_OF_TRUTH_REQ_LAYER
    };
    structs::SclyObject {
        instance_id,
        connections: vec![].into(),
        property_data: structs::SclyProperty::SpecialFunction(
            structs::SpecialFunction {
                name: b"Artifact Layer Switch\0".as_cstr(),
                position: [0., 0., 0.].into(),
                rotation: [0., 0., 0.].into(),
                type_: 16,
                unknown0: b"\0".as_cstr(),
                unknown1: 0.,
                unknown2: 0.,
                unknown3: 0.,
                layer_change_room_id: 0xCD2B0EA2,
                layer_change_layer_id: layer,
                item_id: 0,
                unknown4: 1,
                unknown5: 0.,
                unknown6: 0xFFFFFFFF,
                unknown7: 0xFFFFFFFF,
                unknown8: 0xFFFFFFFF,
            }
        ),
    }
}

fn post_pickup_relay_template<'r>(instance_id: u32, connections: &'static [structs::Connection])
    -> structs::SclyObject<'r>
{
    structs::SclyObject {
        instance_id,
        connections: connections.to_owned().into(),
        property_data: structs::SclyProperty::Relay(structs::Relay {
            name: b"Randomizer Post Pickup Relay\0".as_cstr(),
            active: 1,
        })
    }
}

fn add_skip_hudmemos_strgs(pickup_resources: &mut HashMap<(u32, FourCC), structs::Resource>)
{
    for pt in PickupType::iter() {
        let id = pt.skip_hudmemos_strg();
        let res = pickup_meta::build_resource(
            id,
            structs::ResourceKind::Strg(structs::Strg {
                string_tables: vec![
                    structs::StrgStringTable {
                        lang: b"ENGL".into(),
                        strings: vec![format!("&just=center;{} acquired!\u{0}",
                                              pt.name()).into()].into(),
                    },
                ].into(),
            })
        );
        assert!(pickup_resources.insert((id, b"STRG".into()), res).is_none())
    }
}

fn build_artifact_temple_totem_scan_strings<R>(pickup_layout: &[PickupType], rng: &mut R)
    -> [String; 12]
    where R: Rng
{
    let mut generic_text_templates = [
        "I mean, maybe it'll be in the &push;&main-color=#43CD80;{room}&pop;. I forgot, to be honest.\0",
        "I'm not sure where the artifact exactly is, but like, you can try the &push;&main-color=#43CD80;{room}&pop;.\0",
        "Hey man, so some of the Chozo dudes are telling me that they're might be a thing in the &push;&main-color=#43CD80;{room}&pop;. Just sayin'.\0",
        "Uhh umm... Where was it...? Uhhh, errr, it's definitely in the &push;&main-color=#43CD80;{room}&pop;! I am 100% not totally making it up...\0",
        "Some say it may be in the &push;&main-color=#43CD80;{room}&pop;. Others say that you have no business here. Please leave me alone.\0",
        "So a buddy of mine and I were drinking one night and we thought 'Hey, wouldn't be crazy if we put it at the &push;&main-color=#43CD80;{room}&pop;?' So we did and it took both of us just to get it there!\0",
        "So, uhhh, I kind of got a little lazy and I might have just dropped mine somewhere... Maybe it's in the &push;&main-color=#43CD80;{room}&pop;? Who knows.\0",
        "I uhhh... was a little late to the party and someone had to run out and hide both mine and hers. I owe her one. She told me it might be in the &push;&main-color=#43CD80;{room}&pop;, so you're going to have to trust her on this one.\0",
        "Okay, so this jerk forgets to hide his and I had to hide it for him too. So, I just tossed his somewhere and made up a name for the room. This is literally saving the planet - how can anyone forget that? Anyway, mine is in the &push;&main-color=#43CD80;{room}&pop;, so go check it out. I'm never doing this again...\0",
        "To be honest, I don't know if it was a Missile Expansion or not. Maybe it was... We'll just go with that: There's a Missile Expansion at the &push;&main-color=#43CD80;{room}&pop;.\0",
        "Hear the words of Oh Leer, last Chozo of the Artifact Temple. May they serve you well, that you may find a key lost to our cause... Alright, whatever. It's at the &push;&main-color=#43CD80;{room}&pop;.\0",
        "I kind of just played Frisbee with mine. It flew and landed too far so I didn't want to walk over and grab it because I was lazy. It's in the &push;&main-color=#43CD80;{room}&pop; if you want to find it.\0",
    ];
    generic_text_templates.shuffle(rng);
    let mut generic_templates_iter = generic_text_templates.iter();

    // TODO: If there end up being a large number of these, we could use a binary search
    //       instead of searching linearly.
    // XXX It would be nice if we didn't have to use Vec here and could allocated on the stack
    //     instead, but there doesn't seem to be a way to do it that isn't extremely painful or
    //     relies on unsafe code.
    let mut specific_room_templates = [
        // Artifact Temple
        (0x2398E906, vec!["{pickup} awaits those who truly seek it.\0"]),
    ];
    for rt in &mut specific_room_templates {
        rt.1.shuffle(rng);
    }


    let mut scan_text = [
        String::new(), String::new(), String::new(), String::new(),
        String::new(), String::new(), String::new(), String::new(),
        String::new(), String::new(), String::new(), String::new(),
    ];

    let names_iter = pickup_meta::PICKUP_LOCATIONS.iter()
        .flat_map(|i| i.1.iter()) // Flatten out the rooms of the paks
        .flat_map(|l| iter::repeat((l.room_id, l.name)).take(l.pickup_locations.len()));
    let iter = pickup_layout.iter()
        .zip(names_iter)
        // ▼▼▼▼ Only yield artifacts ▼▼▼▼
        .filter(|&(pt, _)| pt.is_artifact());

    // Shame there isn't a way to flatten tuples automatically
    for (pt, (room_id, name)) in iter {
        let artifact_id = pt.idx() - PickupType::ArtifactOfLifegiver.idx();
        if scan_text[artifact_id].len() != 0 {
            // If there are multiple of this particular artifact, then we use the first instance
            // for the location of the artifact.
            continue;
        }

        // If there are specific messages for this room, choose one, other wise choose a generic
        // message.
        let template = specific_room_templates.iter_mut()
            .find(|row| row.0 == room_id)
            .and_then(|row| row.1.pop())
            .unwrap_or_else(|| generic_templates_iter.next().unwrap());
        let pickup_name = pt.name();
        scan_text[artifact_id] = template.replace("{room}", name).replace("{pickup}", pickup_name);
    }

    // Set a default value for any artifacts that we didn't find.
    for i in 0..scan_text.len() {
        if scan_text[i].len() == 0 {
            scan_text[i] = "Artifact not present. This layout may not be completable.\0".to_owned();
        }
    }
    scan_text
}

fn patch_artifact_totem_scan_strg(res: &mut structs::Resource, text: &str)
    -> Result<(), String>
{
    let strg = res.kind.as_strg_mut().unwrap();
    for st in strg.string_tables.as_mut_vec().iter_mut() {
        let strings = st.strings.as_mut_vec();
        *strings.last_mut().unwrap() = text.to_owned().into();
    }
    Ok(())
}

fn patch_save_banner_txtr(res: &mut structs::Resource)
    -> Result<(), String>
{
    const TXTR_BYTES: &[u8] = include_bytes!("../extra_assets/save_banner.txtr");
    res.compressed = false;
    res.kind = structs::ResourceKind::Unknown(Reader::new(TXTR_BYTES), b"TXTR".into());
    Ok(())
}

fn patch_morphball_hud(res: &mut structs::Resource)
    -> Result<(), String>
{
    let frme = res.kind.as_frme_mut().unwrap();
    let widget = frme.widgets.iter_mut()
        .find(|widget| widget.name == b"textpane_bombdigits\0".as_cstr())
        .unwrap();
    // Use the version of Deface18 that has more than just numerical characters for the powerbomb
    // ammo counter
    match &mut widget.kind {
        structs::FrmeWidgetKind::TextPane(textpane) => {
            textpane.font = resource_info!("Deface18B.FONT").res_id;
            textpane.word_wrap = 0;
        }
        _ => panic!("Widget \"textpane_bombdigits\" should be a TXPN"),
    }
    widget.origin[0] -= 0.1;

    // We need to shift all of the widgets in the bomb UI left so there's
    // room for the longer powerbomb ammo counter
    const BOMB_UI_WIDGET_NAMES: &[&[u8]] = &[
        b"model_bar",
        b"model_bombbrak0",
        b"model_bombdrop0",
        b"model_bombbrak1",
        b"model_bombdrop1",
        b"model_bombbrak2",
        b"model_bombdrop2",
        b"model_bombicon",
    ];
    for widget in frme.widgets.iter_mut() {
        if !BOMB_UI_WIDGET_NAMES.contains(&widget.name.to_bytes()) {
            continue;
        }
        widget.origin[0] -= 0.325;
    }
    Ok(())
}

fn patch_mines_savw_for_phazon_suit_scan(res: &mut structs::Resource)
    -> Result<(), String>
{
    // Add a scan for the Phazon suit.
    let savw = res.kind.as_savw_mut().unwrap();
    savw.scan_array.as_mut_vec().push(structs::ScannableObject {
        scan: custom_asset_ids::PHAZON_SUIT_SCAN,
        logbook_category: 0,
    });
    Ok(())
}

#[derive(Copy, Clone, Debug)]
enum MaybeObfuscatedPickup
{
    Unobfuscated(PickupType),
    Obfuscated(PickupType),
}

impl MaybeObfuscatedPickup
{
    fn orig(&self) -> PickupType
    {
        match self {
            MaybeObfuscatedPickup::Unobfuscated(pt) => *pt,
            MaybeObfuscatedPickup::Obfuscated(pt) => *pt,
        }
    }

    // fn name(&self) -> &'static str
    // {
    //     self.orig().name()
    // }

    fn dependencies(&self) -> &'static [(u32, FourCC)]
    {
        match self {
            MaybeObfuscatedPickup::Unobfuscated(pt) => pt.dependencies(),
            MaybeObfuscatedPickup::Obfuscated(_) => PickupType::Nothing.dependencies(),
        }
    }

    fn hudmemo_strg(&self) -> u32
    {
        self.orig().hudmemo_strg()
    }

    fn skip_hudmemos_strg(&self) -> u32
    {
        self.orig().skip_hudmemos_strg()
    }

    pub fn attainment_audio_file_name(&self) -> &'static str
    {
        self.orig().attainment_audio_file_name()
    }

    pub fn pickup_data<'a>(&self) -> LCow<'a, structs::Pickup<'static>>
    {
        match self {
            MaybeObfuscatedPickup::Unobfuscated(pt) => LCow::Borrowed(pt.pickup_data()),
            MaybeObfuscatedPickup::Obfuscated(original) => {
                let original = original.pickup_data();
                let nothing = PickupType::Nothing.pickup_data();

                LCow::Owned(structs::Pickup {
                    name: original.name.clone(),
                    kind: original.kind,
                    max_increase: original.max_increase,
                    curr_increase: original.curr_increase,
                    ..nothing.clone()
                })
            },
        }
    }
}

fn modify_pickups_in_mrea<'r>(
    ps: &mut PatcherState,
    area: &mut mlvl_wrapper::MlvlArea<'r, '_, '_, '_>,
    pickup_type: PickupType,
    pickup_location: pickup_meta::PickupLocation,
    pickup_resources: &HashMap<(u32, FourCC), structs::Resource<'r>>,
    config: &ParsedConfig,
) -> Result<(), String>
{
    let location_idx = 0;

    let pickup_type = if config.obfuscate_items {
        MaybeObfuscatedPickup::Obfuscated(pickup_type)
    } else {
        MaybeObfuscatedPickup::Unobfuscated(pickup_type)
    };

    let deps_iter = pickup_type.dependencies().iter()
        .map(|&(file_id, fourcc)| structs::Dependency {
                asset_id: file_id,
                asset_type: fourcc,
            });

    let name = CString::new(format!(
            "Randomizer - Pickup {} ({:?})", location_idx, pickup_type.pickup_data().name)).unwrap();
    area.add_layer(Cow::Owned(name));

    let new_layer_idx = area.layer_flags.layer_count as usize - 1;

    // Add our custom STRG
    let hudmemo_dep = structs::Dependency {
        asset_id: if config.skip_hudmenus && !ALWAYS_MODAL_HUDMENUS.contains(&location_idx) {
                pickup_type.skip_hudmemos_strg()
            } else {
                pickup_type.hudmemo_strg()
            },
        asset_type: b"STRG".into(),
    };
    let deps_iter = deps_iter.chain(iter::once(hudmemo_dep));
    area.add_dependencies(pickup_resources, new_layer_idx, deps_iter);

    let scly = area.mrea().scly_section_mut();
    let layers = scly.layers.as_mut_vec();

    let mut additional_connections = Vec::new();

    // Add a post-pickup relay. This is used to support cutscene-skipping
    let instance_id = ps.fresh_instance_id_range.next().unwrap();
    let relay = post_pickup_relay_template(instance_id,
                                            pickup_location.post_pickup_relay_connections);
    layers[new_layer_idx].objects.as_mut_vec().push(relay);
    additional_connections.push(structs::Connection {
        state: structs::ConnectionState::ARRIVED,
        message: structs::ConnectionMsg::SET_TO_ZERO,
        target_object_id: instance_id,
    });

    // If this is an artifact, insert a layer change function
    let pickup_kind = pickup_type.pickup_data().kind;
    if pickup_kind >= 29 && pickup_kind <= 40 {
        let instance_id = ps.fresh_instance_id_range.next().unwrap();
        let function = artifact_layer_change_template(instance_id, pickup_kind);
        layers[new_layer_idx].objects.as_mut_vec().push(function);
        additional_connections.push(structs::Connection {
            state: structs::ConnectionState::ARRIVED,
            message: structs::ConnectionMsg::INCREMENT,
            target_object_id: instance_id,
        });
    }

    let pickup = layers[pickup_location.location.layer as usize].objects.iter_mut()
        .find(|obj| obj.instance_id ==  pickup_location.location.instance_id)
        .unwrap();
    update_pickup(pickup, pickup_type);
    if additional_connections.len() > 0 {
        pickup.connections.as_mut_vec().extend_from_slice(&additional_connections);
    }

    let hudmemo = layers[pickup_location.hudmemo.layer as usize].objects.iter_mut()
        .find(|obj| obj.instance_id ==  pickup_location.hudmemo.instance_id)
        .unwrap();
    update_hudmemo(hudmemo, pickup_type, location_idx, config.skip_hudmenus);


    let location = pickup_location.attainment_audio;
    let attainment_audio = layers[location.layer as usize].objects.iter_mut()
        .find(|obj| obj.instance_id ==  location.instance_id)
        .unwrap();
    update_attainment_audio(attainment_audio, pickup_type);
    Ok(())
}

fn update_pickup(pickup: &mut structs::SclyObject, pickup_type: MaybeObfuscatedPickup)
{
    let pickup = pickup.property_data.as_pickup_mut().unwrap();
    let original_pickup = pickup.clone();

    let original_aabb = pickup_meta::aabb_for_pickup_cmdl(original_pickup.cmdl).unwrap();
    let new_aabb = pickup_meta::aabb_for_pickup_cmdl(pickup_type.pickup_data().cmdl).unwrap();
    let original_center = calculate_center(original_aabb, original_pickup.rotation,
                                            original_pickup.scale);
    let new_center = calculate_center(new_aabb, pickup_type.pickup_data().rotation,
                                        pickup_type.pickup_data().scale);

    // The pickup needs to be repositioned so that the center of its model
    // matches the center of the original.
    *pickup = structs::Pickup {
        position: [
            original_pickup.position[0] - (new_center[0] - original_center[0]),
            original_pickup.position[1] - (new_center[1] - original_center[1]),
            original_pickup.position[2] - (new_center[2] - original_center[2]),
        ].into(),
        hitbox: original_pickup.hitbox,
        scan_offset: [
            original_pickup.scan_offset[0] + (new_center[0] - original_center[0]),
            original_pickup.scan_offset[1] + (new_center[1] - original_center[1]),
            original_pickup.scan_offset[2] + (new_center[2] - original_center[2]),
        ].into(),

        fade_in_timer: original_pickup.fade_in_timer,
        spawn_delay: original_pickup.spawn_delay,
        active: original_pickup.active,

        ..(pickup_type.pickup_data().into_owned())
    };
}

fn update_hudmemo(
    hudmemo: &mut structs::SclyObject,
    pickup_type: MaybeObfuscatedPickup,
    location_idx: usize,
    skip_hudmenus: bool)
{
    // The items in Watery Hall (Charge beam), Research Core (Thermal Visor), and Artifact Temple
    // (Artifact of Truth) should always have modal hudmenus because a cutscene plays immediately
    // after each item is acquired, and the nonmodal hudmenu wouldn't properly appear.
    let hudmemo = hudmemo.property_data.as_hud_memo_mut().unwrap();
    if skip_hudmenus && !ALWAYS_MODAL_HUDMENUS.contains(&location_idx) {
        hudmemo.first_message_timer = 5.;
        hudmemo.memo_type = 0;
        hudmemo.strg = pickup_type.skip_hudmemos_strg();
    } else {
        hudmemo.strg = pickup_type.hudmemo_strg();
    }
}

fn update_attainment_audio(attainment_audio: &mut structs::SclyObject,
                           pickup_type: MaybeObfuscatedPickup)
{
    let attainment_audio = attainment_audio.property_data.as_streamed_audio_mut().unwrap();
    let bytes = pickup_type.attainment_audio_file_name().as_bytes();
    attainment_audio.audio_file_name = bytes.as_cstr();
}

fn calculate_center(aabb: [f32; 6], rotation: GenericArray<f32, U3>, scale: GenericArray<f32, U3>)
    -> [f32; 3]
{
    let start = [aabb[0], aabb[1], aabb[2]];
    let end = [aabb[3], aabb[4], aabb[5]];

    let mut position = [0.; 3];
    for i in 0..3 {
        position[i] = (start[i] + end[i]) / 2. * scale[i];
    }

    rotate(position, [rotation[0], rotation[1], rotation[2]], [0.; 3])
}

fn rotate(mut coordinate: [f32; 3], mut rotation: [f32; 3], center: [f32; 3])
    -> [f32; 3]
{
    // Shift to the origin
    for i in 0..3 {
        coordinate[i] -= center[i];
        rotation[i] = rotation[i].to_radians();
    }

    for i in 0..3 {
        let original = coordinate;
        let x = (i + 1) % 3;
        let y = (i + 2) % 3;
        coordinate[x] = original[x] * rotation[i].cos() - original[y] * rotation[i].sin();
        coordinate[y] = original[x] * rotation[i].sin() + original[y] * rotation[i].cos();
    }

    // Shift back to original position
    for i in 0..3 {
        coordinate[i] += center[i];
    }
    coordinate
}


fn make_elevators_patch<'a>(
    patcher: &mut PrimePatcher<'_, 'a>,
    layout: &'a [Elevator],
    dest_names: &Vec<String>,
    auto_enabled_elevators: bool,
    tiny_elvetator_samus: bool,
)
{
    let mut idx = 0;
    for (elv, dest) in ELEVATORS.iter().zip(layout) {
        
        let dest_name = &dest_names[idx];
        idx = idx + 1;

        if elv.pak_name.len() == 0 {
            // Skip destination only elevators
            continue
        }
        patcher.add_scly_patch((elv.pak_name.as_bytes(), elv.mrea), move |ps, area| {
            let scly = area.mrea().scly_section_mut();
            for layer in scly.layers.iter_mut() {
                let obj = layer.objects.iter_mut()
                    .find(|obj| obj.instance_id == elv.scly_id);
                if let Some(obj) = obj {
                    let wt = obj.property_data.as_world_transporter_mut().unwrap();
                    wt.mrea = dest.mrea;
                    wt.mlvl = dest.mlvl;
                    wt.volume = 0; // if we don't turn down the volume of the "wooshing" effect, the player will hear it indefinitely if the destination isn't a WorldTransporter
                    
                    if tiny_elvetator_samus
                    {
                        wt.player_scale = [0.33,0.33,0.33].into();
                    }
                }
            }

            if auto_enabled_elevators {
                // Auto enable the elevator
                let layer = &mut scly.layers.as_mut_vec()[0];
                let mr_id = layer.objects.iter()
                    .find(|obj| obj.property_data.as_memory_relay()
                        .map(|mr| mr.name == b"Memory Relay - dim scan holo\0".as_cstr())
                        .unwrap_or(false)
                    )
                    .map(|mr| mr.instance_id);

                if let Some(mr_id) = mr_id {
                    layer.objects.as_mut_vec().push(structs::SclyObject {
                        instance_id: ps.fresh_instance_id_range.next().unwrap(),
                        property_data: structs::SclyProperty::Timer(structs::Timer {
                            name: b"Auto enable elevator\0".as_cstr(),

                            start_time: 0.001,
                            max_random_add: 0f32,
                            reset_to_zero: 0,
                            start_immediately: 1,
                            active: 1,
                        }),
                        connections: vec![
                            structs::Connection {
                                state: structs::ConnectionState::ZERO,
                                message: structs::ConnectionMsg::ACTIVATE,
                                target_object_id: mr_id,
                            },
                        ].into(),
                    });
                }
            }

            Ok(())
        });

        let room_dest_name = dest_name.replace('\0', "\n");
        let hologram_name = dest_name.replace('\0', " ");
        let control_name = dest_name.replace('\0', " ");
        patcher.add_resource_patch((&[elv.pak_name.as_bytes()], elv.room_strg, b"STRG".into()), move |res| {
            let string = format!("Transport to {}\u{0}", room_dest_name);
            let strg = structs::Strg::from_strings(vec![string]);
            res.kind = structs::ResourceKind::Strg(strg);
            Ok(())
        });
        patcher.add_resource_patch((&[elv.pak_name.as_bytes()], elv.hologram_strg, b"STRG".into()), move |res| {
            let string = format!(
                "Access to &main-color=#FF3333;{} &main-color=#89D6FF;granted. Please step into the hologram.\u{0}",
                hologram_name,
            );
            let strg = structs::Strg::from_strings(vec![string]);
            res.kind = structs::ResourceKind::Strg(strg);
            Ok(())
        });
        patcher.add_resource_patch((&[elv.pak_name.as_bytes()], elv.control_strg, b"STRG".into()), move |res| {
            let string = format!(
                "Transport to &main-color=#FF3333;{}&main-color=#89D6FF; active.\u{0}",
                control_name,
            );
            let strg = structs::Strg::from_strings(vec![string]);
            res.kind = structs::ResourceKind::Strg(strg);
            Ok(())
        });
    }
}

fn patch_landing_site_cutscene_triggers(
    ps: &mut PatcherState,
    area: &mut mlvl_wrapper::MlvlArea,
) -> Result<(), String>
{
    // XXX I'd like to do this some other way than inserting a timer to trigger
    //     the memory relay, but I couldn't figure out how to make the memory
    //     relay default to on/enabled.
    let layer = area.mrea().scly_section_mut().layers.iter_mut().next().unwrap();
    let timer_id = ps.fresh_instance_id_range.next().unwrap();
    for obj in layer.objects.iter_mut() {
        if obj.instance_id == 427 {
            obj.connections.as_mut_vec().push(structs::Connection {
                state: structs::ConnectionState::ACTIVE,
                message: structs::ConnectionMsg::DEACTIVATE,
                target_object_id: timer_id,
            });
        }
        if obj.instance_id == 221 {
            obj.property_data.as_trigger_mut().unwrap().active = 0;
        }
    }
    layer.objects.as_mut_vec().push(structs::SclyObject {
        instance_id: timer_id,
        property_data: structs::SclyProperty::Timer(structs::Timer {
            name: b"Cutscene fixup timer\0".as_cstr(),

            start_time: 0.001,
            max_random_add: 0f32,
            reset_to_zero: 0,
            start_immediately: 1,
            active: 1,
        }),
        connections: vec![
            structs::Connection {
                state: structs::ConnectionState::ZERO,
                message: structs::ConnectionMsg::ACTIVATE,
                target_object_id: 323,// "Memory Relay Set For Load"
            },
            structs::Connection {
                state: structs::ConnectionState::ZERO,
                message: structs::ConnectionMsg::ACTIVATE,
                target_object_id: 427,// "Memory Relay Ship"
            },
            structs::Connection {
                state: structs::ConnectionState::ZERO,
                message: structs::ConnectionMsg::ACTIVATE,
                target_object_id: 484,// "Effect_BaseLights"
            },
            structs::Connection {
                state: structs::ConnectionState::ZERO,
                message: structs::ConnectionMsg::ACTIVATE,
                target_object_id: 463,// "Actor Save Station Beam"
            },
        ].into(),
    });
    Ok(())
}

fn patch_ending_scene_straight_to_credits(
    _ps: &mut PatcherState,
    area: &mut mlvl_wrapper::MlvlArea,
) -> Result<(), String>
{
    let layer = area.mrea().scly_section_mut().layers.iter_mut().next().unwrap();
    let trigger = layer.objects.iter_mut()
        .find(|obj| obj.instance_id == 1103) // "Trigger - Start this Beatch"
        .unwrap();
    trigger.connections.as_mut_vec().push(structs::Connection {
        state: structs::ConnectionState::ENTERED,
        message: structs::ConnectionMsg::ACTION,
        target_object_id: 1241, // "SpecialFunction-edngame"
    });
    Ok(())
}


fn patch_frigate_teleporter<'r>(area: &mut mlvl_wrapper::MlvlArea, spawn_room: SpawnRoom)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let wt = scly.layers.iter_mut()
        .flat_map(|layer| layer.objects.iter_mut())
        .find(|obj| obj.property_data.is_world_transporter())
        .and_then(|obj| obj.property_data.as_world_transporter_mut())
        .unwrap();
    wt.mlvl = spawn_room.mlvl;
    wt.mrea = spawn_room.mrea;
    Ok(())
}

fn calculate_door_type(pak_name: &str, mut rng: &mut StdRng, weights: &Weights) -> DoorType {
    let range = Uniform::from(0..100);
    let weights : &[u8;4] = match pak_name {
        "Metroid2.pak" => &weights.chozo_ruins,
        "Metroid3.pak" => &weights.phendrana_drifts,
        "Metroid4.pak" => &weights.tallon_overworld,
        "metroid5.pak" => &weights.phazon_mines,
        "Metroid6.pak" => &weights.magmoor_caverns,
        "Metroid7.pak" => &[0,0,0,100],
        _ => &[100,0,0,0]
    };
    if weights[0]+weights[1]+weights[2]+weights[3] != 100 { panic!("The sum of all weights for each area must equal exactly 100.") }
    let num:u8 = range.sample(&mut rng);
    if num < weights[0] { DoorType::Blue }
    else if num < (weights[1]+weights[0]) { DoorType::Purple }
    else if num < (weights[2]+weights[1]+weights[0]) { DoorType::White }
    else if num < (weights[3]+weights[2]+weights[1]+weights[0]) { DoorType::Red }
    else {
        panic!("RNG outside the range 0-99.")
    }
}

fn patch_door<'r>(
    area: &mut mlvl_wrapper::MlvlArea<'r, '_, '_, '_>,
    door_loc: DoorLocation,
    door_type: DoorType,
    door_resources:&HashMap<(u32, FourCC), structs::Resource<'r>>,
    lockpick: bool,
) -> Result<(), String> {

    let deps = door_type.dependencies();
    let deps_iter = deps.iter()
        .map(|&(file_id, fourcc)| structs::Dependency {
                asset_id: file_id,
                asset_type: fourcc,
        });

    area.add_dependencies(&door_resources,0,deps_iter);

    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[0];

    let door_force = layer.objects.iter_mut()
        .find(|obj| obj.instance_id == door_loc.door_force_location.instance_id)
        .and_then(|obj| obj.property_data.as_damageable_trigger_mut())
        .unwrap();
    door_force.color_txtr = door_type.forcefield_txtr();
    door_force.damage_vulnerability = door_type.vulnerability();

    if lockpick {
        door_force.damage_vulnerability.power_bomb = 0x1 as u32;
    }

    if door_loc.door_shield_location.is_some() {
        let door_shield = layer.objects.iter_mut()
            .find(|obj| obj.instance_id == door_loc.door_shield_location.unwrap().instance_id)
            .and_then(|obj| obj.property_data.as_actor_mut())
            .unwrap();
        door_shield.cmdl = door_type.shield_cmdl();
    }

    Ok(())
}

fn patch_map_door_icon(
    res: &mut structs::Resource,
    door: DoorLocation,
    door_type: DoorType,
) -> Result<(), String>
{
    let mapa = res.kind.as_mapa_mut().unwrap();

    let door_icon = mapa.objects.iter_mut()
        .find(|obj| obj.editor_id == door.door_location.instance_id)
        .unwrap();
    
    if !door_icon.is_vertical() {
        door_icon.type_ = door_type.map_object_type();
    };

    Ok(())
}

fn fix_artifact_of_truth_requirements(
    ps: &mut PatcherState,
    area: &mut mlvl_wrapper::MlvlArea,
    pickup_layout: &[PickupType],
) -> Result<(), String>
{
    let truth_req_layer_id = area.layer_flags.layer_count;
    assert_eq!(truth_req_layer_id, ARTIFACT_OF_TRUTH_REQ_LAYER);

    // Create a new layer that will be toggled on when the Artifact of Truth is collected
    area.add_layer(b"Randomizer - Got Artifact 1\0".as_cstr());

    let at_pickup_kind = pickup_layout[63].pickup_data().kind;
    for i in 0..12 {
        let layer_number = if i == 0 {
            truth_req_layer_id
        } else {
            i + 1
        };
        let kind = i + 29;
        let exists = pickup_layout.iter()
            .any(|pt| kind == pt.pickup_data().kind);
        if exists && at_pickup_kind != kind {
            // If the artifact exsts, but is not the artifact at the Artifact Temple, mark this
            // layer as inactive. It will be activated when the item is collected.
            area.layer_flags.flags &= !(1 << layer_number);
        } else {
            // Either the artifact doesn't exist or it does and it is in the Artifact Temple, so
            // mark this layer as active. In the former case, it needs to always be active since it
            // will never be collect and in the latter case it needs to be active so the Ridley
            // fight can start immediately if its the last artifact collected.
            area.layer_flags.flags |= 1 << layer_number;
        }
    }

    let scly = area.mrea().scly_section_mut();

    // A relay on the new layer is created and connected to "Relay Show Progress 1"
    let new_relay_instance_id = ps.fresh_instance_id_range.next().unwrap();
    let new_relay = structs::SclyObject {
        instance_id: new_relay_instance_id,
        connections: vec![
            structs::Connection {
                state: structs::ConnectionState::ZERO,
                message: structs::ConnectionMsg::SET_TO_ZERO,
                target_object_id: 1048869,
            },
        ].into(),
        property_data: structs::SclyProperty::Relay(structs::Relay {
            name: b"Relay Show Progress1\0".as_cstr(),
            active: 1,
        }),
    };
    scly.layers.as_mut_vec()[truth_req_layer_id as usize].objects.as_mut_vec().push(new_relay);

    // An existing relay is disconnected from "Relay Show Progress 1" and connected
    // to the new relay
    let relay = scly.layers.as_mut_vec()[1].objects.iter_mut()
        .find(|i| i.instance_id == 68158836).unwrap();
    relay.connections.as_mut_vec().retain(|i| i.target_object_id != 1048869);
    relay.connections.as_mut_vec().push(structs::Connection {
        state: structs::ConnectionState::ZERO,
        message: structs::ConnectionMsg::SET_TO_ZERO,
        target_object_id: new_relay_instance_id,
    });
    Ok(())
}

fn patch_artifact_hint_availability(
    _ps: &mut PatcherState,
    area: &mut mlvl_wrapper::MlvlArea,
    hint_behavior: ArtifactHintBehavior,
) -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    const HINT_RELAY_OBJS: &[u32] = &[
        68157732,
        68157735,
        68157738,
        68157741,
        68157744,
        68157747,
        68157750,
        68157753,
        68157756,
        68157759,
        68157762,
        68157765,
    ];
    match hint_behavior {
        ArtifactHintBehavior::Default => (),
        ArtifactHintBehavior::All => {
            // Unconditionaly connect the hint relays directly to the relay that triggers the hints
            // to conditionally.
            let obj = scly.layers.as_mut_vec()[0].objects.iter_mut()
                .find(|obj| obj.instance_id == 1048956) // "Relay One Shot Out"
                .unwrap();
            obj.connections.as_mut_vec().extend(HINT_RELAY_OBJS.iter().map(|id| {
                structs::Connection {
                    state: structs::ConnectionState::ZERO,
                    message: structs::ConnectionMsg::SET_TO_ZERO,
                    target_object_id: *id,
                }
            }));
        },
        ArtifactHintBehavior::None => {
            // Remove relays that activate artifact hint objects
            scly.layers.as_mut_vec()[1].objects.as_mut_vec()
                .retain(|obj| !HINT_RELAY_OBJS.contains(&obj.instance_id));
        },
    }
    Ok(())
}

fn patch_sun_tower_prevent_wild_before_flaahgra(
    _ps: &mut PatcherState,
    area: &mut mlvl_wrapper::MlvlArea
) -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let idx = scly.layers.as_mut_vec()[0].objects.iter_mut()
        .position(|obj| obj.instance_id == 0x001d015b)
        .unwrap();
    let sunchamber_layer_change_trigger = scly.layers.as_mut_vec()[0].objects.as_mut_vec().remove(idx);
    *scly.layers.as_mut_vec()[1].objects.as_mut_vec() = vec![sunchamber_layer_change_trigger];
    Ok(())
}


fn patch_sunchamber_prevent_wild_before_flaahgra(
    ps: &mut PatcherState,
    area: &mut mlvl_wrapper::MlvlArea
) -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let enable_sun_tower_layer_id = ps.fresh_instance_id_range.next().unwrap();
    scly.layers.as_mut_vec()[1].objects.as_mut_vec().push(structs::SclyObject {
        instance_id: enable_sun_tower_layer_id,
        connections: vec![].into(),
        property_data: structs::SclyProperty::SpecialFunction(
            structs::SpecialFunction {
                name: b"Enable Sun Tower Layer Change Trigger\0".as_cstr(),
                position: [0., 0., 0.].into(),
                rotation: [0., 0., 0.].into(),
                type_: 16,
                unknown0: b"\0".as_cstr(),
                unknown1: 0.,
                unknown2: 0.,
                unknown3: 0.,
                layer_change_room_id: 0xcf4c7aa5,
                layer_change_layer_id: 1,
                item_id: 0,
                unknown4: 1,
                unknown5: 0.,
                unknown6: 0xFFFFFFFF,
                unknown7: 0xFFFFFFFF,
                unknown8: 0xFFFFFFFF,
            }
        ),
    });
    let flaahgra_dead_relay = scly.layers.as_mut_vec()[1].objects.iter_mut()
        .find(|obj| obj.instance_id == 0x42500D4)
        .unwrap();
    flaahgra_dead_relay.connections.as_mut_vec().push(structs::Connection {
        state: structs::ConnectionState::ZERO,
        message: structs::ConnectionMsg::INCREMENT,
        target_object_id: enable_sun_tower_layer_id,
    });

    Ok(())
}

fn patch_temple_security_station_cutscene_trigger(_ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let trigger = scly.layers.iter_mut()
        .flat_map(|layer| layer.objects.iter_mut())
        .find(|obj| obj.instance_id == 0x70067)
        .and_then(|obj| obj.property_data.as_trigger_mut())
        .unwrap();
    trigger.active = 0;

    Ok(())
}

fn patch_ridley_phendrana_shorelines_cinematic(_ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    scly.layers.as_mut_vec()[4].objects.as_mut_vec().clear();
    Ok(())
}

fn make_elite_research_fight_prereq_patches(patcher: &mut PrimePatcher)
{
    patcher.add_scly_patch(resource_info!("03_mines.MREA").into(), |_ps, area| {
        let flags = &mut area.layer_flags.flags;
        *flags |= 1 << 1; // Turn on "3rd pass elite bustout"
        *flags &= !(1 << 5); // Turn off the "dummy elite"
        Ok(())
    });

    patcher.add_scly_patch(resource_info!("07_mines_electric.MREA").into(), |_ps, area| {
        let scly = area.mrea().scly_section_mut();
        scly.layers.as_mut_vec()[0].objects.as_mut_vec()
            .retain(|obj| obj.instance_id != 0x1B0525 && obj.instance_id != 0x1B0522);
        Ok(())
    });
}

fn patch_research_lab_hydra_barrier<'r>(_ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[3];

    let obj = layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == 202965810)
        .unwrap();
    let actor = obj.property_data.as_actor_mut().unwrap();
    actor.actor_params.visor_params.target_passthrough = 1;
    Ok(())
}

fn patch_research_lab_aether_exploding_wall<'r>(
    ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea
)
    -> Result<(), String>
{
    // The room we're actually patching is Research Core..
    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[0];

    let id = ps.fresh_instance_id_range.next().unwrap();
    let obj = layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == 2622568)
        .unwrap();
    obj.connections.as_mut_vec().push(structs::Connection {
        state: structs::ConnectionState::ZERO,
        message: structs::ConnectionMsg::DECREMENT,
        target_object_id: id,
    });

    layer.objects.as_mut_vec().push(structs::SclyObject {
        instance_id: id,
        property_data: structs::SclyProperty:: SpecialFunction(structs::SpecialFunction {
                name: b"SpecialFunction - Remove Research Lab Aether wall\0".as_cstr(),
                position: [0., 0., 0.].into(),
                rotation: [0., 0., 0.].into(),
                type_: 16,
                unknown0: b"\0".as_cstr(),
                unknown1: 0.0,
                unknown2: 0.0,
                unknown3: 0.0,
                layer_change_room_id: 0x354889CE,
                layer_change_layer_id: 3,
                item_id: 0,
                unknown4: 1,
                unknown5: 0.0,
                unknown6: 0xFFFFFFFF,
                unknown7: 0xFFFFFFFF,
                unknown8: 0xFFFFFFFF
            }
        ),
        connections: vec![].into(),
    });
    Ok(())
}

fn patch_observatory_2nd_pass_solvablility<'r>(_ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[2];

    let iter = layer.objects.as_mut_vec().iter_mut()
        .filter(|obj| obj.instance_id == 0x81E0460 || obj.instance_id == 0x81E0461);
    for obj in iter {
        obj.connections.as_mut_vec().push(structs::Connection {
            state: structs::ConnectionState::DEATH_RATTLE,
            message: structs::ConnectionMsg::INCREMENT,
            target_object_id: 0x1E02EA,// Counter - dead pirates active panel
        });
    }

    Ok(())
}

fn patch_main_ventilation_shaft_section_b_door<'r>(
    ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea
)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[0];

    layer.objects.as_mut_vec().push(structs::SclyObject {
        instance_id: ps.fresh_instance_id_range.next().unwrap(),
        property_data: structs::SclyProperty::Trigger(structs::Trigger {
                name: b"Trigger_DoorOpen-component\0".as_cstr(),
                position: [31.232622, 442.69165, -64.20529].into(),
                scale: [6.0, 17.0, 6.0].into(),
                damage_info: structs::structs::DamageInfo {
                    weapon_type: 0,
                    damage: 0.0,
                    radius: 0.0,
                    knockback_power: 0.0
                },
                unknown0: [0.0, 0.0, 0.0].into(),
                unknown1: 1,
                active: 1,
                unknown2: 0,
                unknown3: 0
            }),
        connections: vec![
            structs::Connection {
                state: structs::ConnectionState::INSIDE,
                message: structs::ConnectionMsg::SET_TO_ZERO,
                target_object_id: 1376367,
            },
        ].into(),
    });
    Ok(())
}

fn make_main_plaza_locked_door_two_ways(
    _ps: &mut PatcherState,
    area: &mut mlvl_wrapper::MlvlArea,
    door_type: DoorType,
    config: &ParsedConfig
) -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[0];

    let trigger_dooropen_id = 0x20007;
    let timer_doorclose_id = 0x20008;
    let actor_doorshield_id = 0x20004;
    let relay_unlock_id = 0x20159;
    let trigger_doorunlock_id = 0x2000F;
    let door_id = 0x20060;
    let trigger_remove_scan_target_locked_door_id = 0x202B8;
    let scan_target_locked_door_id = 0x202F4;
    let relay_notice_ineffective_weapon_id = 0x202FD;

    layer.objects.as_mut_vec().extend_from_slice(&[
        structs::SclyObject {
            instance_id: trigger_doorunlock_id,
            property_data: structs::SclyProperty::DamageableTrigger(structs::DamageableTrigger {
                    name: b"Trigger_DoorUnlock\0".as_cstr(),
                    position: [152.232117, 86.451134, 24.472418].into(),
                    scale: [0.25, 4.5, 4.0].into(),
                    health_info: structs::structs::HealthInfo {
                        health: 1.0,
                        knockback_resistance: 1.0
                    },
                    damage_vulnerability: structs::structs::DamageVulnerability {
                        power: 1,           // Normal
                        ice: 1,             // Normal
                        wave: 1,            // Normal
                        plasma: 1,          // Normal
                        bomb: 1,            // Normal
                        power_bomb: 1,      // Normal
                        missile: 2,         // Reflect
                        boost_ball: 2,      // Reflect
                        phazon: 1,          // Normal
                        enemy_weapon0: 3,   // Immune
                        enemy_weapon1: 2,   // Reflect
                        enemy_weapon2: 2,   // Reflect
                        enemy_weapon3: 2,   // Reflect
                        unknown_weapon0: 2, // Reflect
                        unknown_weapon1: 2, // Reflect
                        unknown_weapon2: 1, // Normal
                        charged_beams: structs::structs::ChargedBeams {
                            power: 1,       // Normal
                            ice: 1,         // Normal
                            wave: 1,        // Normal
                            plasma: 1,      // Normal
                            phazon: 1       // Normal
                        },
                        beam_combos: structs::structs::BeamCombos {
                            power: 2,       // Reflect
                            ice: 2,         // Reflect
                            wave: 2,        // Reflect
                            plasma: 2,      // Reflect
                            phazon: 1       // Normal
                        }
                    },
                    unknown0: 3, // Render Side : East
                    pattern_txtr0: 0x544A9892, // testb.TXTR
                    pattern_txtr1: 0x544A9892, // testb.TXTR
                    color_txtr: 0x8A7F3683, // blue.TXTR
                    lock_on: 0,
                    active: 1,
                    visor_params: structs::structs::VisorParameters {
                        unknown0: 0,
                        target_passthrough: 0,
                        unknown2: 15 // Visor Flags : Combat|Scan|Thermal|XRay
                    }
                }),
                connections: vec![
                    structs::Connection {
                        state: structs::ConnectionState::REFLECTED_DAMAGE,
                        message: structs::ConnectionMsg::SET_TO_ZERO,
                        target_object_id: relay_notice_ineffective_weapon_id,
                    },
                    structs::Connection {
                        state: structs::ConnectionState::DEAD,
                        message: structs::ConnectionMsg::DEACTIVATE,
                        target_object_id: actor_doorshield_id,
                    },
                    structs::Connection {
                        state: structs::ConnectionState::MAX_REACHED,
                        message: structs::ConnectionMsg::ACTIVATE,
                        target_object_id: actor_doorshield_id,
                    },
                    structs::Connection {
                        state: structs::ConnectionState::DEAD,
                        message: structs::ConnectionMsg::ACTIVATE,
                        target_object_id: trigger_dooropen_id,
                    },
                    structs::Connection {
                        state: structs::ConnectionState::DEAD,
                        message: structs::ConnectionMsg::SET_TO_ZERO,
                        target_object_id: door_id,
                    },
                ].into(),
        },

        structs::SclyObject {
            instance_id: relay_unlock_id,
            property_data: structs::SclyProperty::Relay(structs::Relay {
                    name: b"Relay_Unlock\0".as_cstr(),
                    active: 1,
                }),
                connections: vec![
                    structs::Connection {
                        state: structs::ConnectionState::ZERO,
                        message: structs::ConnectionMsg::ACTIVATE,
                        target_object_id: actor_doorshield_id,
                    },
                    structs::Connection {
                        state: structs::ConnectionState::ZERO,
                        message: structs::ConnectionMsg::ACTIVATE,
                        target_object_id: trigger_doorunlock_id,
                    },
                ].into(),
        },

        structs::SclyObject {
            instance_id: trigger_dooropen_id,
            property_data: structs::SclyProperty::Trigger(structs::Trigger {
                    name: b"Trigger_DoorOpen\0".as_cstr(),
                    position: [149.35614, 86.567917, 26.471249].into(),
                    scale: [5.0, 5.0, 8.0].into(),
                    damage_info: structs::structs::DamageInfo {
                        weapon_type: 0,
                        damage: 0.0,
                        radius: 0.0,
                        knockback_power: 0.0
                    },
                    unknown0: [0.0, 0.0, 0.0].into(),
                    unknown1: 1,
                    active: 0,
                    unknown2: 0,
                    unknown3: 0
                }),
            connections: vec![
                structs::Connection {
                    state: structs::ConnectionState::INSIDE,
                    message: structs::ConnectionMsg::OPEN,
                    target_object_id: door_id,
                },
                structs::Connection {
                    state: structs::ConnectionState::INSIDE,
                    message: structs::ConnectionMsg::RESET_AND_START,
                    target_object_id: timer_doorclose_id,
                },
            ].into(),
        },

        structs::SclyObject {
            instance_id: actor_doorshield_id,
            property_data: structs::SclyProperty::Actor(structs::Actor {
                    name: b"Actor_DoorShield\0".as_cstr(),
                    position: [151.951187, 86.412575, 24.403177].into(),
                    rotation: [0.0, 0.0, 0.0].into(),
                    scale: [1.0, 1.0, 1.0].into(),
                    unknown0: [0.0, 0.0, 0.0].into(),
                    scan_offset: [0.0, 0.0, 0.0].into(),
                    unknown1: 1.0,
                    unknown2: 0.0,
                    health_info: structs::structs::HealthInfo {
                        health: 5.0,
                        knockback_resistance: 1.0
                    },
                    damage_vulnerability: structs::structs::DamageVulnerability {
                        power: 1,           // Normal
                        ice: 1,             // Normal
                        wave: 1,            // Normal
                        plasma: 1,          // Normal
                        bomb: 1,            // Normal
                        power_bomb: 1,      // Normal
                        missile: 1,         // Normal
                        boost_ball: 1,      // Normal
                        phazon: 1,          // Normal
                        enemy_weapon0: 2,   // Reflect
                        enemy_weapon1: 2,   // Reflect
                        enemy_weapon2: 2,   // Reflect
                        enemy_weapon3: 2,   // Reflect
                        unknown_weapon0: 2, // Reflect
                        unknown_weapon1: 2, // Reflect
                        unknown_weapon2: 0, // Double Damage
                        charged_beams: structs::structs::ChargedBeams {
                            power: 1,       // Normal
                            ice: 1,         // Normal
                            wave: 1,        // Normal
                            plasma: 1,      // Normal
                            phazon: 0       // Double Damage
                        },
                        beam_combos: structs::structs::BeamCombos {
                            power: 1,       // Normal
                            ice: 1,         // Normal
                            wave: 1,        // Normal
                            plasma: 1,      // Normal
                            phazon: 0       // Double Damage
                        }
                    },
                    cmdl: 0x0734977A, // blueShield_v1.CMDL
                    ancs: structs::structs::AncsProp {
                        file_id: 0xFFFFFFFF, // None
                        node_index: 0,
                        unknown: 0xFFFFFFFF, // -1
                    },
                    actor_params: structs::structs::ActorParameters {
                        light_params: structs::structs::LightParameters {
                            unknown0: 1,
                            unknown1: 1.0,
                            shadow_tessellation: 0,
                            unknown2: 1.0,
                            unknown3: 20.0,
                            color: [1.0, 1.0, 1.0, 1.0].into(),
                            unknown4: 1,
                            world_lighting: 1,
                            light_recalculation: 1,
                            unknown5: [0.0, 0.0, 0.0].into(),
                            unknown6: 4,
                            unknown7: 4,
                            unknown8: 0,
                            light_layer_id: 0
                        },
                        scan_params: structs::structs::ScannableParameters {
                            scan: 0xFFFFFFFF // None
                        },
                        xray_cmdl: 0xFFFFFFFF, // None
                        xray_cskr: 0xFFFFFFFF, // None
                        thermal_cmdl: 0xFFFFFFFF, // None
                        thermal_cskr: 0xFFFFFFFF, // None

                        unknown0: 1,
                        unknown1: 1.0,
                        unknown2: 1.0,

                        visor_params: structs::structs::VisorParameters {
                            unknown0: 0,
                            target_passthrough: 0,
                            unknown2: 15 // Visor Flags : Combat|Scan|Thermal|XRay
                        },
                        enable_thermal_heat: 1,
                        unknown3: 0,
                        unknown4: 1,
                        unknown5: 1.0
                    },
                    looping: 1,
                    snow: 1,
                    solid: 0,
                    camera_passthrough: 0,
                    active: 1,
                    unknown8: 0,
                    unknown9: 1.0,
                    unknown10: 1,
                    unknown11: 0,
                    unknown12: 0,
                    unknown13: 0
                }),
                connections: vec![].into()
        },

        structs::SclyObject {
            instance_id: timer_doorclose_id,
            property_data: structs::SclyProperty::Timer(structs::Timer {
                    name: b"Timer_DoorClose\0".as_cstr(),
                    start_time: 0.25,
                    max_random_add: 0.0,
                    reset_to_zero: 1,
                    start_immediately: 0,
                    active: 1
                }),
            connections: vec![
                structs::Connection {
                    state: structs::ConnectionState::ZERO,
                    message: structs::ConnectionMsg::CLOSE,
                    target_object_id: door_id,
                },
                structs::Connection {
                    state: structs::ConnectionState::ZERO,
                    message: structs::ConnectionMsg::DEACTIVATE,
                    target_object_id: trigger_dooropen_id,
                },
            ].into(),
        },
    ]);

    let locked_door_scan = layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == scan_target_locked_door_id)
        .and_then(|obj| obj.property_data.as_point_of_interest_mut())
        .unwrap();
    locked_door_scan.active = 0;
    locked_door_scan.scan_param.scan = 0xFFFFFFFF; // None

    let locked_door = layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == door_id)
        .and_then(|obj| obj.property_data.as_door_mut())
        .unwrap();
    locked_door.ancs.file_id = 0x26886945; // newmetroiddoor.ANCS
    locked_door.ancs.unknown = 2;
    locked_door.projectiles_collide = 0;

    if config.excluded_doors[World::ChozoRuins as usize]["Main Plaza"][4] == "default" {
        let door_force = layer.objects.as_mut_vec().iter_mut()
            .find(|obj| obj.instance_id == trigger_doorunlock_id)
            .and_then(|obj| obj.property_data.as_damageable_trigger_mut())
            .unwrap();
        door_force.color_txtr = door_type.forcefield_txtr();

        door_force.damage_vulnerability = door_type.vulnerability();

        if door_type!= DoorType::Blue && !config.powerbomb_lockpick {
            door_force.damage_vulnerability.power_bomb = 2;
        } else {
            door_force.damage_vulnerability.power_bomb = 1;
        }

        let door_shield = layer.objects.as_mut_vec().iter_mut()
            .find(|obj| obj.instance_id == actor_doorshield_id)
            .and_then(|obj| obj.property_data.as_actor_mut())
            .unwrap();
        door_shield.cmdl = door_type.shield_cmdl();
    }

    let trigger_remove_scan_target_locked_door_and_etank = layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == trigger_remove_scan_target_locked_door_id)
        .and_then(|obj| obj.property_data.as_trigger_mut())
        .unwrap();
    trigger_remove_scan_target_locked_door_and_etank.active = 0;

    layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == door_id)
        .unwrap()
        .connections
        .as_mut_vec()
        .extend_from_slice(
            &[
                structs::Connection {
                    state: structs::ConnectionState::OPEN,
                    message: structs::ConnectionMsg::ACTIVATE,
                    target_object_id: trigger_dooropen_id,
                },
                structs::Connection {
                    state: structs::ConnectionState::OPEN,
                    message: structs::ConnectionMsg::START,
                    target_object_id: timer_doorclose_id,
                },
                structs::Connection {
                    state: structs::ConnectionState::CLOSED,
                    message: structs::ConnectionMsg::DEACTIVATE,
                    target_object_id: trigger_dooropen_id,
                },
                structs::Connection {
                    state: structs::ConnectionState::OPEN,
                    message: structs::ConnectionMsg::DEACTIVATE,
                    target_object_id: trigger_doorunlock_id,
                },
                structs::Connection {
                    state: structs::ConnectionState::OPEN,
                    message: structs::ConnectionMsg::DEACTIVATE,
                    target_object_id: actor_doorshield_id,
                },
                structs::Connection {
                    state: structs::ConnectionState::CLOSED,
                    message: structs::ConnectionMsg::SET_TO_ZERO,
                    target_object_id: relay_unlock_id,
                },
                structs::Connection {
                    state: structs::ConnectionState::MAX_REACHED,
                    message: structs::ConnectionMsg::DEACTIVATE,
                    target_object_id: actor_doorshield_id,
                },
                structs::Connection {
                    state: structs::ConnectionState::MAX_REACHED,
                    message: structs::ConnectionMsg::DEACTIVATE,
                    target_object_id: trigger_doorunlock_id,
                },
            ]
        );

    Ok(())
}

fn patch_main_plaza_locked_door_map_icon(res: &mut structs::Resource,door_type:DoorType)
    -> Result<(),String> {
    let mapa = res.kind.as_mapa_mut().unwrap();

    let door_icon = mapa.objects.iter_mut()
    .find(|obj| obj.editor_id == 0x20060)
    .unwrap();
    
    door_icon.type_ = door_type.map_object_type();

    Ok(())
}

fn patch_main_quarry_door_lock_0_02<'r>(_ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[0];
    layer.objects.as_mut_vec().retain(|obj| obj.instance_id != 132563);
    Ok(())
}

fn patch_geothermal_core_door_lock_0_02<'r>(_ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[0];
    layer.objects.as_mut_vec().retain(|obj| obj.instance_id != 1311646);
    Ok(())
}

fn patch_hive_totem_boss_trigger_0_02(_ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[1];
    let trigger_obj_id = 0x4240140;

    let trigger_obj = layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == trigger_obj_id)
        .and_then(|obj| obj.property_data.as_trigger_mut())
        .unwrap();
    trigger_obj.position = [94.571053, 301.616028, 0.344905].into();
    trigger_obj.scale = [6.052994, 24.659973, 7.878154].into();

    Ok(())
}

fn patch_ruined_courtyard_thermal_conduits_0_02(_ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[0];

    let thermal_conduit_actor_obj_id = 0xF01C7;
    let thermal_conduit_damageable_trigger_obj_id = 0xF01C8;

    let thermal_conduit_actor_obj = layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == thermal_conduit_actor_obj_id)
        .and_then(|obj| obj.property_data.as_actor_mut())
        .unwrap();
    thermal_conduit_actor_obj.active = 1;

    let thermal_conduit_damageable_trigger_obj = layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == thermal_conduit_damageable_trigger_obj_id)
        .and_then(|obj| obj.property_data.as_damageable_trigger_mut())
        .unwrap();
    thermal_conduit_damageable_trigger_obj.active = 1;

    Ok(())
}

fn patch_geothermal_core_destructible_rock_pal(_ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[0];

    let platform_obj_id = 0x1403AE;
    let scan_target_platform_obj_id = 0x1403B4;

    let platform_obj = layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == platform_obj_id)
        .and_then(|obj| obj.property_data.as_platform_mut())
        .unwrap();
    platform_obj.active = 0;

    let scan_target_platform_obj = layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == scan_target_platform_obj_id)
        .and_then(|obj| obj.property_data.as_point_of_interest_mut())
        .unwrap();
    scan_target_platform_obj.active = 0;

    Ok(())
}

fn patch_ore_processing_destructible_rock_pal(_ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[0];

    let platform_obj_id = 0x60372;
    let scan_target_platform_obj_id = 0x60378;

    let platform_obj = layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == platform_obj_id)
        .and_then(|obj| obj.property_data.as_platform_mut())
        .unwrap();
    platform_obj.active = 0;

    let scan_target_platform_obj = layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == scan_target_platform_obj_id)
        .and_then(|obj| obj.property_data.as_point_of_interest_mut())
        .unwrap();
    scan_target_platform_obj.active = 0;

    Ok(())
}

fn patch_main_quarry_door_lock_pal(_ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[7];

    let locked_door_actor_obj_id = 0x1c0205db;

    let locked_door_actor_obj = layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == locked_door_actor_obj_id)
        .and_then(|obj| obj.property_data.as_actor_mut())
        .unwrap();
    locked_door_actor_obj.active = 0;

    Ok(())
}

fn patch_mines_security_station_soft_lock<'r>(_ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[0];

    // Disable the the trigger when all the pirates are killed
    let obj = layer.objects.as_mut_vec().iter_mut()
        .find(|obj| obj.instance_id == 460074)
        .unwrap();
    obj.connections.as_mut_vec().push(structs::Connection {
            state: structs::ConnectionState::MAX_REACHED,
            message: structs::ConnectionMsg::DEACTIVATE,
            target_object_id: 67568447,
        });
    // TODO: Trigger a MemoryRelay too

    // TODO: Instead of the above, when you pass through a trigger near the "other" door, disable
    // the all of triggers related to the cutscenes in the room.
    Ok(())
}

fn patch_gravity_chamber_stalactite_grapple_point<'r>(_ps: &mut PatcherState, area: &mut mlvl_wrapper::MlvlArea)
    -> Result<(), String>
{
    let scly = area.mrea().scly_section_mut();
    let layer = &mut scly.layers.as_mut_vec()[0];

    // Remove the object that turns off the stalactites layer
    layer.objects.as_mut_vec().retain(|obj| obj.instance_id != 3473722);

    Ok(())
}

fn patch_main_strg(res: &mut structs::Resource, msg: &str) -> Result<(), String>
{
    let strings = res.kind.as_strg_mut().unwrap()
        .string_tables
        .as_mut_vec()
        .iter_mut()
        .find(|table| table.lang == b"ENGL".into())
        .unwrap()
        .strings
        .as_mut_vec();

    let s = strings.iter_mut()
        .find(|s| *s == "Metroid Fusion Connection Bonuses\u{0}")
        .unwrap();
    *s = "Extras\u{0}".to_string().into();

    strings.push(format!("{}\0", msg).into());
    Ok(())
}

fn patch_main_menu(res: &mut structs::Resource) -> Result<(), String>
{
    let frme = res.kind.as_frme_mut().unwrap();

    frme.widgets.as_mut_vec().push(structs::FrmeWidget {
        name: b"textpane_identifier\0".as_cstr(),
        parent: b"kGSYS_HeadWidgetID\0".as_cstr(),
        use_anim_controller: 0,
        default_visible: 1,
        default_active: 1,
        cull_faces: 0,
        color: [1.0, 1.0, 1.0, 1.0].into(),
        model_draw_flags: 2,
        kind: structs::FrmeWidgetKind::TextPane(
            structs::TextPaneWidget {
                x_dim: 10.455326,
                z_dim: 1.813613,
                scale_center: [
                    -5.227663,
                    0.0,
                    -0.51,
                ].into(),
                font: 3265024497,
                word_wrap: 0,
                horizontal: 1,
                justification: 0,
                vertical_justification: 0,
                fill_color: [1.0, 1.0, 1.0, 1.0].into(),
                outline_color: [0.0, 0.0, 0.0, 1.0].into(),
                block_extent: [213.0, 38.0].into(),
                jpn_font: None,
                jpn_point_scale: None,
            },
        ),
        worker_id: None,
        origin: [9.25, 1.500001, 0.0].into(),
        basis: [
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0,
        ].into(),
        rotation_center: [0.0, 0.0, 0.0].into(),
        unknown0: 0,
        unknown1: 0,
    });

    let mut shadow_widget = frme.widgets.as_mut_vec().last().unwrap().clone();
    shadow_widget.name = b"textpane_identifierb\0".as_cstr();
    let tp = match &mut shadow_widget.kind {
        structs::FrmeWidgetKind::TextPane(tp) => tp,
        _ => unreachable!(),
    };
    tp.fill_color = [0.0, 0.0, 0.0, 0.4].into();
    tp.outline_color = [0.0, 0.0, 0.0, 0.2].into();
    shadow_widget.origin[0] -= -0.235091;
    shadow_widget.origin[1] -= -0.104353;
    shadow_widget.origin[2] -= 0.176318;

    frme.widgets.as_mut_vec().push(shadow_widget);

    Ok(())
}


fn patch_credits(res: &mut structs::Resource, pickup_layout: &[PickupType])
    -> Result<(), String>
{
    use std::fmt::Write;
    const PICKUPS_TO_PRINT: &[PickupType] = &[
        PickupType::ScanVisor,
        PickupType::ThermalVisor,
        PickupType::XRayVisor,
        PickupType::VariaSuit,
        PickupType::GravitySuit,
        PickupType::PhazonSuit,
        PickupType::MorphBall,
        PickupType::BoostBall,
        PickupType::SpiderBall,
        PickupType::MorphBallBomb,
        PickupType::PowerBomb,
        PickupType::ChargeBeam,
        PickupType::SpaceJumpBoots,
        PickupType::GrappleBeam,
        PickupType::SuperMissile,
        PickupType::Wavebuster,
        PickupType::IceSpreader,
        PickupType::Flamethrower,
        PickupType::WaveBeam,
        PickupType::IceBeam,
        PickupType::PlasmaBeam
    ];

    let mut output = concat!(
        "\n\n\n\n\n\n\n",
        "&push;&font=C29C51F1;&main-color=#89D6FF;",
        "Major Item Locations",
        "&pop;",
    ).to_owned();
    for pickup_type in PICKUPS_TO_PRINT {
        let room_idx = if let Some(i) = pickup_layout.iter().position(|i| i == pickup_type) {
            i
        } else {
            continue
        };
        let room_name = pickup_meta::PICKUP_LOCATIONS.iter()
            .flat_map(|pak_locs| pak_locs.1.iter())
            .flat_map(|loc| iter::repeat(loc.name).take(loc.pickup_locations.len()))
            .nth(room_idx)
            .unwrap();
        let pickup_name = pickup_type.name();
        write!(output, "\n\n{}: {}", pickup_name, room_name).unwrap();
    }
    output += "\n\n\n\n\0";
    res.kind.as_strg_mut().unwrap().string_tables
        .as_mut_vec()
        .iter_mut()
        .find(|table| table.lang == b"ENGL".into())
        .unwrap()
        .strings
        .as_mut_vec()
        .push(output.into());
    Ok(())
}


fn patch_starting_pickups(
    area: &mut mlvl_wrapper::MlvlArea,
    mut starting_items: u64,
    debug_print: bool,
) -> Result<(), String>
{

    let scly = area.mrea().scly_section_mut();
    let mut first = debug_print;
    macro_rules! print_maybe {
        ($first:ident, $($tts:tt)*) => {
            if $first {
                println!($($tts)*);
            }

        };
    }
    for layer in scly.layers.iter_mut() {
        for obj in layer.objects.iter_mut() {
            let spawn_point = if let Some(spawn_point) = obj.property_data.as_spawn_point_mut() {
                spawn_point
            } else {
                continue;
            };

            let mut fetch_bits = move |bits: u8| {
                let ret = starting_items & ((1 << bits) - 1);
                starting_items >>= bits;
                ret as u32
            };

            print_maybe!(first, "Starting pickups set:");

            spawn_point.scan_visor = fetch_bits(1);
            print_maybe!(first, "    scan_visor: {}", spawn_point.scan_visor);

            spawn_point.missiles = fetch_bits(8);
            print_maybe!(first, "    missiles: {}", spawn_point.missiles);

            spawn_point.energy_tanks = fetch_bits(4);
            print_maybe!(first, "    energy_tanks: {}", spawn_point.energy_tanks);

            spawn_point.power_bombs = fetch_bits(4);
            print_maybe!(first, "    power_bombs: {}", spawn_point.power_bombs);

            spawn_point.wave = fetch_bits(1);
            print_maybe!(first, "    wave: {}", spawn_point.wave);

            spawn_point.ice = fetch_bits(1);
            print_maybe!(first, "    ice: {}", spawn_point.ice);

            spawn_point.plasma = fetch_bits(1);
            print_maybe!(first, "    plasma: {}", spawn_point.plasma);

            spawn_point.charge = fetch_bits(1);
            print_maybe!(first, "    charge: {}", spawn_point.charge);

            spawn_point.morph_ball = fetch_bits(1);
            print_maybe!(first, "    morph_ball: {}", spawn_point.morph_ball);

            spawn_point.bombs = fetch_bits(1);
            print_maybe!(first, "    bombs: {}", spawn_point.bombs);

            spawn_point.spider_ball = fetch_bits(1);
            print_maybe!(first, "    spider_ball: {}", spawn_point.spider_ball);

            spawn_point.boost_ball = fetch_bits(1);
            print_maybe!(first, "    boost_ball: {}", spawn_point.boost_ball);

            spawn_point.varia_suit = fetch_bits(1);
            print_maybe!(first, "    varia_suit: {}", spawn_point.varia_suit);

            spawn_point.gravity_suit = fetch_bits(1);
            print_maybe!(first, "    gravity_suit: {}", spawn_point.gravity_suit);

            spawn_point.phazon_suit = fetch_bits(1);
            print_maybe!(first, "    phazon_suit: {}", spawn_point.phazon_suit);

            spawn_point.thermal_visor = fetch_bits(1);
            print_maybe!(first, "    thermal_visor: {}", spawn_point.thermal_visor);

            spawn_point.xray= fetch_bits(1);
            print_maybe!(first, "    xray: {}", spawn_point.xray);

            spawn_point.space_jump = fetch_bits(1);
            print_maybe!(first, "    space_jump: {}", spawn_point.space_jump);

            spawn_point.grapple = fetch_bits(1);
            print_maybe!(first, "    grapple: {}", spawn_point.grapple);

            spawn_point.super_missile = fetch_bits(1);
            print_maybe!(first, "    super_missile: {}", spawn_point.super_missile);

            spawn_point.wavebuster = fetch_bits(1);
            print_maybe!(first, "    wavebuster: {}", spawn_point.wavebuster);

            spawn_point.ice_spreader = fetch_bits(1);
            print_maybe!(first, "    ice_spreader: {}", spawn_point.ice_spreader);

            spawn_point.flamethrower = fetch_bits(1);
            print_maybe!(first, "    flamethrower: {}", spawn_point.flamethrower);

            first = false;
        }
    }
    Ok(())
}

include!("../compile_to_ppc/patches_config.rs");
fn create_rel_config_file(
    spawn_room: SpawnRoom,
    quickplay: bool,
) -> Vec<u8>
{
    let config = RelConfig {
        quickplay_mlvl: if quickplay { spawn_room.mlvl } else { 0xFFFFFFFF },
        quickplay_mrea: if quickplay { spawn_room.mrea } else { 0xFFFFFFFF },
    };
    let mut buf = vec![0; mem::size_of::<RelConfig>()];
    ssmarshal::serialize(&mut buf, &config).unwrap();
    buf
}

fn patch_dol<'r>(
    file: &mut structs::FstEntryFile,
    spawn_room: SpawnRoom,
    version: Version,
    patch_heat_damage: bool,
    patch_suit_damage: bool,
) -> Result<(), String>
{
    macro_rules! symbol_addr {
        ($sym:tt, $version:expr) => {
            {
                let s = mp1_symbol!($sym);
                match &$version {
                    Version::Ntsc0_00 => s.addr_0_00,
                    Version::Ntsc0_01 => s.addr_0_01,
                    Version::Ntsc0_02 => s.addr_0_02,
                    Version::Pal      => s.addr_pal,
                }.unwrap_or_else(|| panic!("Symbol {} unknown for version {}", $sym, $version))
            }
        }
    }

    let reader = match *file {
        structs::FstEntryFile::Unknown(ref reader) => reader.clone(),
        _ => panic!(),
    };

    let mut dol_patcher = DolPatcher::new(reader);
    if version == Version::Pal {
        dol_patcher
            .patch(symbol_addr!("aMetroidprime", version), b"randomprime\0"[..].into())?;
    } else {
        dol_patcher
            .patch(symbol_addr!("aMetroidprimeA", version), b"randomprime A\0"[..].into())?
            .patch(symbol_addr!("aMetroidprimeB", version), b"randomprime B\0"[..].into())?;
    }

    let cinematic_skip_patch = ppcasm!(symbol_addr!("ShouldSkipCinematic__22CScriptSpecialFunctionFR13CStateManager", version), {
            li      r3, 0x1;
            blr;
    });
    dol_patcher.ppcasm_patch(&cinematic_skip_patch)?;

    // TODO: This offset needs to be adjusted for PAL, probably (or the patch temporarily disabled)
    let unlockables_default_ctor_patch = ppcasm!(symbol_addr!("__ct__14CSystemOptionsFv", version) + 0x194, {
            li      r6, 100;
            stw     r6, 0xcc(r3);
            lis     r6, 0xF7FF;
            stw     r6, 0xd0(r3);
    });
    dol_patcher.ppcasm_patch(&unlockables_default_ctor_patch)?;
    // TODO: This offset needs to be adjusted for PAL, probably (or the patch temporarily disabled)
    let unlockables_read_ctor_patch = ppcasm!(symbol_addr!("__ct__14CSystemOptionsFRC12CInputStream", version) + 0x308, {
            li      r6, 100;
            stw     r6, 0xcc(r28);
            lis     r6, 0xF7FF;
            stw     r6, 0xd0(r28);
            mr      r3, r29;
            li      r4, 2;
    });
    dol_patcher.ppcasm_patch(&unlockables_read_ctor_patch)?;


    if version != Version::Pal {
        let missile_hud_formating_patch = ppcasm!(symbol_addr!("SetNumMissiles__20CHudMissileInterfaceFiRC13CStateManager", version) + 0x14, {
                b          skip;
            fmt:
                .asciiz b"%03d/%03d";

            skip:
                stw        r30, 40(r1);// var_8(r1);
                mr         r30, r3;
                stw        r4, 8(r1);// var_28(r1)

                lwz        r6, 4(r30);

                mr         r5, r4;

                lis        r4, fmt@h;
                addi       r4, r4, fmt@l;

                addi       r3, r1, 12;// arg_C

                nop; // crclr      cr6;
                bl         { symbol_addr!("sprintf", version) };

                addi       r3, r1, 20;// arg_14;
                addi       r4, r1, 12;// arg_C
        });
        dol_patcher.ppcasm_patch(&missile_hud_formating_patch)?;
    }

    let powerbomb_hud_formating_patch = ppcasm!(symbol_addr!("SetBombParams__17CHudBallInterfaceFiiibbb", version) + 0x2c, {
            b skip;
        fmt:
            .asciiz b"%d/%d"; // %d";
            nop;
        skip:
            mr         r6, r27;
            mr         r5, r28;
            lis        r4, fmt@h;
            addi       r4, r4, fmt@l;
            addi       r3, r1, 12;// arg_C;
            nop; // crclr      cr6;
            bl         { symbol_addr!("sprintf", version) };

    });
    dol_patcher.ppcasm_patch(&powerbomb_hud_formating_patch)?;

    // TODO: The offset here needs to be higher for PAL. +16 and +28
    let level_select_mlvl_upper_patch = ppcasm!(symbol_addr!("__sinit_CFrontEndUI_cpp", version) + 4, {
            lis         r4, {spawn_room.mlvl}@h;
    });
    dol_patcher.ppcasm_patch(&level_select_mlvl_upper_patch)?;

    let level_select_mlvl_lower_patch = ppcasm!(symbol_addr!("__sinit_CFrontEndUI_cpp", version) + 0x10, {
            addi        r0, r4, {spawn_room.mlvl}@l;
    });
    dol_patcher.ppcasm_patch(&level_select_mlvl_lower_patch)?;

    let level_select_mrea_idx_patch = ppcasm!(symbol_addr!("__ct__11CWorldStateFUi", version) + 0x10, {
            li          r0, { spawn_room.mrea_idx };
    });
    dol_patcher.ppcasm_patch(&level_select_mrea_idx_patch)?;

    let disable_hints_setting_patch = ppcasm!(symbol_addr!("ResetToDefaults__12CGameOptionsFv", version) + 0x80, {
            rlwimi      r0, r6, 3, 28, 28;
    });
    dol_patcher.ppcasm_patch(&disable_hints_setting_patch)?;

    if patch_heat_damage {
        let heat_damage_patch = ppcasm!(symbol_addr!("ThinkAreaDamage__22CScriptSpecialFunctionFfR13CStateManager", version) + 0x4c, {
                lwz     r4, 0xdc(r4);
                nop;
                subf    r0, r6, r5;
                cntlzw  r0, r0;
                nop;
        });
        dol_patcher.ppcasm_patch(&heat_damage_patch)?;
    }

    if patch_suit_damage {
        // TODO: The jump offset is almost certainly wrong, so double check that
        let staggered_suit_damage_patch = ppcasm!(symbol_addr!("ApplyLocalDamage__13CStateManagerFRC9CVector3fRC9CVector3fR6CActorfRC11CWeaponMode", version) + 0x128, {
                lwz     r3, 0x8b8(r25);
                lwz     r3, 0(r3);
                lwz     r4, 220(r3);
                lwz     r5, 212(r3);
                addc    r4, r4, r5;
                lwz     r5, 228(r3);
                addc    r4, r4, r5;
                rlwinm  r4, r4, 2, 0, 29;
                lis     r6, data@h;
                addi    r6, r6, data@l;
                lfsx     f0, r4, r6;
                b       { symbol_addr!("ApplyLocalDamage__13CStateManagerFRC9CVector3fRC9CVector3fR6CActorfRC11CWeaponMode", version) + 0x1c4 };
            data:
                .float 0.0;
                .float 0.1;
                .float 0.2;
                .float 0.5;
        });
        dol_patcher.ppcasm_patch(&staggered_suit_damage_patch)?;
    }

    if version == Version::Ntsc0_02 || version == Version::Pal {
        let players_choice_scan_dash_patch = ppcasm!(symbol_addr!("SidewaysDashAllowed__7CPlayerCFffRC11CFinalInputR13CStateManager", version) + 0x3c, {
                b       { symbol_addr!("SidewaysDashAllowed__7CPlayerCFffRC11CFinalInputR13CStateManager", version) + 0x54 };
        });
        dol_patcher.ppcasm_patch(&players_choice_scan_dash_patch)?;
    }
    let (rel_loader_bytes, rel_loader_map_str) = match version {
        Version::Ntsc0_00 => {
            let loader_bytes = generated::REL_LOADER_100;
            let map_str = generated::REL_LOADER_100_MAP;
            (loader_bytes, map_str)
        },
        Version::Ntsc0_01 => unreachable!(),
        Version::Ntsc0_02 => {
            let loader_bytes = generated::REL_LOADER_102;
            let map_str = generated::REL_LOADER_102_MAP;
            (loader_bytes, map_str)
        },
        Version::Pal => {
            let loader_bytes = generated::REL_LOADER_PAL;
            let map_str = generated::REL_LOADER_PAL_MAP;
            (loader_bytes, map_str)
        },
    };

    let mut rel_loader = rel_loader_bytes.to_vec();

    let rel_loader_map = dol_linker::parse_symbol_table(
        "extra_assets/rel_loader_1.0?.bin.map".as_ref(),
        rel_loader_map_str.lines().map(|l| Ok(l.to_owned())),
    ).map_err(|e| e.to_string())?;


    let bytes_needed = ((rel_loader.len() + 31) & !31) - rel_loader.len();
    rel_loader.extend([0; 32][..bytes_needed].iter().copied());

    dol_patcher.add_text_segment(0x80002000, Cow::Owned(rel_loader))?;

    dol_patcher.ppcasm_patch(&ppcasm!(symbol_addr!("PPCSetFpIEEEMode", version) + 4, {
        b      { rel_loader_map["rel_loader_hook"] };
    }))?;


    *file = structs::FstEntryFile::ExternalFile(Box::new(dol_patcher));
    Ok(())
}

fn empty_frigate_pak<'r>(file: &mut structs::FstEntryFile)
    -> Result<(), String>
{
    // To reduce the amount of data that needs to be copied, empty the contents of the pak
    let pak = match file {
        structs::FstEntryFile::Pak(pak) => pak,
        _ => unreachable!(),
    };

    // XXX This is a workaround for a bug in some versions of Nintendont.
    //     The details can be found in a comment on issue #5.
    let res = pickup_meta::build_resource(
        0,
        structs::ResourceKind::External(vec![0; 64], b"XXXX".into())
    );
    pak.resources = iter::once(res).collect();
    Ok(())
}

fn patch_bnr(file: &mut structs::FstEntryFile, config: &ParsedConfig) -> Result<(), String>
{
    let bnr = match file {
        structs::FstEntryFile::Bnr(bnr) => bnr,
        _ => panic!(),
    };

    bnr.pixels.clone_from_slice(include_bytes!("../extra_assets/banner_image.bin"));

    fn write_encoded_str(field: &str, s: &Option<String>, slice: &mut [u8]) -> Result<(), String>
    {
        if let Some(s) = s {
            let mut bytes = WINDOWS_1252.encode(&s, EncoderTrap::Strict)
                .map_err(|e| format!("Failed to encode banner field {}: {}", field, e))?;
            if bytes.len() >= (slice.len() - 1) {
                Err(format!("Invalid encoded length for banner field {}: expect {}, got {}",
                            field, slice.len() - 1, bytes.len()))?
            }
            bytes.resize(slice.len(), 0u8);
            slice.clone_from_slice(&bytes);
        }
        Ok(())
    }

    write_encoded_str("game_name", &config.bnr_game_name, &mut bnr.english_fields.game_name)?;
    write_encoded_str("developer", &config.bnr_developer, &mut bnr.english_fields.developer)?;
    write_encoded_str(
        "game_name_full",
        &config.bnr_game_name_full,
        &mut bnr.english_fields.game_name_full
    )?;
    write_encoded_str(
        "developer_full",
        &config.bnr_developer_full,
        &mut bnr.english_fields.developer_full)
    ?;
    write_encoded_str("description", &config.bnr_description, &mut bnr.english_fields.description)?;

    Ok(())
}

// XXX Deserialize is implemented here for c_interface. Ideally this could be done in
//     c_interface.rs itself...
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IsoFormat
{
    Iso,
    Gcz,
    Ciso,
}

impl Default for IsoFormat
{
    fn default() -> IsoFormat
    {
        IsoFormat::Iso
    }
}

#[derive(Deserialize, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ArtifactHintBehavior
{
    Default,
    None,
    All,
}

impl Default for ArtifactHintBehavior
{
    fn default() -> Self
    {
        ArtifactHintBehavior::Default
    }
}

pub struct ParsedConfig
{
    pub input_iso: memmap::Mmap,
    pub output_iso: File,
    pub layout_string: String,
    pub is_item_randomized: Option<bool>,

    pub pickup_layout: Vec<u8>,
    pub elevator_layout: Vec<u8>,
    pub elevator_layout_override: Vec<String>,
    pub starting_room: String,
    pub item_seed: u64,
    pub seed: u64,
    pub door_weights: Weights,
    pub excluded_doors: [HashMap<String,Vec<String>>;7],
    pub patch_map: bool,

    pub iso_format: IsoFormat,
    pub skip_frigate: bool,
    pub skip_hudmenus: bool,
    pub keep_fmvs: bool,
    pub obfuscate_items: bool,
    pub nonvaria_heat_damage: bool,
    pub staggered_suit_damage: bool,
    pub auto_enabled_elevators: bool,
    pub powerbomb_lockpick: bool,
    pub quiet: bool,
    pub tiny_elvetator_samus: bool,

    pub skip_impact_crater: bool,
    pub enable_vault_ledge_door: bool,
    pub artifact_hint_behavior: ArtifactHintBehavior,
    pub patch_vertical_to_blue: bool,

    pub flaahgra_music_files: Option<[nod_wrapper::FileWrapper; 2]>,

    pub starting_items: Option<u64>,
    pub starting_items_frigate: Option<u64>,
    pub comment: String,
    pub main_menu_message: String,

    pub quickplay: bool,

    pub bnr_game_name: Option<String>,
    pub bnr_developer: Option<String>,

    pub bnr_game_name_full: Option<String>,
    pub bnr_developer_full: Option<String>,
    pub bnr_description: Option<String>,

    pub pal_override: bool,
}


#[derive(PartialEq, Copy, Clone)]
enum Version
{
    Ntsc0_00,
    Ntsc0_01,
    Ntsc0_02,
    Pal,
}

impl fmt::Display for Version
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error>
    {
        match self {
            Version::Ntsc0_00 => write!(f, "1.00"),
            Version::Ntsc0_01 => write!(f, "1.01"),
            Version::Ntsc0_02 => write!(f, "1.02"),
            Version::Pal      => write!(f, "pal"),
        }
    }
}

pub fn patch_iso<T>(mut config: ParsedConfig, mut pn: T) -> Result<(), String>
    where T: structs::ProgressNotifier
{
    let mut ct = Vec::new();
    writeln!(ct, "Created by randomprime version {}", env!("CARGO_PKG_VERSION")).unwrap();
    writeln!(ct).unwrap();
    writeln!(ct, "Options used:").unwrap();
    writeln!(ct, "configuration string: {}", config.layout_string).unwrap();
    writeln!(ct, "skip frigate: {}", config.skip_frigate).unwrap();
    writeln!(ct, "keep fmvs: {}", config.keep_fmvs).unwrap();
    writeln!(ct, "nonmodal hudmemos: {}", config.skip_hudmenus).unwrap();
    writeln!(ct, "obfuscated items: {}", config.obfuscate_items).unwrap();

    let mut dt = Vec::new();
    writeln!(dt, "{}",config.comment).unwrap();
    writeln!(dt).unwrap();
    writeln!(dt, "Configuration:").unwrap();
    writeln!(dt, "seed: {}",config.seed).unwrap();
    writeln!(dt, "door weights: {:?}",config.door_weights).unwrap();
    writeln!(dt, "excluded_doors: {:?}",config.excluded_doors).unwrap();

    let mut reader = Reader::new(&config.input_iso[..]);

    let mut gc_disc: structs::GcDisc = reader.read(());

    let version = match (&gc_disc.header.game_identifier(), gc_disc.header.disc_id, gc_disc.header.version) {
        (b"GM8E01", 0, 0) => Version::Ntsc0_00,
        (b"GM8E01", 0, 1) => Version::Ntsc0_01,
        (b"GM8E01", 0, 2) => Version::Ntsc0_02,
        (b"GM8P01", 0, 0) => Version::Pal,
        _ => Err("The input ISO doesn't appear to be NTSC-US or PAL Metroid Prime.".to_string())?
    };
    config.is_item_randomized = Some(gc_disc.find_file("randomprime.txt").is_some());
    if config.is_item_randomized.unwrap_or(false) {
        pn.notify_stacking_warning();
    }
    if gc_disc.find_file("mpdr.txt").is_some() {
        Err(concat!("The input ISO has already been randomized using MPDR. ",
                    "You must start from an unmodified ISO or an item randomized one every time."
        ))?
    }
    if version == Version::Ntsc0_01 || (version == Version::Pal && !config.pal_override) {
        Err("The NTSC 0-01 and PAL versions of Metroid Prime are not current supported.")?;
    }

    build_and_run_patches(&mut gc_disc, &config, version)?;

    gc_disc.add_file("randomprime.txt", structs::FstEntryFile::Unknown(Reader::new(&ct)))?;
    gc_disc.add_file("mpdr.txt",structs::FstEntryFile::Unknown(Reader::new(&dt)))?;


    if !config.is_item_randomized.unwrap_or(false) && version != Version::Ntsc0_01 && version != Version::Pal {
        let patches_rel_bytes = match version {
            Version::Ntsc0_00 => generated::PATCHES_100_REL,
            Version::Ntsc0_01 => unreachable!(),
            Version::Ntsc0_02 => generated::PATCHES_102_REL,
            Version::Pal      => generated::PATCHES_PAL_REL,
        };
        gc_disc.add_file(
            "patches.rel",
            structs::FstEntryFile::Unknown(Reader::new(patches_rel_bytes))
        )?;
    }

    match config.iso_format {
        IsoFormat::Iso => {
            let mut file = config.output_iso;
            file.set_len(structs::GC_DISC_LENGTH as u64)
                .map_err(|e| format!("Failed to resize output file: {}", e))?;
            gc_disc.write(&mut file, &mut pn)
                .map_err(|e| format!("Error writing output file: {}", e))?;
            pn.notify_flushing_to_disk();
        },
        IsoFormat::Gcz => {
            let mut gcz_writer = GczWriter::new(config.output_iso, structs::GC_DISC_LENGTH as u64)
                .map_err(|e| format!("Failed to prepare output file for writing: {}", e))?;
            gc_disc.write(&mut *gcz_writer, &mut pn)
                .map_err(|e| format!("Error writing output file: {}", e))?;
            pn.notify_flushing_to_disk();
        },
        IsoFormat::Ciso => {
            let mut ciso_writer = CisoWriter::new(config.output_iso)
                .map_err(|e| format!("Failed to prepare output file for writing: {}", e))?;
            gc_disc.write(&mut ciso_writer, &mut pn)
                .map_err(|e| format!("Error writing output file: {}", e))?;
            pn.notify_flushing_to_disk();
        }
    };
    Ok(())
}

fn spawn_room_from_string(room_string: String) -> SpawnRoom {
    let vec: Vec<&str> = room_string.split(":").collect();
    assert!(vec.len() == 2);
    let world_name = vec[0];
    let room_name = vec[1];

    for (pak_name, rooms) in pickup_meta::PICKUP_LOCATIONS.iter() { // for each pak
        let world = World::from_pak(pak_name).unwrap();

        if !world.as_string().to_lowercase().starts_with(&world_name.to_lowercase()) {
            continue;
        }

        let mut idx: u32 = 0;
        for room_info in rooms.iter() { // for each room in the pak
            if room_info.name.to_lowercase() == room_name.to_lowercase() {
                return SpawnRoom {
                    pak_name,
                    mlvl: world.mlvl(),
                    mrea: room_info.room_id,
                    mrea_idx: idx,
                };
            }
            idx = idx + 1;
        }
    }

    assert!(false);
    return SpawnRoom::landing_site_spawn_room();
}

fn room_strg_id_from_mrea_id(mrea_id: u32) -> (u32, u32)
{
    for _ in pickup_meta::PICKUP_LOCATIONS.iter().map(|(name, _)| name) {
        let mut idx = 0;
        for (_, rooms) in pickup_meta::PICKUP_LOCATIONS.iter() {
            for room_info in rooms.iter() {
                if room_info.room_id == mrea_id {
                    return (idx ,room_info.name_id);
                }
            }
            idx = idx + 1;
        }
    }

    assert!(false);
    (0, 0)
}

fn build_and_run_patches(gc_disc: &mut structs::GcDisc, config: &ParsedConfig, version: Version)
    -> Result<(), String>
{
    let pickup_layout: Vec<_> = config.pickup_layout.iter()
        .map(|i| PickupType::from_idx(*i as usize).unwrap())
        .collect();
    let pickup_layout = &pickup_layout[..];

    let mut elevator_layout: Vec<_> = config.elevator_layout[..ELEVATORS.len()].iter()
        .map(|i| ELEVATORS[*i as usize])
        .map(|elv| if config.skip_impact_crater && elv.name == "Crater Entry Point" {
                Elevator::end_game_elevator()
            } else {
                elv
            })
        .collect();
    
    let mut idx = 0;
    for elv in &config.elevator_layout_override {
        let spawn_room = spawn_room_from_string(elv.to_string());
        
        assert!(!(spawn_room.mlvl == World::FrigateOrpheon.mlvl() && config.skip_frigate)); // panic if a elevator destination takes you to the removed frigate level
        elevator_layout[idx].mlvl = spawn_room.mlvl;
        elevator_layout[idx].mrea = spawn_room.mrea; 

        let (mrea_idx, _) = room_strg_id_from_mrea_id(spawn_room.mrea);
        elevator_layout[idx].mrea_idx = mrea_idx;
        idx = idx + 1;
    }
    
    let spawn_room = spawn_room_from_string(config.starting_room.to_string());
    assert!(spawn_room.mlvl != World::FrigateOrpheon.mlvl()); // The game will freeze if frigate is skipped, you can never get to the planet if it isn't
    assert!(spawn_room.mlvl != World::ImpactCrater.mlvl() || !config.skip_impact_crater);
    
    let mut rng = StdRng::seed_from_u64(config.seed);
    let artifact_totem_strings = build_artifact_temple_totem_scan_strings(pickup_layout, &mut rng);
    let mut pickup_resources = collect_pickup_resources(gc_disc);
    let door_resources = collect_door_resources(gc_disc);
    if config.skip_hudmenus {
        add_skip_hudmemos_strgs(&mut pickup_resources);
    }

    // XXX These values need to out live the patcher
    let select_game_fmv_suffix = ["A", "B", "C"].choose(&mut rng).unwrap();
    let n = format!("Video/02_start_fileselect_{}.thp", select_game_fmv_suffix);
    let start_file_select_fmv = gc_disc.find_file(&n).unwrap().file().unwrap().clone();
    let n = format!("Video/04_fileselect_playgame_{}.thp", select_game_fmv_suffix);
    let file_select_play_game_fmv = gc_disc.find_file(&n).unwrap().file().unwrap().clone();


    let pickup_resources = &pickup_resources;
    let door_resources = &door_resources;
    let mut patcher = PrimePatcher::new();
    if !config.is_item_randomized.unwrap_or(false) && !config.keep_fmvs {
        patcher.add_file_patch(b"opening.bnr", |file| patch_bnr(file, config));
        // Replace the attract mode FMVs with empty files to reduce the amount of data we need to
        // copy and to make compressed ISOs smaller.
        const FMV_NAMES: &[&[u8]] = &[
            b"Video/attract0.thp",
            b"Video/attract1.thp",
            b"Video/attract2.thp",
            b"Video/attract3.thp",
            b"Video/attract4.thp",
            b"Video/attract5.thp",
            b"Video/attract6.thp",
            b"Video/attract7.thp",
            b"Video/attract8.thp",
            b"Video/attract9.thp",

        ];
        const FMV: &[u8] = include_bytes!("../extra_assets/attract_mode.thp");
        for name in FMV_NAMES {
            patcher.add_file_patch(name, |file| {
                *file = structs::FstEntryFile::ExternalFile(Box::new(FMV));
                Ok(())
            });
        }
    }

    // patch videos
    if !config.is_item_randomized.unwrap_or(false) {
        if let Some(flaahgra_music_files) = &config.flaahgra_music_files {
            const MUSIC_FILE_NAME: &[&[u8]] = &[
                b"Audio/rui_flaaghraR.dsp",
                b"Audio/rui_flaaghraL.dsp",
            ];
            for (file_name, music_file) in MUSIC_FILE_NAME.iter().zip(flaahgra_music_files.iter()) {
                patcher.add_file_patch(file_name, move |file| {
                    *file = structs::FstEntryFile::ExternalFile(Box::new(music_file.clone()));
                    Ok(())
                });
            }
        }

        // Replace the FMVs that play when you select a file so each ISO always plays the only one.
        const SELECT_GAMES_FMVS: &[&[u8]] = &[
            b"Video/02_start_fileselect_A.thp",
            b"Video/02_start_fileselect_B.thp",
            b"Video/02_start_fileselect_C.thp",
            b"Video/04_fileselect_playgame_A.thp",
            b"Video/04_fileselect_playgame_B.thp",
            b"Video/04_fileselect_playgame_C.thp",
        ];
        for fmv_name in SELECT_GAMES_FMVS {
            let fmv_ref = if fmv_name[7] == b'2' {
                &start_file_select_fmv
            } else {
                &file_select_play_game_fmv
            };
            patcher.add_file_patch(fmv_name, move |file| {
                *file = fmv_ref.clone();
                Ok(())
            });
        }
    }
            
    // TODO: patch other interactive actors here (e.g. power conduits, iced over things, missile locks etc...)
    
    // Patch pickups and doors
    let mut layout_iterator = pickup_layout.iter();
    let mut door_rng = StdRng::seed_from_u64(config.seed);
    for (name, rooms) in pickup_meta::PICKUP_LOCATIONS.iter() { // for each .pak
        let world = World::from_pak(name).unwrap();
        let level = world as usize;

        if level == 0 && config.skip_frigate {continue;} // If we're skipping the frigate, there's nothing to patch
        if level == 6 && config.skip_impact_crater {continue;} // If we're skipping the frigate, there's nothing to patch

        for room_info in rooms.iter() { // for each room in the pak
            // patch the item locations
            if !config.is_item_randomized.unwrap_or(false) {
                 patcher.add_scly_patch((name.as_bytes(), room_info.room_id), move |_, area| {
                    // Remove objects
                    let layers = area.mrea().scly_section_mut().layers.as_mut_vec();
                    for otr in room_info.objects_to_remove {
                        layers[otr.layer as usize].objects.as_mut_vec()
                            .retain(|i| !otr.instance_ids.contains(&i.instance_id));
                    }
                    Ok(())
                });
                let iter = room_info.pickup_locations.iter().zip(&mut layout_iterator);
                for (&pickup_location, &pickup_type) in iter {
                    // 1 in 1024 chance of a missile being shiny means a player is likely to see a
                    // shiny missile every 40ish games (assuming most players collect about half of the
                    // missiles)
                    let pickup_type = if pickup_type == PickupType::Missile && rng.gen_ratio(1, 1024) {
                        PickupType::ShinyMissile
                    } else {
                        pickup_type
                    };
                    patcher.add_scly_patch(
                        (name.as_bytes(), room_info.room_id),
                        move |ps, area| modify_pickups_in_mrea(
                                ps,
                                area,
                                pickup_type,
                                pickup_location,
                                pickup_resources,
                                config
                            )
                    );
                }
            }

            // patch the door locations
            let iter = room_info.door_locations.iter();
            for &door_location in iter // for each door location in the room
            {
                if door_location.dock_number.is_none() { continue; }
                let door_index = door_location.dock_number.unwrap() as usize;
                
                // println!("excluded_doors[{}][{}][{}]", level, room_info.name.to_string(), door_index);
                let door_specification = &config.excluded_doors[level][room_info.name][door_index];

                let is_vertical_door =  (room_info.room_id == 0x11BD63B7 && door_index == 0) || // Tower Chamber
                                        (room_info.room_id == 0x0D72F1F7 && door_index == 1) || // Tower of Light
                                        (room_info.room_id == 0xFB54A0CB && door_index == 4) || // Hall of the Elders 
                                        (room_info.room_id == 0xE1981EFC && door_index == 0) || // Elder Chamber
                                        (room_info.room_id == 0x43E4CC25 && door_index == 1) || // Research Lab Hydra
                                        (room_info.room_id == 0x37BBB33C && door_index == 1) || // Observatory Access
                                        (room_info.room_id == 0xD8E905DD && door_index == 1) || // Research Core Access
                                        (room_info.room_id == 0x21B4BFF6 && door_index == 1) || // Research Lab Aether
                                        (room_info.room_id == 0x3F375ECC && door_index == 2) || // Omega Research
                                        (room_info.room_id == 0xF517A1EA && door_index == 1) || // Dynamo Access (Careful of Chozo room w/ same name)
                                        (room_info.room_id == 0x8A97BB54 && door_index == 1) || // Elite Research
                                        (room_info.room_id == 0xA20201D4                   ) || // Security Access B (both doors)
                                        (room_info.room_id == 0x956F1552 && door_index == 1) || // Mine Security Station
                                        (room_info.room_id == 0xC50AF17A && door_index == 2) || // Elite Control
                                        (room_info.room_id == 0x90709AAC && door_index == 1);   // Ventilation Shaft

                let mut door_type = calculate_door_type(name,&mut door_rng,&config.door_weights); // randomly pick a door color using weights
                
                if door_specification == "blue"
                {
                    if !is_vertical_door {
                        door_type = DoorType::Blue;
                    } else {
                        door_type = DoorType::VerticalBlue;
                    }
                }
                
                if door_specification == "purple"
                {
                    if !is_vertical_door {
                        door_type = DoorType::Purple;
                    } else {
                        door_type = DoorType::VerticalPurple;
                    }
                }
                
                if door_specification == "white"
                {
                    if !is_vertical_door {
                        door_type = DoorType::White;
                    } else {
                        door_type = DoorType::VerticalWhite;
                    }
                }
                
                if door_specification == "red"
                {
                    if !is_vertical_door {
                        door_type = DoorType::Red;
                    } else {
                        door_type = DoorType::VerticalRed;
                    }
                }

                if door_specification == "power_bomb"
                {
                    if !is_vertical_door {
                        door_type = DoorType::PowerBomb;
                    } else {
                        door_type = DoorType::VerticalPowerBomb;
                    }
                }
                
                if door_specification == "bomb"
                {
                    if !is_vertical_door {
                        door_type = DoorType::Bomb;
                    } else {
                        door_type = DoorType::VerticalBomb;
                    }
                }
                
                if door_specification == "missile"
                {
                    if !is_vertical_door {
                        door_type = DoorType::Missile;
                    } else {
                        door_type = DoorType::VerticalMissile;
                    }
                }

                if door_specification == "charge"
                {
                    if !is_vertical_door {
                        door_type = DoorType::Charge;
                    } else {
                        door_type = DoorType::VerticalCharge;
                    }
                }
                
                if door_specification == "super"
                {
                    if !is_vertical_door {
                        door_type = DoorType::Super;
                    } else {
                        door_type = DoorType::VerticalSuper;
                    }
                }

                if door_specification == "wavebuster"
                {
                    if !is_vertical_door {
                        door_type = DoorType::Wavebuster;
                    } else {
                        door_type = DoorType::VerticalWavebuster;
                    }
                }

                if door_specification == "icespreader"
                {
                    if !is_vertical_door {
                        door_type = DoorType::Icespreader;
                    } else {
                        door_type = DoorType::VerticalIcespreader;
                    }
                }

                if door_specification == "flamethrower"
                {
                    if !is_vertical_door {
                        door_type = DoorType::Flamethrower;
                    } else {
                        door_type = DoorType::VerticalFlamethrower;
                    }
                }
                
                if door_specification == "disabled"
                {
                    if !is_vertical_door {
                        door_type = DoorType::Disabled;
                    } else {
                        door_type = DoorType::VerticalDisabled;
                    }
                }

                if door_specification == "ai"
                {
                    if !is_vertical_door {
                        door_type = DoorType::Ai;
                    } else {
                        door_type = DoorType::VerticalAi;
                    }
                }
                
                if is_vertical_door && config.patch_vertical_to_blue {
                    door_type = DoorType::VerticalBlue;
                }

                if (door_specification != "default") || (is_vertical_door && config.patch_vertical_to_blue)
                {
                    patcher.add_scly_patch(
                        (name.as_bytes(), room_info.room_id),
                        move |_ps, area| patch_door(area,door_location,door_type,door_resources,config.powerbomb_lockpick)
                    );

                    if config.patch_map && room_info.mapa_id != 0 {
                        patcher.add_resource_patch(
                            (&[name.as_bytes()], room_info.mapa_id,b"MAPA".into()),
                            move |res| patch_map_door_icon(res,door_location,door_type)
                        );
                    }
                }
            }
        }
    }

    if !config.is_item_randomized.unwrap_or(false) {
        let rel_config;
        if config.skip_frigate {
            patcher.add_file_patch(
                b"default.dol",
                move |file| patch_dol(
                    file,
                    spawn_room,
                    version,
                    config.nonvaria_heat_damage,
                    config.staggered_suit_damage,
                )
            );
            patcher.add_file_patch(b"Metroid1.pak", empty_frigate_pak);
            rel_config = create_rel_config_file(spawn_room, config.quickplay);
        } else {
            patcher.add_file_patch(
                b"default.dol",
                |file| patch_dol(
                    file,
                    SpawnRoom::frigate_spawn_room(),
                    version,
                    config.nonvaria_heat_damage,
                    config.staggered_suit_damage,
                )
            );
            patcher.add_scly_patch(
                resource_info!("01_intro_hanger.MREA").into(),
                move |_ps, area| patch_frigate_teleporter(area, spawn_room)
            );
            rel_config = create_rel_config_file(SpawnRoom::frigate_spawn_room(), config.quickplay);
        }

        gc_disc.add_file(
            "rel_config.bin",
            structs::FstEntryFile::ExternalFile(Box::new(rel_config)),
        )?;

        // Patch Frigate Starting Items //
        {
            let (starting_items, print_sis) = if let Some(starting_items) = config.starting_items_frigate {
                (starting_items, true)
            } else {
                (1, false)
            };
            patcher.add_scly_patch(
                (SpawnRoom::frigate_spawn_room().pak_name.as_bytes(), SpawnRoom::frigate_spawn_room().mrea),
                move |_ps, area| patch_starting_pickups(area, starting_items, print_sis)
            );
        }

        // Patch TallonIV Starting Items //
        {
            let (starting_items, print_sis) = if let Some(starting_items) = config.starting_items {
                (starting_items, true)
            } else {
                (1, false)
            };
            patcher.add_scly_patch(
                (spawn_room.pak_name.as_bytes(), spawn_room.mrea),
                move |_ps, area| patch_starting_pickups(area, starting_items, print_sis)
            );
        }

        const ARTIFACT_TOTEM_SCAN_STRGS: &[ResourceInfo] = &[
            resource_info!("07_Over_Stonehenge Totem 5.STRG"), // Lifegiver
            resource_info!("07_Over_Stonehenge Totem 4.STRG"), // Wild
            resource_info!("07_Over_Stonehenge Totem 10.STRG"), // World
            resource_info!("07_Over_Stonehenge Totem 9.STRG"), // Sun
            resource_info!("07_Over_Stonehenge Totem 3.STRG"), // Elder
            resource_info!("07_Over_Stonehenge Totem 11.STRG"), // Spirit
            resource_info!("07_Over_Stonehenge Totem 1.STRG"), // Truth
            resource_info!("07_Over_Stonehenge Totem 7.STRG"), // Chozo
            resource_info!("07_Over_Stonehenge Totem 6.STRG"), // Warrior
            resource_info!("07_Over_Stonehenge Totem 12.STRG"), // Newborn
            resource_info!("07_Over_Stonehenge Totem 8.STRG"), // Nature
            resource_info!("07_Over_Stonehenge Totem 2.STRG"), // Strength
        ];
        for (res_info, strg_text) in ARTIFACT_TOTEM_SCAN_STRGS.iter().zip(artifact_totem_strings.iter()) {
            patcher.add_resource_patch(
                (*res_info).into(),
                move |res| patch_artifact_totem_scan_strg(res, &strg_text),
            );
        }

        patcher.add_resource_patch(
            resource_info!("STRG_Main.STRG").into(),// 0x0552a456
            |res| patch_main_strg(res, &config.main_menu_message)
        );
        patcher.add_resource_patch(
            resource_info!("FRME_NewFileSelect.FRME").into(),
            patch_main_menu
        );

        patcher.add_resource_patch(
            resource_info!("STRG_Credits.STRG").into(),
            |res| patch_credits(res, &pickup_layout)
        );

        patcher.add_resource_patch(
            resource_info!("!MinesWorld_Master.SAVW").into(),
            patch_mines_savw_for_phazon_suit_scan
        );
        patcher.add_scly_patch(
            resource_info!("07_stonehenge.MREA").into(),
            |ps, area| fix_artifact_of_truth_requirements(ps, area, &pickup_layout)
        );
        patcher.add_scly_patch(
            resource_info!("07_stonehenge.MREA").into(),
            |ps, area| patch_artifact_hint_availability(ps, area, config.artifact_hint_behavior)
        );

        patcher.add_resource_patch(
            resource_info!("TXTR_SaveBanner.TXTR").into(),
            patch_save_banner_txtr
        );

        patcher.add_resource_patch(resource_info!("FRME_BallHud.FRME").into(), patch_morphball_hud);


        make_elevators_patch(&mut patcher, &elevator_layout, &config.elevator_layout_override, config.auto_enabled_elevators, config.tiny_elvetator_samus);

        make_elite_research_fight_prereq_patches(&mut patcher);

        patcher.add_scly_patch(
            resource_info!("22_Flaahgra.MREA").into(),
            patch_sunchamber_prevent_wild_before_flaahgra
        );
        patcher.add_scly_patch(
            resource_info!("0v_connect_tunnel.MREA").into(),
            patch_sun_tower_prevent_wild_before_flaahgra
        );
        patcher.add_scly_patch(
            resource_info!("00j_over_hall.MREA").into(),
            patch_temple_security_station_cutscene_trigger
        );
        patcher.add_scly_patch(
            resource_info!("01_ice_plaza.MREA").into(),
            patch_ridley_phendrana_shorelines_cinematic
        );
        patcher.add_scly_patch(
            resource_info!("08b_under_intro_ventshaft.MREA").into(),
            patch_main_ventilation_shaft_section_b_door
        );
        patcher.add_scly_patch(
            resource_info!("10_ice_research_a.MREA").into(),
            patch_research_lab_hydra_barrier);
        patcher.add_scly_patch(
            resource_info!("13_ice_vault.MREA").into(),
            patch_research_lab_aether_exploding_wall
        );
        patcher.add_scly_patch(
            resource_info!("11_ice_observatory.MREA").into(),
            patch_observatory_2nd_pass_solvablility
        );
        patcher.add_scly_patch(
            resource_info!("02_mines_shotemup.MREA").into(),
            patch_mines_security_station_soft_lock
        );
        patcher.add_scly_patch(
            resource_info!("18_ice_gravity_chamber.MREA").into(),
            patch_gravity_chamber_stalactite_grapple_point
        );


        if version == Version::Ntsc0_02 {
            patcher.add_scly_patch(
                resource_info!("01_mines_mainplaza.MREA").into(),
                patch_main_quarry_door_lock_0_02
            );
            patcher.add_scly_patch(
                resource_info!("13_over_burningeffigy.MREA").into(),
                patch_geothermal_core_door_lock_0_02
            );
            patcher.add_scly_patch(
                resource_info!("19_hive_totem.MREA").into(),
                patch_hive_totem_boss_trigger_0_02
            );
            patcher.add_scly_patch(
                resource_info!("05_ice_shorelines.MREA").into(),
                patch_ruined_courtyard_thermal_conduits_0_02
            );
        }

        if version == Version::Pal {
            patcher.add_scly_patch(
                resource_info!("04_mines_pillar.MREA").into(),
                patch_ore_processing_destructible_rock_pal
            );
            patcher.add_scly_patch(
                resource_info!("13_over_burningeffigy.MREA").into(),
                patch_geothermal_core_destructible_rock_pal
            );
            patcher.add_scly_patch(
                resource_info!("01_mines_mainplaza.MREA").into(),
                patch_main_quarry_door_lock_pal
            );
        }

        if spawn_room != SpawnRoom::landing_site_spawn_room() {
            // If we have a non-default start point, patch the landing site to avoid
            // weirdness with cutscene triggers and the ship spawning.
            patcher.add_scly_patch(
                resource_info!("01_over_mainplaza.MREA").into(),
                patch_landing_site_cutscene_triggers
            );
        }

        if config.skip_impact_crater {
            patcher.add_scly_patch(
                resource_info!("01_endcinema.MREA").into(),
                patch_ending_scene_straight_to_credits
            );
        }
    }

    if config.enable_vault_ledge_door {
        let door_type = calculate_door_type("Metroid2.pak",&mut rng,&config.door_weights);
        if !config.is_item_randomized.unwrap_or(false) {
            patcher.add_scly_patch(
                resource_info!("01_mainplaza.MREA").into(),
                move |ps,area| make_main_plaza_locked_door_two_ways(ps,area, door_type,&config)
            );
        }
        if config.patch_map {
            patcher.add_resource_patch(
                resource_info!("01_mainplaza.MAPA").into(),
                move |res| patch_main_plaza_locked_door_map_icon(res,door_type)
            )
        }
    }

    patcher.run(gc_disc)?;
    Ok(())
}
