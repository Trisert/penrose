/**
 * penrose :: example configuration
 *
 * penrose does not have a traditional configuration file and is not typically set up by patching
 * the source code: it is more like Xmonad or Qtile in the sense that it is really a library for
 * writing your own window manager. Below is an example main.rs that can serve as a template should
 * you decide to write your own WM using penrose.
 */
#[macro_use]
extern crate penrose;

use penrose::{
    client::Client,
    draw::{dwm_bar, TextStyle, XCBDraw},
    contrib::{
        extensions::Scratchpad,
        hooks::{DefaultWorkspace, LayoutSymbolAsRootName},
        layouts::paper,
    },
    helpers::{index_selectors, spawn},
    hooks::Hook,
    layout::{bottom_stack, side_stack, Layout, LayoutConf},
    Backward, Config, Forward, Less, More, Result, Selector, WindowManager, XcbConnection,
};

use simplelog::{LevelFilter, SimpleLogger};
use std::collections::HashMap;

// An example of a simple custom hook. In this case we are creating a NewClientHook which will
// be run each time a new client program is spawned.
struct MyClientHook {}
impl Hook for MyClientHook {
    fn new_client(&mut self, wm: &mut WindowManager, c: &mut Client) {
        wm.log(&format!("new client with WM_CLASS='{}'", c.wm_class()));
    }
}

const HEIGHT: usize = 18;
const PROFONT: &str = "JetBrainsMono Nerd Font";

const BLACK: u32 = 0x282828ff; 
const GREY: u32 = 0x7e7e7eff;
const WHITE: u32 = 0xebdbb2ff;
const ORANGE: u32 = 0xf59342ff;

