//! Stylized Master Duel card widget with level stars, attribute orbs, and stats plates.

use bevy_egui::egui::{
    pos2, vec2, Align2, Color32, FontFamily, FontId, Rect, Response, Sense, Stroke, Ui,
};
use ygopro_data::Card;

use crate::theme::*;

/// Draws a stylized Master Duel card widget.
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

    // 1. Base card frame background with rounded corners
    let bg_color = if card.is_xyz() {
        Color32::from_rgb(18, 18, 24)
    } else {
        frame_color
    };
    painter.rect_filled(rect, 4.0, bg_color);

    // Subtle dark gradient overlay
    let inner_frame = rect.shrink(2.0);
    painter.rect_stroke(
        inner_frame,
        3.0,
        Stroke::new(1.0, Color32::from_white_alpha(35)),
        bevy_egui::egui::StrokeKind::Inside,
    );

    // 2. Outer border / Master Duel glow
    let border_stroke = if is_selected {
        selection_stroke()
    } else if is_hovered {
        hover_stroke()
    } else {
        card_border_stroke()
    };
    painter.rect_stroke(rect, 4.0, border_stroke, bevy_egui::egui::StrokeKind::Outside);

    // --- COMPACT CARD LAYOUT (Deck Grid & Catalog) ---
    if width < 140.0 {
        // Name Header Banner
        let title_rect = Rect::from_min_size(
            pos2(rect.min.x + 3.0, rect.min.y + 3.0),
            vec2(rect.width() - 6.0, 16.0),
        );
        painter.rect_filled(title_rect, 2.0, Color32::from_black_alpha(200));

        let text_color = if card.is_xyz() {
            Color32::WHITE
        } else {
            TEXT_PRIMARY
        };
        let mut display_name = card.name.clone();
        let max_len = if width < 80.0 { 12 } else { 18 };
        if display_name.len() > max_len {
            display_name.truncate(max_len.saturating_sub(1));
            display_name.push('…');
        }
        painter.text(
            pos2(title_rect.min.x + 3.0, title_rect.center().y),
            Align2::LEFT_CENTER,
            display_name,
            FontId::new(9.0, FontFamily::Proportional),
            text_color,
        );

        // Attribute icon in top right
        if card.attribute != 0 {
            let attr_short = match card.attribute {
                ygopro_data::constants::ATTRIBUTE_DARK => "●",
                ygopro_data::constants::ATTRIBUTE_LIGHT => "●",
                ygopro_data::constants::ATTRIBUTE_WATER => "●",
                ygopro_data::constants::ATTRIBUTE_FIRE => "●",
                ygopro_data::constants::ATTRIBUTE_EARTH => "●",
                ygopro_data::constants::ATTRIBUTE_WIND => "●",
                ygopro_data::constants::ATTRIBUTE_DIVINE => "★",
                _ => "",
            };
            painter.text(
                pos2(title_rect.max.x - 3.0, title_rect.center().y),
                Align2::RIGHT_CENTER,
                attr_short,
                FontId::new(10.0, FontFamily::Proportional),
                attribute_color(card.attribute),
            );
        }

        // Center Art Vignette
        let art_top = rect.min.y + 21.0;
        let art_h = (height - 39.0).max(10.0);
        let art_rect = Rect::from_min_size(
            pos2(rect.min.x + 4.0, art_top),
            vec2(rect.width() - 8.0, art_h),
        );
        painter.rect_filled(art_rect, 2.0, Color32::from_rgb(14, 18, 26));
        painter.rect_stroke(
            art_rect,
            2.0,
            Stroke::new(1.0, Color32::from_black_alpha(150)),
            bevy_egui::egui::StrokeKind::Inside,
        );

        // Card Category Badge watermark inside art
        let label = if (card.type_flags & ygopro_data::constants::TYPE_SPELL) != 0 {
            spell_trap_type_label(card.type_flags)
        } else if (card.type_flags & ygopro_data::constants::TYPE_TRAP) != 0 {
            spell_trap_type_label(card.type_flags)
        } else {
            card_type_label(card.type_flags)
        };
        painter.text(
            pos2(art_rect.center().x, art_rect.center().y - 4.0),
            Align2::CENTER_CENTER,
            label,
            FontId::new(9.0, FontFamily::Monospace),
            Color32::from_white_alpha(110),
        );

        // Stars / Level row
        if card.is_monster() && card.level > 0 {
            let star_char = if card.is_xyz() { "✦" } else { "★" };
            let stars_count = card.level.min(12) as usize;
            let stars_str = if stars_count > 6 {
                format!("{stars_count}{star_char}")
            } else {
                star_char.repeat(stars_count)
            };
            painter.text(
                pos2(art_rect.center().x, art_rect.center().y + 8.0),
                Align2::CENTER_CENTER,
                stars_str,
                FontId::new(8.0, FontFamily::Proportional),
                if card.is_xyz() { ACCENT_RED } else { ACCENT_GOLD },
            );
        }

        // Bottom Stats Bar
        let bottom_rect = Rect::from_min_size(
            pos2(rect.min.x + 3.0, rect.max.y - 17.0),
            vec2(rect.width() - 6.0, 14.0),
        );
        painter.rect_filled(bottom_rect, 2.0, Color32::from_black_alpha(210));

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
                FontId::new(8.0, FontFamily::Proportional),
                Color32::LIGHT_GRAY,
            );
        }
    } else {
        // --- DETAILED CARD INSPECTOR (Master Duel Style) ---
        // Header Name Strip with Gold Accent
        let title_rect = Rect::from_min_size(
            pos2(rect.min.x + 6.0, rect.min.y + 6.0),
            vec2(rect.width() - 12.0, 26.0),
        );
        painter.rect_filled(title_rect, 3.0, Color32::from_black_alpha(210));
        painter.text(
            pos2(title_rect.min.x + 8.0, title_rect.center().y),
            Align2::LEFT_CENTER,
            &card.name,
            FontId::new(13.0, FontFamily::Proportional),
            TEXT_PRIMARY,
        );

        if card.attribute != 0 {
            painter.text(
                pos2(title_rect.max.x - 8.0, title_rect.center().y),
                Align2::RIGHT_CENTER,
                attribute_icon(card.attribute),
                FontId::new(11.0, FontFamily::Proportional),
                attribute_color(card.attribute),
            );
        }

        // Level / Rank / Link Rating Banner
        if card.is_monster() {
            let star_str = if card.is_xyz() {
                format!("✦ Rank {}", card.level)
            } else if card.is_link() {
                format!("◆ Link Rating {}", card.level)
            } else {
                format!("★ Level {}", card.level)
            };
            let color = if card.is_xyz() { ACCENT_RED } else { ACCENT_GOLD };
            painter.text(
                pos2(rect.min.x + 12.0, rect.min.y + 38.0),
                Align2::LEFT_CENTER,
                star_str,
                FontId::new(11.0, FontFamily::Proportional),
                color,
            );
        }

        // Large Artwork Viewport with Metallic Bevel
        let art_top = if card.is_monster() { rect.min.y + 50.0 } else { rect.min.y + 38.0 };
        let art_height = (height * 0.46).min(180.0);
        let art_rect = Rect::from_min_size(
            pos2(rect.min.x + 8.0, art_top),
            vec2(rect.width() - 16.0, art_height),
        );
        painter.rect_filled(art_rect, 4.0, Color32::from_rgb(12, 15, 22));
        painter.rect_stroke(
            art_rect,
            4.0,
            Stroke::new(1.5, Color32::from_black_alpha(180)),
            bevy_egui::egui::StrokeKind::Inside,
        );

        // Center card watermark
        painter.text(
            pos2(art_rect.center().x, art_rect.center().y - 10.0),
            Align2::CENTER_CENTER,
            format!("[ {} ]", card_type_label(card.type_flags)),
            FontId::new(14.0, FontFamily::Monospace),
            Color32::from_white_alpha(90),
        );
        painter.text(
            pos2(art_rect.center().x, art_rect.center().y + 12.0),
            Align2::CENTER_CENTER,
            format!("Passcode: {:08}", card.code),
            FontId::new(11.0, FontFamily::Monospace),
            Color32::from_white_alpha(120),
        );

        // Type Ribbon (e.g. [Aqua / Effect])
        let type_rect = Rect::from_min_size(
            pos2(rect.min.x + 8.0, art_rect.max.y + 6.0),
            vec2(rect.width() - 16.0, 20.0),
        );
        painter.rect_filled(type_rect, 2.0, Color32::from_black_alpha(160));

        let type_desc = if card.is_monster() {
            format!("[ {} / {} ]", race_label(card.race), card_type_label(card.type_flags))
        } else {
            format!("[ {} ]", spell_trap_type_label(card.type_flags))
        };
        painter.text(
            pos2(type_rect.center().x, type_rect.center().y),
            Align2::CENTER_CENTER,
            type_desc,
            FontId::new(10.5, FontFamily::Proportional),
            TEXT_GOLD,
        );

        // Bottom ATK/DEF Plaque
        let bottom_rect = Rect::from_min_size(
            pos2(rect.min.x + 8.0, rect.max.y - 28.0),
            vec2(rect.width() - 16.0, 22.0),
        );
        painter.rect_filled(bottom_rect, 3.0, Color32::from_black_alpha(220));

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
                FontId::new(11.5, FontFamily::Monospace),
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
