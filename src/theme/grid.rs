//! The one place the app builds columns (design §5.6, §1.4 L2/L3/L6).
//!
//! `egui::Ui::columns` divides whatever width it is handed by the column count, has no minimum
//! and no guard: at zero available width it asks for a negative column and trips a
//! `debug_assert` (that is the V4 launch panic). So `columns` is called here and nowhere else —
//! `scripts/check-no-raw-columns.sh` fails the build if it ever is — and callers reach the grid
//! through the two primitives below:
//!
//! * [`grid`] — N cards of the same kind, wrapping down to one column when they stop fitting.
//! * [`split`] — a fixed-width list pane beside a flexible one, stacking when both cannot fit.
//!
//! Both treat zero as a legal width and never ask for one they have not clamped (L2/L6).

use eframe::egui;

/// `want.min(available_width).max(0.0)` — the only sanctioned way to turn a desired width into a
/// usable one. A fixed width that skips this can exceed the parent and paint outside it.
pub fn clamp_w(ui: &egui::Ui, want: f32) -> f32 {
    want.min(ui.available_width()).max(0.0)
}

/// The height twin of [`clamp_w`]. Note that it also bottoms out at zero: a child given more
/// height than its parent has will be chopped at the parent's edge.
pub fn clamp_h(ui: &egui::Ui, want: f32) -> f32 {
    want.min(ui.available_height()).max(0.0)
}

/// The widest count in `1..=n` that fits `wanted` cards of at least `min_col_w` separated by
/// `gap` into `usable` pixels. Never returns zero: one column always fits, even at width zero.
fn fitting_columns(usable: f32, wanted: usize, min_col_w: f32, gap: f32) -> usize {
    (1..=wanted.max(1))
        .rev()
        .find(|columns| {
            let needed = min_col_w * *columns as f32 + gap * (*columns as f32 - 1.0);
            usable >= needed
        })
        .unwrap_or(1)
}

/// Lays `items` cards out in columns — never more than `max_columns`, never more than fit at
/// `min_col_w` each — wrapping into rows of `row_gap` spacing.
///
/// `add` is called once per card with its own column `Ui` and the card's index, so cards of
/// different kinds can share one grid and a short column slice can never drop one. Content that
/// does not fit gets fewer columns, not a truncated card: 3 → 2 → 1.
pub fn grid(
    ui: &mut egui::Ui,
    items: usize,
    max_columns: usize,
    min_col_w: f32,
    row_gap: f32,
    mut add: impl FnMut(&mut egui::Ui, usize),
) {
    if items == 0 {
        return;
    }
    let gap = ui.spacing().item_spacing.x;
    let columns = fitting_columns(
        clamp_w(ui, ui.available_width()),
        max_columns.min(items),
        min_col_w,
        gap,
    );

    let mut first = 0;
    let mut row = 0;
    while first < items {
        if row > 0 {
            ui.add_space(row_gap);
        }
        let end = (first + columns).min(items);
        ui.columns(end - first, |column_uis| {
            for (column_ui, index) in column_uis.iter_mut().zip(first..end) {
                add(column_ui, index);
            }
        });
        first = end;
        row += 1;
    }
}

/// A fixed-width pane beside a flexible one: a module list next to its detail view. When the two
/// cannot both be given their minimum, the panes stack — top pane first — instead of squeezing
/// either one to nothing.
///
/// `add` is called once per pane with the pane's `Ui` and its index (`0` = left/top, `1` =
/// right/bottom), so one closure can reach `&mut self` for both panes.
pub fn split(
    ui: &mut egui::Ui,
    left_w: f32,
    min_right_w: f32,
    mut add: impl FnMut(&mut egui::Ui, usize),
) {
    let gap = ui.spacing().item_spacing.x;
    let usable = clamp_w(ui, ui.available_width());
    let left = clamp_w(ui, left_w);

    if left <= 0.0 || left + gap + min_right_w > usable {
        // One above the other. The bodies stay in an explicit `vertical`, so a `Frame` inside
        // them cannot inherit a left-to-right layout (L1 — that inheritance is the V4 panic).
        ui.vertical(|ui| add(ui, 0));
        ui.add_space(gap);
        ui.vertical(|ui| add(ui, 1));
        return;
    }

    let height = clamp_h(ui, ui.available_height());
    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(left, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(left);
                ui.vertical(|ui| add(ui, 0));
            },
        );
        ui.add_space(gap);
        ui.vertical(|ui| add(ui, 1));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The panic's arithmetic: at 763 px (1024×640) four 150 px dials fit; squeezes drop
    /// columns instead of shrinking every card below its minimum.
    #[test]
    fn fitting_columns_counts_whole_cards() {
        assert_eq!(fitting_columns(763.0, 4, 150.0, 8.0), 4);
        assert_eq!(fitting_columns(624.0, 4, 150.0, 8.0), 4);
        assert_eq!(fitting_columns(623.0, 4, 150.0, 8.0), 3);
        assert_eq!(fitting_columns(308.0, 4, 150.0, 8.0), 2);
        assert_eq!(fitting_columns(307.0, 4, 150.0, 8.0), 1);
        assert_eq!(fitting_columns(0.0, 4, 150.0, 8.0), 1);
        assert_eq!(fitting_columns(-6.0, 4, 150.0, 8.0), 1);
        assert_eq!(fitting_columns(1920.0, 1, 150.0, 8.0), 1);
    }

    /// AC-02/AC-05's picture, without a window: no width may panic, including the zero that the
    /// old `Ui::columns` divided by.
    #[test]
    fn grid_and_split_survive_every_width_including_zero() {
        for width in [1920.0_f32, 1024.0, 763.0, 320.0, 0.0] {
            let ctx = egui::Context::default();
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 640.0),
                )),
                ..Default::default()
            };
            let output = ctx.run_ui(input, |ui| {
                ui.set_max_width(width);
                let mut seen = 0;
                grid(ui, 4, 4, 150.0, 6.0, |column, index| {
                    assert_eq!(index, seen, "cards arrive in order, once each: {width}");
                    seen += 1;
                    column.label("card");
                });
                assert_eq!(seen, 4);
                let mut panes = Vec::new();
                split(ui, 365.0, 320.0, |pane, index| {
                    panes.push(index);
                    pane.label("pane");
                });
                assert_eq!(panes, [0, 1], "both panes arrive once, in order: {width}");
            });
            output.drop_without_applying_deltas();
        }
    }

    /// The grid wraps: four cards in two columns are painted as two rows, not one squeezed row.
    #[test]
    fn grid_wraps_into_rows() {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(320.0, 640.0),
            )),
            ..Default::default()
        };
        let output = ctx.run_ui(input, |ui| {
            let mut tops = Vec::new();
            let mut next_top = 0.0_f32;
            grid(ui, 4, 4, 150.0, 6.0, |column, _index| {
                let (rect, _) =
                    column.allocate_exact_size(egui::vec2(40.0, 20.0), egui::Sense::hover());
                tops.push(rect.top());
                next_top = rect.top();
            });
            assert!(next_top > 0.0);
            let first_row_top = tops[0];
            assert_eq!(
                tops.iter().filter(|top| **top == first_row_top).count(),
                2,
                "two 150 px cards fit in 320 px: {tops:?}"
            );
        });
        output.drop_without_applying_deltas();
    }
}
