//! Card rendering widget for egui.

use bevy_egui::egui::{
    pos2, vec2, Align2, Color32, FontFamily, FontId, Rect, Response, Sense, Stroke, Ui,
};
use ygopro_data::Card;

use crate::theme::*;

/// Draws a stylized Yu-Gi-Oh! card widget.
pub fn draw_card_item(
    ui: &mut Ui,
    card: &Card,
    width: f32,
    height: f32,
    is_selected: bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());

    if !ui.is_rect_visible(rect) {
        return response;
    }

    let painter = ui.painter();
    let frame_color = card_type_color(card.type_flags);
    let is_hovered = response.hovered();

    // 1. Card Frame background
    let bg_color = if card.is_xyz() {
        Color32::from_rgb(20, 20, 24)
    } else {
        frame_color
    };
    painter.rect_filled(rect, 4.0, bg_color);

    // 2. Outer border / Hover glow
    let border_stroke = if is_selected {
        selection_stroke()
    } else if is_hovered {
        hover_stroke()
    } else {
        Stroke::new(1.0, Color32::from_black_alpha(180))
    };
    painter.rect_stroke(rect, 4.0, border_stroke, bevy_egui::egui::StrokeKind::Outside);

    // If small card (e.g. deck builder grid item), draw compact layout
    if width < 120.0 {
        // Top name strip
        let title_rect = Rect::from_min_size(
            pos2(rect.min.x + 3.0, rect.min.y + 3.0),
            vec2(rect.width() - 6.0, 16.0),
        );
        painter.rect_filled(title_rect, 2.0, Color32::from_black_alpha(160));

        let text_color = if card.is_xyz() {
            Color32::WHITE
        } else {
            TEXT_PRIMARY
        };
        let mut display_name = card.name.clone();
        if display_name.len() > 14 {
            display_name.truncate(13);
            display_name.push('…');
        }
        painter.text(
            pos2(title_rect.min.x + 3.0, title_rect.center().y),
            Align2::LEFT_CENTER,
            display_name,
            FontId::new(9.0, FontFamily::Proportional),
            text_color,
        );

        // Art box area
        let art_rect = Rect::from_min_size(
            pos2(rect.min.x + 5.0, rect.min.y + 22.0),
            vec2(rect.width() - 10.0, height - 42.0),
        );
        painter.rect_filled(art_rect, 2.0, Color32::from_rgb(18, 22, 30));
        painter.rect_stroke(art_rect, 2.0, Stroke::new(1.0, Color32::from_black_alpha(120)), bevy_egui::egui::StrokeKind::Inside);

        // Card Type & Attribute Badge inside art
        painter.text(
            pos2(art_rect.center().x, art_rect.center().y - 6.0),
            Align2::CENTER_CENTER,
            card_type_label(card.type_flags),
            FontId::new(10.0, FontFamily::Monospace),
            Color32::from_white_alpha(100),
        );
        if card.is_monster() && card.attribute != 0 {
            painter.text(
                pos2(art_rect.center().x, art_rect.center().y + 8.0),
                Align2::CENTER_CENTER,
                attribute_label(card.attribute),
                FontId::new(9.0, FontFamily::Proportional),
                attribute_color(card.attribute),
            );
        }

        // Bottom ATK/DEF bar for monsters
        let bottom_rect = Rect::from_min_size(
            pos2(rect.min.x + 3.0, rect.max.y - 18.0),
            vec2(rect.width() - 6.0, 15.0),
        );
        painter.rect_filled(bottom_rect, 2.0, Color32::from_black_alpha(180));

        if card.is_monster() {
            let stats_text = if card.is_link() {
                format!("{}/L{}", card.attack.max(0), card.level)
            } else {
                format!("{}/{}", card.attack.max(0), card.defense.max(0))
            };
            painter.text(
                pos2(bottom_rect.center().x, bottom_rect.center().y),
                Align2::CENTER_CENTER,
                stats_text,
                FontId::new(8.5, FontFamily::Monospace),
                ACCENT_GOLD,
            );
        } else {
            painter.text(
                pos2(bottom_rect.center().x, bottom_rect.center().y),
                Align2::CENTER_CENTER,
                card_type_label(card.type_flags),
                FontId::new(8.5, FontFamily::Proportional),
                Color32::LIGHT_GRAY,
            );
        }
    } else {
        // Detailed Card Preview (Inspector Pane)
        // Title Bar
        let title_rect = Rect::from_min_size(
            pos2(rect.min.x + 6.0, rect.min.y + 6.0),
            vec2(rect.width() - 12.0, 24.0),
        );
        painter.rect_filled(title_rect, 3.0, Color32::from_black_alpha(180));
        painter.text(
            pos2(title_rect.min.x + 6.0, title_rect.center().y),
            Align2::LEFT_CENTER,
            &card.name,
            FontId::new(12.0, FontFamily::Proportional),
            TEXT_PRIMARY,
        );

        if card.attribute != 0 {
            painter.text(
                pos2(title_rect.max.x - 6.0, title_rect.center().y),
                Align2::RIGHT_CENTER,
                attribute_label(card.attribute),
                FontId::new(11.0, FontFamily::Proportional),
                attribute_color(card.attribute),
            );
        }

        // Stars / Level bar
        if card.is_monster() {
            let star_str = if card.is_xyz() {
                format!("★ Rank {}", card.level)
            } else if card.is_link() {
                format!("◆ Link {}", card.level)
            } else {
                format!("★ Level {}", card.level)
            };
            painter.text(
                pos2(rect.min.x + 10.0, rect.min.y + 36.0),
                Align2::LEFT_CENTER,
                star_str,
                FontId::new(11.0, FontFamily::Proportional),
                ACCENT_GOLD,
            );
        }

        // Art frame
        let art_height = (height * 0.45).min(200.0);
        let art_rect = Rect::from_min_size(
            pos2(rect.min.x + 10.0, rect.min.y + 48.0),
            vec2(rect.width() - 20.0, art_height),
        );
        painter.rect_filled(art_rect, 3.0, Color32::from_rgb(16, 20, 28));
        painter.rect_stroke(art_rect, 3.0, Stroke::new(1.0, Color32::from_black_alpha(140)), bevy_egui::egui::StrokeKind::Inside);

        painter.text(
            art_rect.center(),
            Align2::CENTER_CENTER,
            format!("[ {} ]\nPasscode: {}", card_type_label(card.type_flags), card.code),
            FontId::new(12.0, FontFamily::Monospace),
            Color32::from_white_alpha(120),
        );

        // Stats bar at bottom
        let bottom_rect = Rect::from_min_size(
            pos2(rect.min.x + 8.0, rect.max.y - 28.0),
            vec2(rect.width() - 16.0, 22.0),
        );
        painter.rect_filled(bottom_rect, 3.0, Color32::from_black_alpha(190));

        if card.is_monster() {
            let stats_text = if card.is_link() {
                format!("ATK / {}   LINK - {}", card.attack.max(0), card.level)
            } else {
                format!("ATK / {}   DEF / {}", card.attack.max(0), card.defense.max(0))
            };
            painter.text(
                pos2(bottom_rect.center().x, bottom_rect.center().y),
                Align2::CENTER_CENTER,
                stats_text,
                FontId::new(11.0, FontFamily::Monospace),
                ACCENT_GOLD,
            );
        } else {
            painter.text(
                pos2(bottom_rect.center().x, bottom_rect.center().y),
                Align2::CENTER_CENTER,
                format!("[ {} CARD ]", card_type_label(card.type_flags)),
                FontId::new(11.0, FontFamily::Proportional),
                Color32::LIGHT_GRAY,
            );
        }
    }

    response
}