fn main() -> Result<()> {
    // penrose will log useful information about the current state of the WindowManager during
    // normal operation that can be used to drive scripts and related programs. Additional debug
    // output can be helpful if you are hitting issues.
    SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default())?;

    // Config structs can be intiialised directly as all fields are public.
    // A default config is provided which sets sensible (but minimal) values for each field.
    let mut config = Config::default();

    // Created at startup. See keybindings below for how to access them
    config.workspaces = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"];

    // Windows with a matching WM_CLASS will always float
    config.floating_classes = &["dmenu", "dunst", "polybar", "rofi"];

    // Client border colors are set based on X focus
    config.focused_border = 0xf59342; // #cc241d
    config.unfocused_border = 0x3c3836; // #3c3836

    config.hooks.push(Box::new(dwm_bar(
                Box::new(XCBDraw::new()?),
                HEIGHT,
                &TextStyle {
                    font: PROFONT.to_string(),
                    point_size: 11,
                    fg: WHITE.into(),
                    bg: Some(BLACK.into()),
                    padding: (2.0, 2.0),
                },
                ORANGE,
                GREY,
                &config.workspaces,
            )?));

    // When specifying a layout, most of the time you will want LayoutConf::default() as shown
    // below, which will honour gap settings and will not be run on focus changes (only when
    // clients are added/removed). To customise when/how each layout is applied you can create a
    // LayoutConf instance with your desired properties enabled.
    let follow_focus_conf = LayoutConf {
        floating: false,
        gapless: true,
        follow_focus: true,
        allow_wrapping: true,
    };

    // Defauly number of clients in the main layout area
    let n_main = 1;

    // Default percentage of the screen to fill with the main area of the layout
    let ratio = 0.55;

    // Layouts to be used on each workspace. Currently all workspaces have the same set of Layouts
    // available to them, though they track modifications to n_main and ratio independently.
    config.layouts = vec![
        Layout::new("[side]", LayoutConf::default(), side_stack, n_main, ratio),
        Layout::new("[botm]", LayoutConf::default(), bottom_stack, n_main, ratio),
        Layout::new("[papr]", follow_focus_conf, paper, n_main, ratio),
        Layout::floating("[----]"),
    ];

    // NOTE: change these to programs that you have installed!
    let my_program_launcher = "rofi -combi-modi run,drun,window -show combi";
    let my_file_manager = "alacritty -e ranger";
    let my_terminal = "st";
    let my_browser = "firefox";

    /* hooks
     *
     * penrose provides several hook points where you can run your own code as part of
     * WindowManager methods. This allows you to trigger custom code without having to use a key
     * binding to do so. See the hooks module in the docs for details of what hooks are avaliable
     * and when/how they will be called. Note that each class of hook will be called in the order
     * that they are defined. Hooks may maintain their own internal state which they can use to
     * modify their behaviour if desired.
     */
    config.hooks.push(Box::new(MyClientHook {}));

    // Using a simple contrib hook that takes no config. By convention, contrib hooks have a 'new'
    // method that returns a boxed instance of the hook with any configuration performed so that it
    // is ready to push onto the corresponding *_hooks vec.
    config.hooks.push(LayoutSymbolAsRootName::new());

    // Here we are using a contrib hook that requires configuration to set up a default workspace
    // on workspace "9". This will set the layout and spawn the supplied programs if we make
    // workspace "9" active while it has no clients.
    config.hooks.push(DefaultWorkspace::new(
        "9",
        "[botm]",
        vec![my_terminal, my_terminal, my_file_manager],
    ));

    // Scratchpad is an extension: it makes use of the same Hook points as the examples above but
    // additionally provides a 'toggle' method that can be bound to a key combination in order to
    // trigger the bound scratchpad client.
    let sp = Scratchpad::new(my_terminal, 0.8, 0.8);
    sp.register(&mut config);

    let fr = Scratchpad::new("firefox", 0.9, 0.9);
    fr.register(&mut config);

    /* The gen_keybindings macro parses user friendly key binding definitions into X keycodes and
     * modifier masks. It uses the 'xmodmap' program to determine your current keymap and create
     * the bindings dynamically on startup. If this feels a little too magical then you can
     * alternatively construct a  HashMap<KeyCode, FireAndForget> manually with your chosen
     * keybindings (see helpers.rs and data_types.rs for details).
     * FireAndForget functions do not need to make use of the mutable WindowManager reference they
     * are passed if it is not required: the run_external macro ignores the WindowManager itself
     * and instead spawns a new child process.
     */
    let key_bindings = gen_keybindings! {
        // Program launch
        "M-p" => run_external!(my_program_launcher);
        "M-S-Return" => run_external!(my_terminal);
        "M-f" => run_external!(my_file_manager);
        "M-s" => run_external!(my_browser);
        "M-a" => run_external!("slock");
        "M-S-p" => run_external!("picomdmenu");
        "XF86AudioRaiseVolume" => run_external!("pactl set-sink-volume @DEFAULT_SINK@ +10%");
        "XF86AudioLowerVolume" => run_external!("pactl set-sink-volume @DEFAULT_SINK@ -10%");
        "M-S-l" => run_external!("cmpdmenu");

        // client management
        "M-j" => run_internal!(cycle_client, Forward);
        "M-k" => run_internal!(cycle_client, Backward);
        "M-S-j" => run_internal!(drag_client, Forward);
        "M-S-k" => run_internal!(drag_client, Backward);
        "M-S-c" => run_internal!(kill_client);
        "M-S-f" => run_internal!(toggle_client_fullscreen, &Selector::Focused);
        "M-slash" => sp.toggle();
        "M-comma" => fr.toggle();

        // workspace management
        "M-Tab" => run_internal!(toggle_workspace);
        "M-bracketright" => run_internal!(cycle_screen, Forward);
        "M-bracketleft" => run_internal!(cycle_screen, Backward);
        "M-S-bracketright" => run_internal!(drag_workspace, Forward);
        "M-S-bracketleft" => run_internal!(drag_workspace, Backward);

        // Layout management
        "M-period" => run_internal!(cycle_layout, Forward);
        "M-S-period" => run_internal!(cycle_layout, Backward);
        "M-A-Up" => run_internal!(update_max_main, More);
        "M-A-Down" => run_internal!(update_max_main, Less);
        "M-A-Right" => run_internal!(update_main_ratio, More);
        "M-A-Left" => run_internal!(update_main_ratio, Less);

        "M-A-s" => run_internal!(detect_screens);
        "M-S-Escape" => run_internal!(exit);

        // Each keybinding here will be templated in with the workspace index of each workspace,
        // allowing for common workspace actions to be bound at once.
        refmap [ config.ws_range() ] in {
            "M-{}" => focus_workspace [ index_selectors(config.workspaces.len()) ];
            "M-S-{}" => client_to_workspace [ index_selectors(config.workspaces.len()) ];
        };
    };

    // The underlying connection to the X server is handled as a trait: XConn. XcbConnection is the
    // reference implementation of this trait that uses the XCB library to communicate with the X
    // server. You are free to provide your own implementation if you wish, see xconnection.rs for
    // details of the required methods and expected behaviour.
    let conn = XcbConnection::new()?;

    // Create the WindowManager instance with the config we have built and a connection to the X
    // server. Before calling grab_keys_and_run, it is possible to run additional start-up actions
    // such as configuring initial WindowManager state, running custom code / hooks or spawning
    // external processes such as a start-up script.
    let mut wm = WindowManager::init(config, &conn);

    spawn("wmname LG3D");
    spawn("setxkbmap it");
    spawn("wal -R");
    spawn("picom");

    // grab_keys_and_run will start listening to events from the X server and drop into the main
    // event loop. From this point on, program control passes to the WindowManager so make sure
    // that any logic you wish to run is done before here!
    wm.grab_keys_and_run(key_bindings, HashMap::new());

    Ok(())
}
