use clap::{
    Arg,
    ArgGroup,
    App,
    Format, // XXX This is an undocumented enum
    crate_version,
};

use randomprime::{
    door_meta::Weights, extract_flaahgra_music_files, parse_layout, patches, reader_writer, structs
};

use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    fs,
    panic,
    process::Command,
};

use serde::{Deserialize};


struct ProgressNotifier
{
    total_size: usize,
    bytes_so_far: usize,
    quiet: bool,
}

impl ProgressNotifier
{
    fn new(quiet: bool) -> ProgressNotifier
    {
        ProgressNotifier {
            total_size: 0,
            bytes_so_far: 0,
            quiet,
        }
    }
}

impl structs::ProgressNotifier for ProgressNotifier
{
    fn notify_total_bytes(&mut self, total_size: usize)
    {
        self.total_size = total_size
    }

    fn notify_writing_file(&mut self, file_name: &reader_writer::CStr, file_bytes: usize)
    {
        if self.quiet {
            return;
        }
        let percent = self.bytes_so_far as f64 / self.total_size as f64 * 100.;
        println!("{:02.0}% -- Writing file {:?}", percent, file_name);
        self.bytes_so_far += file_bytes;
    }

    fn notify_writing_header(&mut self)
    {
        if self.quiet {
            return;
        }
        let percent = self.bytes_so_far as f64 / self.total_size as f64 * 100.;
        println!("{:02.0}% -- Writing ISO header", percent);
    }

    fn notify_flushing_to_disk(&mut self)
    {
        if self.quiet {
            return;
        }
        println!("Flushing written data to the disk...");
    }

    fn notify_stacking_warning(&mut self)
    {
        if self.quiet {
            return;
        }
        println!("Item randomized game. Skipping item randomizer configuration.");
    }
}

fn default_as_false() -> bool {
    false
}

fn default_as_empty_str_vec() -> Vec<String> {
    Vec::new()
}

fn default_as_empty_bool_vec() -> Vec<bool> {
    Vec::new()
}

fn default_as_empty_liquid_volume_vec() -> Vec<patches::LiquidVolume> {
    Vec::new()
}

fn default_as_empty_aether_transform_vec() -> Vec<patches::AetherTransform> {
    Vec::new()
}

fn default_empty_string() -> String {
    "".to_string()
}

fn default_u64_123456789() -> u64 {
    123456789
}

#[derive(Deserialize)]
struct PatchConfig {
    skip_frigate: bool,
    skip_crater: bool,
    fix_flaaghra_music: bool,
    trilogy_iso: Option<String>,
    varia_heat_protection: bool,
    stagger_suit_damage: bool,
    skip_hudmemos: bool,
    powerbomb_lockpick: bool,
    enable_one_way_doors: bool,
    patch_map: bool,
    obfuscate_items:bool,
    artifact_hints:String,
    auto_enabled_elevators:bool,
    
    #[serde(default = "default_as_false")]
    patch_vertical_to_blue:bool,
    
    #[serde(default = "default_as_false")]
    patch_power_conduits: bool,

    #[serde(default = "default_as_false")]
    tiny_elvetator_samus: bool,

    #[serde(default = "default_as_false")]
    remove_missile_locks: bool,

    #[serde(default = "default_as_false")]
    remove_frigidite_lock: bool,

    #[serde(default = "default_as_false")]
    remove_mine_security_station_locks: bool,

    #[serde(default = "default_as_false")]
    lower_mines_backwards: bool,
}

#[derive(Deserialize)]
struct Config {
    input_iso: String,
    output_iso: String,
    layout_string: String,

    #[serde(default = "default_as_empty_str_vec")]
    elevator_layout_override: Vec<String>,

    #[serde(default = "default_as_empty_bool_vec")]
    missile_lock_override: Vec<bool>,

    #[serde(default = "default_as_empty_str_vec")]
    superheated_rooms: Vec<String>,

    #[serde(default = "default_as_empty_str_vec")]
    drain_liquid_rooms: Vec<String>,

    #[serde(default = "default_as_empty_str_vec")]
    underwater_rooms: Vec<String>,

