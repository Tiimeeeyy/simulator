//! High-Performance Desktop Yu-Gi-Oh! Simulator Binary.
//!
//! Powered by Bevy + `bevy_egui` with:
//! - MR5 ocgcore C++ rules engine and Lua 5.4 scripts
//! - Date-aware TCG/OCG historical Banlist Time-Machine (2002 - Oct 2026)
//! - Dual AI: Native Rust `TearlamentsPolicy` & Generic Bot + WindBot TCP bridge
//! - Full-featured virtualized 14,000+ card Deck Builder

mod banlist_browser;
mod card_widget;
mod deckbuilder;
mod duel_ui;
mod menu;
mod theme;

use std::path::PathBuf;
use std::sync::Arc;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use chrono::NaiveDate;
use ygopro_core::ScriptLoader;
use ygopro_data::{BanlistRegistry, CardDatabase, ReleaseDateIndex};

use crate::banlist_browser::{render_banlist_browser, BanlistBrowserState};
use crate::deckbuilder::{render_deck_builder, DeckBuilderState};
use crate::duel_ui::{render_duel_screen, BotOpponentType, DuelUiState};
use crate::menu::{render_main_menu, MenuAction};
use crate::theme::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    DeckBuilder,
    Duel,
    BanlistBrowser,
}

#[derive(Resource)]
pub struct ClientResources {
    pub db: Arc<CardDatabase>,
    pub banlists: Arc<BanlistRegistry>,
    pub rel_idx: Arc<ReleaseDateIndex>,
    pub script_loader: Arc<ScriptLoader>,
    pub app_state: AppState,
    pub deck_builder: DeckBuilderState,
    pub duel_ui: DuelUiState,
    pub banlist_browser: BanlistBrowserState,
}

fn resolve_data_paths() -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut data_dir = base.join("data");
    if !data_dir.exists() {
        if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
            data_dir = PathBuf::from(manifest).join("../../data");
        }
    }

    let cdb_path = data_dir.join("cards.cdb");
    let lflists_dir = data_dir.join("lflists");
    let rel_path = data_dir.join("release_dates.csv");
    let scripts_dir = data_dir.join("scripts");

    (cdb_path, lflists_dir, rel_path, scripts_dir)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("Yu-Gi-Oh! Desktop Simulator");
        println!("Usage: ygopro [OPTIONS]");
        println!("Options:");
        println!("  --headless       Run headless bot self-play validation and exit");
        println!("  --help, -h       Display this help message");
        return;
    }

    let (cdb_path, lflists_dir, rel_path, scripts_dir) = resolve_data_paths();

    println!("Loading card database from: {}", cdb_path.display());
    let db = Arc::new(CardDatabase::open(&cdb_path).expect("failed to open cards.cdb"));
    println!("Loaded {} cards into memory.", db.len());

    println!("Loading banlists from: {}", lflists_dir.display());
    let banlists = Arc::new(BanlistRegistry::load_dir(&lflists_dir).expect("failed to load banlists"));
    println!("Loaded {} dated banlists.", banlists.len());

    let rel_idx = Arc::new(
        ReleaseDateIndex::load_file(&rel_path).unwrap_or_else(|_| ReleaseDateIndex::default()),
    );
    let script_loader = Arc::new(ScriptLoader::new().with_dir(&scripts_dir));

    if args.iter().any(|a| a == "--headless") {
        println!("Running headless self-play test...");
        let config = ygopro_core::DuelConfig::default();
        let mut duel = ygopro_core::Duel::create(config, Arc::clone(&db), Arc::clone(&script_loader))
            .expect("failed to create duel");
        let deck = ygopro_data::builtin::tearlaments_ishizu_deck();
        duel.load_deck(0, &deck);
        duel.load_deck(1, &deck);

        let p0 = ygopro_host::PlayerController::Bot(Box::new(ygopro_bot::TearlamentsPolicy::new(0)));
        let p1 = ygopro_host::PlayerController::Bot(Box::new(ygopro_bot::GenericAgent::new(1)));
        let mut session = ygopro_host::DuelSession::new(duel, Arc::clone(&db), p0, p1);

        for step_i in 0..60 {
            if let Ok(outcome) = session.step() {
                match outcome {
                    ygopro_host::StepOutcome::Ended { winner, .. } => {
                        println!("Headless match ended on step {step_i}! Winner: {winner:?}");
                        break;
                    }
                    _ => {}
                }
            }
        }
        println!("Headless duel completed successfully.");
        return;
    }

    // Launch Bevy Desktop Client
    let initial_banlist = banlists.names().first().cloned().unwrap_or_default();
    let initial_date = NaiveDate::from_ymd_opt(2023, 3, 1).unwrap();

    let deck_builder = DeckBuilderState::new(&initial_banlist, initial_date);
    let duel_ui = DuelUiState::new();
    let banlist_browser = BanlistBrowserState::new();

    let resources = ClientResources {
        db,
        banlists,
        rel_idx,
        script_loader,
        app_state: AppState::MainMenu,
        deck_builder,
        duel_ui,
        banlist_browser,
    };

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Yu-Gi-Oh! Desktop Simulator".into(),
                        resolution: WindowResolution::new(1366, 768),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin::default())
        .insert_resource(resources)
        .add_systems(Update, client_ui_system)
        .run();
}

