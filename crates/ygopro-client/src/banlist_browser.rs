//! Interactive Banlist Time-Machine format browser.

use bevy_egui::egui::{
    self, Layout, RichText, ScrollArea, Ui, Vec2,
};
use ygopro_data::{BanlistRegistry, CardDatabase};

use crate::card_widget::draw_card_item;
use crate::theme::*;

pub struct BanlistBrowserState {
    pub selected_index: usize,
    pub search_query: String,
    pub filter_limit: Option<u8>, // None = all, Some(0) = Forbidden, Some(1) = Limited, etc.
}

impl BanlistBrowserState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            search_query: String::new(),
            filter_limit: None,
        }
    }
}

pub fn render_banlist_browser(
    ui: &mut Ui,
    state: &mut BanlistBrowserState,
    banlists: &BanlistRegistry,
    db: &CardDatabase,
    return_to_menu: &mut bool,
) {
    let screen_w = ui.available_width();
    let screen_h = ui.available_height();

    // Top Bar
    ui.horizontal(|ui| {
        ui.heading(RichText::new("🕒 BANLIST TIME-MACHINE").color(ACCENT_GOLD).size(18.0).strong());
        ui.separator();
        ui.label(RichText::new(format!("{} Historical Formats Available (2020 - 2026)", banlists.len())).color(TEXT_MUTED));

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Back to Menu").clicked() {
                *return_to_menu = true;
            }
        });
    });

    ui.separator();

    let list_w = 260.0;
    let content_w = screen_w - list_w - 20.0;

    let names = banlists.names();
    if names.is_empty() {
        ui.label("No banlists loaded.");
        return;
    }

    if state.selected_index >= names.len() {
        state.selected_index = 0;
    }
    let current_name = &names[state.selected_index];
    let current_banlist = banlists.get(current_name);

    ui.horizontal(|ui| {
        // --- LEFT COLUMN: BANLIST FORMAT SELECTOR ---
        ui.allocate_ui(Vec2::new(list_w, screen_h - 60.0), |ui| {
            ui.vertical(|ui| {
                ui.heading(RichText::new("Format History").size(14.0).color(TEXT_MUTED));
                ScrollArea::vertical().show(ui, |ui| {
                    for (i, name) in names.iter().enumerate() {
                        let is_selected = state.selected_index == i;
                        if ui.selectable_label(is_selected, name).clicked() {
                            state.selected_index = i;
                        }
                    }
                });
            });
        });

        ui.separator();

        // --- RIGHT COLUMN: FORBIDDEN / LIMITED CARDS GRID ---
        ui.allocate_ui(Vec2::new(content_w, screen_h - 60.0), |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading(RichText::new(current_name).size(16.0).color(ACCENT_CYAN).strong());
                    if let Some(bl) = current_banlist.as_ref() {
                        let eff = bl.effective_date.map(|d| d.to_string()).unwrap_or_else(|| "N/A".to_string());
                        ui.label(RichText::new(format!("Effective: {} | {} restricted entries", eff, bl.len())).color(TEXT_MUTED));
                    }
                });

                ui.add_space(4.0);

                // Filter Buttons & Search
                ui.horizontal(|ui| {
                    if ui.selectable_label(state.filter_limit.is_none(), "All Restricted").clicked() {
                        state.filter_limit = None;
                    }
                    if ui.selectable_label(state.filter_limit == Some(0), "🚫 Forbidden (0)").clicked() {
                        state.filter_limit = Some(0);
                    }
                    if ui.selectable_label(state.filter_limit == Some(1), "⚠️ Limited (1)").clicked() {
                        state.filter_limit = Some(1);
                    }
                    if ui.selectable_label(state.filter_limit == Some(2), "⚖ Semi-Limited (2)").clicked() {
                        state.filter_limit = Some(2);
                    }

                    ui.separator();
                    ui.label("🔍");
                    ui.text_edit_singleline(&mut state.search_query);
                });

                ui.separator();

                if let Some(bl) = current_banlist.as_ref() {
                    let mut cards_to_show = Vec::new();
                    for (code, limit) in bl.entries() {
                        if let Some(target_lim) = state.filter_limit {
                            if limit != target_lim {
                                continue;
                            }
                        }
                        if let Some(card) = db.get(code) {
                            if !state.search_query.is_empty()
                                && !card.name.to_lowercase().contains(&state.search_query.to_lowercase())
                            {
                                continue;
                            }
                            cards_to_show.push((card.clone(), limit));
                        }
                    }

                    // Sort: Forbidden (0) first, then Limited (1), then Semi (2)
                    cards_to_show.sort_by_key(|(c, lim)| (*lim, c.name.clone()));

                    ui.label(RichText::new(format!("Showing {} cards", cards_to_show.len())).color(TEXT_MUTED));

                    ScrollArea::vertical().show(ui, |ui| {
                        let card_w = 68.0;
                        let card_h = 96.0;
                        let cols = ((content_w - 20.0) / (card_w + 6.0)).max(1.0) as usize;

                        egui::Grid::new("banlist_cards_grid").spacing(Vec2::new(6.0, 6.0)).show(ui, |ui| {
                            for (i, (card, limit)) in cards_to_show.iter().enumerate() {
                                ui.vertical(|ui| {
                                    let resp = draw_card_item(ui, card, card_w, card_h, false);
                                    let badge = match limit {
                                        0 => ("FORBIDDEN", ACCENT_RED),
                                        1 => ("LIMITED", ACCENT_GOLD),
                                        2 => ("SEMI-LIMITED", ACCENT_BLUE),
                                        _ => ("LEGAL", ACCENT_GREEN),
                                    };
                                    ui.label(RichText::new(badge.0).size(9.0).color(badge.1).strong());
                                    resp.on_hover_text(format!("{}\nMax copies: {limit}", card.name));
                                });

                                if (i + 1) % cols == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                    });
                }
            });
        });
    });
}
