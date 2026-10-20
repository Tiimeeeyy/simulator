//! Stylized modern Main Menu screen.

use bevy_egui::egui::{
    self, Color32, RichText, Stroke, Ui, Vec2,
};

use crate::theme::*;

pub enum MenuAction {
    StartDuelTearlaments,
    StartDuelGeneric,
    OpenDeckBuilder,
    OpenBanlistBrowser,
    Exit,
}

pub fn render_main_menu(
    ui: &mut Ui,
    card_count: usize,
    banlist_count: usize,
) -> Option<MenuAction> {
    let mut action = None;
    let screen_h = ui.available_height();

    ui.vertical_centered(|ui| {
        ui.add_space((screen_h * 0.12).max(20.0));

        egui::Frame::new()
            .fill(BG_PANEL)
            .stroke(Stroke::new(1.5, ACCENT_GOLD))
            .corner_radius(10.0)
            .inner_margin(28.0)
            .show(ui, |ui| {
                ui.set_max_width(440.0);
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.heading(
                        RichText::new("⚔️ YU-GI-OH! DESKTOP")
                            .color(ACCENT_GOLD)
                            .size(26.0)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("Native High-Performance ocgcore Simulator")
                            .color(ACCENT_CYAN)
                            .size(12.0),
                    );
                    ui.label(
                        RichText::new("MR5 Rules Engine • Date-Aware Banlists • Dual AI")
                            .color(TEXT_MUTED)
                            .size(11.0),
                    );

                    ui.add_space(24.0);

                    let btn_size = Vec2::new(360.0, 42.0);

                    // Button 1: Duel vs Ishizu Tearlaments
                    if ui
                        .add_sized(
                            btn_size,
                            egui::Button::new(
                                RichText::new("⚔️  Duel vs Ishizu Tearlaments AI")
                                    .size(14.0)
                                    .color(Color32::WHITE)
                                    .strong(),
                            )
                            .fill(Color32::from_rgb(120, 45, 140)),
                        )
                        .clicked()
                    {
                        action = Some(MenuAction::StartDuelTearlaments);
                    }

                    ui.add_space(10.0);

                    // Button 2: Duel vs Generic AI
                    if ui
                        .add_sized(
                            btn_size,
                            egui::Button::new(
                                RichText::new("🤖  Duel vs Generic Bot")
                                    .size(14.0)
                                    .color(Color32::WHITE)
                                    .strong(),
                            )
                            .fill(Color32::from_rgb(45, 90, 160)),
                        )
                        .clicked()
                    {
                        action = Some(MenuAction::StartDuelGeneric);
                    }

                    ui.add_space(10.0);

                    // Button 3: Deck Builder
                    if ui
                        .add_sized(
                            btn_size,
                            egui::Button::new(
                                RichText::new("🃏  Deck Builder")
                                    .size(14.0)
                                    .color(Color32::WHITE)
                                    .strong(),
                            )
                            .fill(Color32::from_rgb(35, 120, 80)),
                        )
                        .clicked()
                    {
                        action = Some(MenuAction::OpenDeckBuilder);
                    }

                    ui.add_space(10.0);

                    // Button 4: Banlist Time-Machine
                    if ui
                        .add_sized(
                            btn_size,
                            egui::Button::new(
                                RichText::new("🕒  Banlist Time-Machine")
                                    .size(14.0)
                                    .color(Color32::WHITE)
                                    .strong(),
                            )
                            .fill(Color32::from_rgb(140, 100, 30)),
                        )
                        .clicked()
                    {
                        action = Some(MenuAction::OpenBanlistBrowser);
                    }

                    ui.add_space(10.0);

                    // Button 5: Exit
                    if ui
                        .add_sized(
                            btn_size,
                            egui::Button::new(
                                RichText::new("Exit")
                                    .size(13.0)
                                    .color(TEXT_MUTED),
                            )
                            .fill(Color32::from_rgb(30, 35, 45)),
                        )
                        .clicked()
                    {
                        action = Some(MenuAction::Exit);
                    }

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // Footer statistics
                    ui.label(
                        RichText::new(format!(
                            "Card Database: {} cards loaded  |  {} Historical Formats",
                            card_count, banlist_count
                        ))
                        .size(10.0)
                        .color(TEXT_MUTED),
                    );
                });
            });
    });

    action
}