fn client_ui_system(mut contexts: EguiContexts, mut res: ResMut<ClientResources>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let res = &mut *res;

    // Full screen container window
    let screen_rect = ctx.viewport_rect();
    egui::Window::new("RootContainer")
        .title_bar(false)
        .resizable(false)
        .movable(false)
        .fixed_pos(screen_rect.min)
        .fixed_size(screen_rect.size())
        .frame(egui::Frame::new().fill(BG_DARK))
        .show(ctx, |ui| {
            match res.app_state {
                AppState::MainMenu => {
                    let card_count = res.db.len();
                    let banlist_count = res.banlists.len();
                    if let Some(action) = render_main_menu(ui, card_count, banlist_count) {
                        match action {
                            MenuAction::StartDuelTearlaments => {
                                res.duel_ui.bot_type = BotOpponentType::IshizuTearlaments;
                                let deck = res.deck_builder.deck.clone();
                                let db = Arc::clone(&res.db);
                                let loader = Arc::clone(&res.script_loader);
                                res.duel_ui.start_match(&deck, db, loader);
                                res.app_state = AppState::Duel;
                            }
                            MenuAction::StartDuelGeneric => {
                                res.duel_ui.bot_type = BotOpponentType::Generic;
                                let deck = res.deck_builder.deck.clone();
                                let db = Arc::clone(&res.db);
                                let loader = Arc::clone(&res.script_loader);
                                res.duel_ui.start_match(&deck, db, loader);
                                res.app_state = AppState::Duel;
                            }
                            MenuAction::OpenDeckBuilder => {
                                res.app_state = AppState::DeckBuilder;
                            }
                            MenuAction::OpenBanlistBrowser => {
                                res.app_state = AppState::BanlistBrowser;
                            }
                            MenuAction::Exit => {
                                std::process::exit(0);
                            }
                        }
                    }
                }
                AppState::DeckBuilder => {
                    let mut start_duel = false;
                    render_deck_builder(
                        ui,
                        &mut res.deck_builder,
                        &res.db,
                        &res.banlists,
                        &res.rel_idx,
                        &mut start_duel,
                    );
                    if start_duel {
                        let deck = res.deck_builder.deck.clone();
                        let db = Arc::clone(&res.db);
                        let loader = Arc::clone(&res.script_loader);
                        res.duel_ui.start_match(&deck, db, loader);
                        res.app_state = AppState::Duel;
                    }
                }
                AppState::Duel => {
                    let mut return_to_menu = false;
                    render_duel_screen(ui, &mut res.duel_ui, &res.db, &mut return_to_menu);
                    if return_to_menu {
                        res.app_state = AppState::MainMenu;
                    }
                }
                AppState::BanlistBrowser => {
                    let mut return_to_menu = false;
                    render_banlist_browser(
                        ui,
                        &mut res.banlist_browser,
                        &res.banlists,
                        &res.db,
                        &mut return_to_menu,
                    );
                    if return_to_menu {
                        res.app_state = AppState::MainMenu;
                    }
                }
            }
        });
}
