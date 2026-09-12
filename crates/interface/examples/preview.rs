//! Runs the interface against a simulated world.
//!
//! Lets you iterate on the menus and HUD without a Base RPC connection or
//! peers: fake miners wander around, treasures are "mined" every couple of
//! seconds (draining into a claim at ten), and peers chat now and then.
//!
//! ```bash
//! cargo run -p game_interface --example preview
//! ```
//!
//! Set `PREVIEW_SHOTS=<dir>` to run a scripted tour instead: synthetic key
//! presses walk through the start menu, HUD, chat and pause menu, a
//! screenshot of each is written to `<dir>`, and the app quits.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::PrimaryWindow;
use game_interface::Interface;
use game_primitives::events::GameEvent;
use game_primitives::message::ChatMessage;
use game_primitives::{ExitStatus, Identifier, Player, Position, WorldState};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Grid position
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Pos {
    x: i32,
    y: i32,
}

impl Position for Pos {
    type Unit = i32;

    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn x(&self) -> f64 {
        self.x as f64
    }

    fn y(&self) -> f64 {
        self.y as f64
    }
}

/// Simulated miner
#[derive(Debug, Clone)]
struct Miner {
    id: String,
    pos: Pos,
}

impl Identifier for Miner {
    type Id = String;

    fn identifier(&self) -> Self::Id {
        self.id.clone()
    }
}

impl Player for Miner {
    type Position = Pos;

    fn position(&self) -> Self::Position {
        self.pos
    }
}

/// Simulated world
#[derive(Clone)]
struct MockWorld {
    id: String,
    exit: Arc<ExitStatus>,
    players: Arc<RwLock<HashMap<String, Miner>>>,
    pending: Arc<AtomicU32>,
    messages: Arc<RwLock<Vec<ChatMessage>>>,
}

impl Identifier for MockWorld {
    type Id = String;

    fn identifier(&self) -> Self::Id {
        self.id.clone()
    }
}

impl WorldState for MockWorld {
    type Player = Miner;
    type Message = ChatMessage;
    type MiningBatch = u32;

    fn exit_status(&self) -> Arc<ExitStatus> {
        self.exit.clone()
    }

    fn get_all_players(&self) -> HashMap<Self::Id, Self::Player> {
        self.players.read().unwrap().clone()
    }

    fn get_mining_rewards_count(&self) -> u32 {
        self.pending.load(Ordering::Relaxed)
    }

    fn get_mining_batch(&self) -> HashSet<Self::MiningBatch> {
        HashSet::new()
    }

    fn get_chat_messages(&self) -> Vec<Self::Message> {
        self.messages.read().unwrap().clone()
    }
}

const LOCAL: &str = "0x063049aB85870f97E483567A06BeeA86227d88F9";
const PEERS: [&str; 2] = [
    "0x13455118FFAe8ba8bd94b5D362Db575470a055dF",
    "0x3367dc5DA0F57F69479F3d05DF5f234b4244aaD1",
];
const PEER_LINES: [&str; 4] = [
    "gm miners",
    "difficulty feels lower today",
    "just claimed a batch, lfg",
    "anyone else on the bootnode?",
];

fn main() {
    let (tx, rx) = std::sync::mpsc::channel::<GameEvent<u32, Pos>>();

    let mut players = HashMap::new();
    for (i, id) in std::iter::once(LOCAL).chain(PEERS).enumerate() {
        let pos = Pos::new(i as i32 * 6 - 6, (i as i32 % 2) * 3);
        players.insert(
            id.to_string(),
            Miner {
                id: id.to_string(),
                pos,
            },
        );
    }

    let world = MockWorld {
        id: LOCAL.to_string(),
        exit: Arc::new(ExitStatus::default()),
        players: Arc::new(RwLock::new(players)),
        pending: Arc::new(AtomicU32::new(3)),
        messages: Arc::new(RwLock::new(vec![ChatMessage::new(
            PEERS[0].to_string(),
            "welcome to the pool".to_string(),
        )])),
    };

    std::thread::spawn({
        let world = world.clone();
        move || simulate(world, rx)
    });

    let mut app = Interface::build(tx, world);

    if let Ok(dir) = std::env::var("PREVIEW_SHOTS") {
        std::fs::create_dir_all(&dir).expect("create screenshot directory");
        app.insert_resource(Director::new(dir))
            .add_systems(Update, direct);
    }

    app.run();
}

// ---------------------------------------------------------------------------
// Scripted tour
// ---------------------------------------------------------------------------