    #[serde(default = "default_as_empty_liquid_volume_vec")]
    liquid_volumes: Vec<patches::LiquidVolume>,

    #[serde(default = "default_as_empty_aether_transform_vec")]
    aether_transforms: Vec<patches::AetherTransform>,
    
    #[serde(default = "default_empty_string")]
    new_save_spawn_room: String,

    #[serde(default = "default_empty_string")]
    frigate_done_spawn_room: String,

    seed: u64,
    door_weights: Weights,
    patch_settings: PatchConfig,
    
    #[serde(default = "default_u64_123456789")]
    starting_pickups: u64,

    #[serde(default = "default_u64_123456789")]
    new_save_starting_items: u64,

    #[serde(default = "default_u64_123456789")]
    frigate_done_starting_items: u64,
    
    excluded_doors: [HashMap<String,Vec<String>>;7],
}

#[derive(Deserialize)]
struct ConfigBanner
{
    game_name: Option<String>,
    developer: Option<String>,

    game_name_full: Option<String>,
    developer_full: Option<String>,
    description: Option<String>,
}

fn get_config() -> Result<patches::ParsedConfig, String>
{
    /*let matches = App::new("randomprime ISO patcher")
        .version(crate_version!())
        .arg(Arg::with_name("input iso path")
            .long("input-iso")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("output iso path")
            .long("output-iso")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("pickup layout")
            .long("layout")
            .required(true)
            .takes_value(true)
            .allow_hyphen_values(true))*/


    let matches = App::new("randomprime ISO patcher")
        .version(crate_version!())
        .arg(Arg::with_name("input iso path")
            .long("input-iso")
            .takes_value(true))
        .arg(Arg::with_name("output iso path")
            .long("output-iso")
            .takes_value(true))
        .arg(Arg::with_name("profile json path")
            .long("profile")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("skip frigate")
            .long("skip-frigate")
            .help("New save files will skip the \"Space Pirate Frigate\" tutorial level"))
        .arg(Arg::with_name("skip hudmenus")
            .long("non-modal-item-messages")
            .help("Display a non-modal message when an item is is acquired"))
        .arg(Arg::with_name("nonvaria heat damage")
            .long("nonvaria-heat-damage")
            .help("If the Varia Suit has not been collect, heat damage applies"))
        .arg(Arg::with_name("staggered suit damage")
            .long("staggered-suit-damage")
            .help(concat!("The suit damage reduction is determinted by the number of suits ",
                            "collected rather than the most powerful one collected.")))
        .arg(Arg::with_name("auto enabled elevators")
            .long("auto-enabled-elevators")
            .help("Every elevator will be automatically enabled without scaning its terminal"))
        .arg(Arg::with_name("skip impact crater")
            .long("skip-impact-crater")
            .help("Elevators to the Impact Crater immediately go to the game end sequence"))
        .arg(Arg::with_name("enable vault ledge door")
            .long("enable-vault-ledge-door")
            .help("Enable Chozo Ruins Vault door from Main Plaza"))

        .arg(Arg::with_name("all artifact hints")
            .long("all-artifact-hints")
            .help("All artifact location hints are available immediately"))
        .arg(Arg::with_name("no artifact hints")
            .long("no-artifact-hints")
            .help("Artifact location hints are disabled"))
        .group(ArgGroup::with_name("artifact hint behavior")
               .args(&["all artifact hints", "no artifact hints"]))

        .arg(Arg::with_name("trilogy disc path")
            .long("flaahgra-music-disc-path")
            .help(concat!("Location of a ISO of Metroid Prime Trilogy. If provided the ",
                            "Flaahgra fight music will be used to replace the original"))
            .takes_value(true))
        .arg(Arg::with_name("keep attract mode")
            .long("keep-attract-mode")
            .help("Keeps the attract mode FMVs, which are removed by default"))
        .arg(Arg::with_name("obfuscate items")
            .long("obfuscate-items")
            .help("Replace all item models with an obfuscated one"))
        .arg(Arg::with_name("quiet")
            .long("quiet")
            .help("Don't print the progress messages"))
        .arg(Arg::with_name("main menu message")
            .long("main-menu-message")
            .hidden(true)
            .takes_value(true))
        .arg(Arg::with_name("change starting items")
            .long("starting-items")
            .hidden(true)
            .takes_value(true)
            .validator(|s| s.parse::<u64>().map(|_| ())
                                        .map_err(|_| "Expected an integer".to_string())))
        .arg(Arg::with_name("quickplay")
            .long("quickplay")
            .hidden(true))
        .arg(Arg::with_name("text file comment")
                .long("text-file-comment")
                .hidden(true)
                .takes_value(true))

        .arg(Arg::with_name("pal override")
            .long("pal-override")
            .hidden(true))
        .get_matches();

    let json_path = matches.value_of("profile json path").unwrap();
    let input_json:&str = &fs::read_to_string(json_path)
                .map_err(|e| format!("Could not read JSON file: {}",e)).unwrap();

    let config:Config = serde_json::from_str(input_json)
                .map_err(|e| format!("Could not parse JSON file: {}",e)).unwrap();
    let input_iso_path = config.input_iso;
    let input_iso_file = File::open(input_iso_path)
                .map_err(|e| format!("Failed to open input iso: {}", e))?;
    let input_iso_mmap = unsafe { memmap::Mmap::map(&input_iso_file) }
                .map_err(|e| format!("Failed to open input iso: {}", e))?;

    let output_iso_path = config.output_iso;
    let out_iso = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&output_iso_path)
        .map_err(|e| format!("Failed to open output file: {}", e))?;

    let iso_format = if output_iso_path.ends_with(".gcz") {
        patches::IsoFormat::Gcz
    } else if output_iso_path.ends_with(".ciso") {
        patches::IsoFormat::Ciso
    } else {
        patches::IsoFormat::Iso
    };

    let layout_string = String::from(&config.layout_string);
    let (pickup_layout, elevator_layout, item_seed) = parse_layout(&layout_string)?;

    let seed = config.seed;

    let artifact_hints = String::from(&config.patch_settings.artifact_hints);
    let artifact_hint_behavior = if artifact_hints == "default" {
        patches::ArtifactHintBehavior::Default
    } else if artifact_hints == "none" {
        patches::ArtifactHintBehavior::None
    } else { // e.g. "all"
        patches::ArtifactHintBehavior::All
        
    };

    let flaahgra_music_files = if config.patch_settings.fix_flaaghra_music {
        if let Some(path) = config.patch_settings.trilogy_iso {
            Some(extract_flaahgra_music_files(&path)?)
        } else {
            None
        }
    } else {
        None
    };

    let mpdr_version = "Plando v1.7";
    let mut comment_message:String = "Generated with ".to_owned();
    comment_message.push_str(mpdr_version);

    let mut banner = Some(ConfigBanner {
        game_name: Some(String::from("Metroid Prime")),
        developer: Some(String::from("^_^")),

        game_name_full: Some(String::from("Metroid Prime Plandomizer")),
        developer_full: Some(String::from("^_^")),
        description: Some(String::from("Metroid Prime, but probably a cursed seed")),
    });

    let new_save_starting_items = {
        if config.new_save_starting_items != 123456789 {
            config.new_save_starting_items
        }
        else if config.starting_pickups != 123456789 {
            config.starting_pickups
        }
        else {
            assert!(false);
            0
        }
    };
    
    let frigate_done_starting_items = {
        if config.frigate_done_starting_items != 123456789 {
            config.frigate_done_starting_items
        }
        else if config.starting_pickups != 123456789 {
            config.starting_pickups
        }
        else {
            assert!(false);
            0
        }
    };

    Ok(patches::ParsedConfig {
        input_iso:input_iso_mmap,
        output_iso:out_iso,
        is_item_randomized: None,
        pickup_layout, elevator_layout, seed,
        item_seed,door_weights:config.door_weights,
        excluded_doors:config.excluded_doors,
        patch_map:config.patch_settings.patch_map,
        patch_power_conduits: config.patch_settings.patch_power_conduits,
        remove_missile_locks: config.patch_settings.remove_missile_locks,
        remove_frigidite_lock: config.patch_settings.remove_frigidite_lock,
        remove_mine_security_station_locks: config.patch_settings.remove_mine_security_station_locks,
        lower_mines_backwards: config.patch_settings.lower_mines_backwards,
        superheated_rooms: config.superheated_rooms,
        drain_liquid_rooms: config.drain_liquid_rooms,
        underwater_rooms: config.underwater_rooms,
        liquid_volumes: config.liquid_volumes,
        aether_transforms: config.aether_transforms,
        

        layout_string,
        elevator_layout_override: config.elevator_layout_override,
        missile_lock_override: config.missile_lock_override,
        new_save_spawn_room: config.new_save_spawn_room,
        frigate_done_spawn_room: config.frigate_done_spawn_room,

        iso_format,
        skip_frigate: config.patch_settings.skip_frigate,
        skip_hudmenus: config.patch_settings.skip_hudmemos,
        nonvaria_heat_damage: config.patch_settings.varia_heat_protection,
        staggered_suit_damage: config.patch_settings.stagger_suit_damage,
        powerbomb_lockpick: config.patch_settings.powerbomb_lockpick,
        keep_fmvs: false,
        obfuscate_items: config.patch_settings.obfuscate_items,
        auto_enabled_elevators: config.patch_settings.auto_enabled_elevators,
        quiet: false,

        skip_impact_crater: config.patch_settings.skip_crater,
        enable_vault_ledge_door: config.patch_settings.enable_one_way_doors,
        artifact_hint_behavior,
        patch_vertical_to_blue: config.patch_settings.patch_vertical_to_blue,
        tiny_elvetator_samus: config.patch_settings.tiny_elvetator_samus,

        flaahgra_music_files,

        new_save_starting_items,
        frigate_done_starting_items,

        comment: comment_message,
        main_menu_message: String::from(mpdr_version),

        quickplay: false,

        bnr_game_name: banner.as_mut().and_then(|b| b.game_name.take()),
        bnr_developer: banner.as_mut().and_then(|b| b.developer.take()),

        bnr_game_name_full: banner.as_mut().and_then(|b| b.game_name_full.take()),
        bnr_developer_full: banner.as_mut().and_then(|b| b.developer_full.take()),
        bnr_description: banner.as_mut().and_then(|b| b.description.take()),

        pal_override: false,
    })

}