/// One scripted action
enum Step {
    /// Capture the primary window to `<dir>/<name>.png`
    Shot(&'static str),
    /// Press a key
    Press(Key, KeyCode),
    /// Type text into the chat field
    Type(&'static str),
}

/// Timed script of actions
#[derive(Resource)]
struct Director {
    dir: String,
    steps: Vec<(f32, Step)>,
    next: usize,
}

impl Director {
    fn new(dir: String) -> Self {
        use Step::*;
        let steps = vec![
            (2.0, Shot("01_menu")),
            (2.5, Press(Key::ArrowDown, KeyCode::ArrowDown)),
            (3.0, Shot("02_menu_quit_selected")),
            (3.5, Press(Key::ArrowUp, KeyCode::ArrowUp)),
            (4.0, Press(Key::Enter, KeyCode::Enter)),
            (4.5, Press(Key::ArrowRight, KeyCode::ArrowRight)),
            (4.7, Press(Key::ArrowRight, KeyCode::ArrowRight)),
            (4.9, Press(Key::ArrowUp, KeyCode::ArrowUp)),
            (6.5, Shot("03_hud")),
            (7.0, Press(Key::Enter, KeyCode::Enter)),
            (7.3, Type("Hello pool, mining from Toronto")),
            (8.0, Shot("04_hud_typing")),
            (8.5, Press(Key::Enter, KeyCode::Enter)),
            (10.0, Shot("05_hud_sent")),
            (10.5, Press(Key::Escape, KeyCode::Escape)),
            (12.0, Shot("06_paused")),
            (12.5, Press(Key::ArrowDown, KeyCode::ArrowDown)),
            (14.5, Press(Key::Enter, KeyCode::Enter)),
        ];

        Self {
            dir,
            steps,
            next: 0,
        }
    }
}

/// Runs due script steps
fn direct(
    time: Res<Time>,
    mut director: ResMut<Director>,
    mut commands: Commands,
    mut keys: MessageWriter<KeyboardInput>,
    window: Query<Entity, With<PrimaryWindow>>,
) {
    let Ok(window) = window.single() else {
        return;
    };

    let mut press = |logical_key: Key, key_code: KeyCode, text: Option<&str>| {
        keys.write(KeyboardInput {
            key_code,
            logical_key,
            state: ButtonState::Pressed,
            text: text.map(smol_str::SmolStr::new),
            repeat: false,
            window,
        });
    };

    while let Some((at, step)) = director.steps.get(director.next)
        && time.elapsed_secs() >= *at
    {
        match step {
            Step::Shot(name) => {
                let path = format!("{}/{name}.png", director.dir);
                info!("Capturing {path}");
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(path));
            }
            Step::Press(key, code) => press(key.clone(), *code, None),
            Step::Type(text) => {
                for c in text.chars() {
                    let s = c.to_string();
                    press(
                        Key::Character(smol_str::SmolStr::new(&s)),
                        KeyCode::KeyA,
                        Some(&s),
                    );
                }
            }
        }
        director.next += 1;
    }
}

/// Drives the fake world: applies UI events, mines, wanders and chats
fn simulate(world: MockWorld, rx: Receiver<GameEvent<u32, Pos>>) {
    let start = Instant::now();
    let mut next_mine = start + Duration::from_secs(2);
    let mut next_chat = start + Duration::from_secs(6);
    let mut next_step = start + Duration::from_millis(400);
    let mut line = 0;

    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(GameEvent::PlayerMovement(delta)) => {
                if let Some(me) = world.players.write().unwrap().get_mut(&world.id) {
                    me.pos.x += delta.x;
                    me.pos.y += delta.y;
                }
            }
            Ok(GameEvent::ChatMessage(message)) => {
                let mut messages = world.messages.write().unwrap();
                messages.push(ChatMessage::new(world.id.clone(), message));
            }
            Ok(GameEvent::Quit) => {
                world.exit.exit();
                return;
            }
            Ok(GameEvent::PlayerFound(_)) => {}
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }

        let now = Instant::now();

        if now >= next_mine {
            next_mine = now + Duration::from_secs(2);
            let pending = world.pending.load(Ordering::Relaxed) + 1;
            world
                .pending
                .store(if pending >= 10 { 0 } else { pending }, Ordering::Relaxed);
        }

        if now >= next_chat {
            next_chat = now + Duration::from_secs(6);
            let peer = PEERS[line % PEERS.len()].to_string();
            let text = PEER_LINES[line % PEER_LINES.len()].to_string();
            world
                .messages
                .write()
                .unwrap()
                .push(ChatMessage::new(peer, text));
            line += 1;
        }

        if now >= next_step {
            next_step = now + Duration::from_millis(400);
            let tick = start.elapsed().as_millis() / 400;
            let mut players = world.players.write().unwrap();
            for (i, peer) in PEERS.iter().enumerate() {
                if let Some(miner) = players.get_mut(*peer) {
                    // Deterministic wander: alternate axes per peer and tick
                    let dir = if (tick / 4 + i as u128).is_multiple_of(2) {
                        1
                    } else {
                        -1
                    };
                    if (tick + i as u128).is_multiple_of(2) {
                        miner.pos.x += dir;
                    } else {
                        miner.pos.y += dir;
                    }
                }
            }
        }
    }
}