#[cfg(windows)]
fn was_launched_by_windows_explorer() -> bool
{
    // https://stackoverflow.com/a/513574
    use winapi::um::processenv:: *;
    use winapi::um::winbase:: *;
    use winapi::um::wincon:: *;
    static mut CACHED: Option<bool> = None;
    unsafe {
        if let Some(t) = CACHED {
            return t;
        }
        let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
        let x = GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), &mut csbi);
        CACHED = Some(x == 1 && csbi.dwCursorPosition.X == 0 && csbi.dwCursorPosition.Y == 0);
        CACHED.unwrap()
    }
}

#[cfg(not(windows))]
fn was_launched_by_windows_explorer() -> bool
{
    false
}

fn maybe_pause_at_exit()
{
    if was_launched_by_windows_explorer() {
        // XXX Windows only
        let _ = Command::new("cmd.exe").arg("/c").arg("pause").status();
    }
}

fn main_inner() -> Result<(), String>
{
    let config = get_config()?;
    let pn = ProgressNotifier::new(config.quiet);
    patches::patch_iso(config, pn)?;
    println!("Done");
    Ok(())
}

fn main()
{
    // XXX We have to check this before we print anything; it relies on the cursor position and
    //     caches its result.
    was_launched_by_windows_explorer();

    // On non-debug builds, suppress the default panic message and print a more helpful and
    // user-friendly one
    if !cfg!(debug_assertions) {
        panic::set_hook(Box::new(|_| {
            let _ = eprintln!("{} \
An error occurred while parsing the input ISO. \
This most likely means your ISO is corrupt. \
Please verify that your ISO matches one of the following hashes:
MD5:  eeacd0ced8e2bae491eca14f141a4b7c
SHA1: ac20c744db18fdf0339f37945e880708fd317231
", Format::Error("error:"));

            maybe_pause_at_exit();
        }));
    }

    match main_inner() {
        Err(s) => eprintln!("{} {}", Format::Error("error:"), s),
        Ok(()) => (),
    };

    maybe_pause_at_exit();
}
