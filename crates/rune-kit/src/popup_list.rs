//! Anchored popup listbox (`PopupList` / `PopupItem`) for popovers filtered by an outside
//! field, such as a chat composer's `@`-mention and `/`-command popovers, and the shared list
//! inside [`crate::Combobox`].

use std::rc::Rc;

use rune::{Align, Color, El, GrowthSpec, ScrollSpec, Side, Transition, Tx, col, el, row, text};

use crate::overlay::{Placement, anchor_at, raised, scrim};
use crate::{ComboboxOption, Kit};

/// One row in a [`PopupList`]: `PopupItem::new("review", "/review").description("Review changes")`.
#[derive(Clone)]
pub struct PopupItem {
    id: String,
    label: String,
    description: Option<String>,
    leading: Option<El>,
    trailing: Option<El>,
    disabled: bool,
}

impl PopupItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            leading: None,
            trailing: None,
            disabled: false,
        }
    }
    /// Secondary muted text shown after the label on the same row.
    pub fn description(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.description = (!text.is_empty()).then_some(text);
        self
    }
    /// Leading element before the label (an icon, avatar, or swatch).
    pub fn leading(mut self, el: El) -> Self {
        self.leading = Some(el);
        self
    }
    /// Trailing element at the right edge of the row (a keybind hint, badge, or check icon).
    pub fn trailing(mut self, el: El) -> Self {
        self.trailing = Some(el);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl From<&PopupItem> for PopupItem {
    fn from(item: &PopupItem) -> Self {
        item.clone()
    }
}

impl From<&ComboboxOption> for PopupItem {
    fn from(o: &ComboboxOption) -> Self {
        Self::new(&o.id, &o.label).description(&o.detail).disabled(o.disabled)
    }
}

impl From<ComboboxOption> for PopupItem {
    fn from(o: ComboboxOption) -> Self {
        Self::new(o.id, o.label).description(o.detail).disabled(o.disabled)
    }
}

type ActiveFn<S> = Rc<dyn Fn(&mut S, usize)>;
type ChooseFn<S> = Rc<dyn Fn(&mut S, &str, &mut Tx)>;
type CloseFn<S> = Rc<dyn Fn(&mut S)>;

/// An anchored popup listbox driven by an outside field:
/// `PopupList::new("mentions", "composer").open(s.open).items(items).active(s.active).build(&k, &mut layer)`.
///
/// The popup takes no keyboard focus, so focus and typing stay in the app's field while it is
/// open. Moving `active` in app state scrolls the active option into view.
pub struct PopupList<S> {
    key: String,
    anchor: String,
    open: bool,
    side: Side,
    align: Align,
    gutter: f32,
    width: f32,
    max_height: f32,
    padding: f32,
    row_radius: f32,
    items: Vec<PopupItem>,
    animate_filtering: bool,
    active: Option<usize>,
    empty: String,
    label: Option<String>,
    header: Option<String>,
    caret: bool,
    on_active: Option<ActiveFn<S>>,
    on_choose: Option<ChooseFn<S>>,
    on_close: Option<CloseFn<S>>,
}

impl<S: 'static> PopupList<S> {
    pub fn new(key: impl Into<String>, anchor: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            anchor: anchor.into(),
            open: true,
            side: Side::Top,
            align: Align::Start,
            gutter: 4.0,
            width: 260.0,
            max_height: 240.0,
            padding: 4.0,
            row_radius: 4.0,
            items: Vec::new(),
            animate_filtering: false,
            active: None,
            empty: "No results".into(),
            label: None,
            header: None,
            caret: false,
            on_active: None,
            on_choose: None,
            on_close: None,
        }
    }
    pub fn anchor(mut self, anchor: impl Into<String>) -> Self {
        self.anchor = anchor.into();
        self
    }
    /// Anchor to `anchor` with explicit side, alignment and offset, as [`El::anchored`].
    pub fn anchored(mut self, anchor: impl Into<String>, side: Side, align: Align, gap: f32) -> Self {
        self.anchor = anchor.into();
        self.side = side;
        self.align = align;
        self.gutter = gap;
        self
    }
    /// Hang the list from the caret of the focused text field keyed `field` (or inside an
    /// element keyed `field`), on the side and alignment set so far (above the caret, at its
    /// start, by default), as an `@`-mention picker follows the word being typed. It follows
    /// the caret as the person types and flips when it doesn't fit; while the field has no
    /// focus it sits beside the field's box. See [`rune::El::anchored_to_caret`].
    ///
    /// ```ignore
    /// PopupList::new("mentions", "composer").at_caret("composer").placement(Placement::BottomStart)
    /// ```
    pub fn at_caret(mut self, field: impl Into<String>) -> Self {
        self.anchor = field.into();
        self.caret = true;
        self
    }
    /// [`Self::at_caret`] with explicit side, alignment and offset, as [`Self::anchored`].
    pub fn anchored_to_caret(self, field: impl Into<String>, side: Side, align: Align, gap: f32) -> Self {
        let mut list = self.anchored(field, side, align, gap);
        list.caret = true;
        list
    }
    pub fn placement(mut self, placement: Placement) -> Self {
        let (side, align) = placement.split();
        self.side = side;
        self.align = align;
        self
    }
    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
    pub fn offset(mut self, gap: f32) -> Self {
        self.gutter = gap;
        self
    }
    pub fn gutter(mut self, gap: f32) -> Self {
        self.gutter = gap;
        self
    }
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }
    /// Space between the popover's edge and its rows, in points (default 4).
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
    /// Corner radius of each row's highlight, in points (default 4).
    pub fn row_radius(mut self, radius: f32) -> Self {
        self.row_radius = radius;
        self
    }
    /// The rows. A change lands at once: the popover takes its new height and rows appear and
    /// go in the same frame, even in a commit that animates the field the list follows (one
    /// with `animate_edit_layout`). [`Self::animate_filtering`] animates it instead.
    pub fn items(mut self, items: impl IntoIterator<Item = impl Into<PopupItem>>) -> Self {
        self.items = items.into_iter().map(Into::into).collect();
        self
    }
    /// Animate a change to the rows: the popover springs to its new height, and the rows it
    /// drops fade out where they stood (a 0.24 s spring) as the new ones fade in (0.3 s).
    /// Off by default, so a list that follows each keystroke swaps at once. Opening, closing
    /// and the hover highlight animate either way.
    pub fn animate_filtering(mut self, animate: bool) -> Self {
        self.animate_filtering = animate;
        self
    }
    pub fn item(mut self, item: impl Into<PopupItem>) -> Self {
        self.items.push(item.into());
        self
    }
    pub fn active(mut self, active: Option<usize>) -> Self {
        self.active = active;
        self
    }
    pub fn empty_message(mut self, text: impl Into<String>) -> Self {
        self.empty = text.into();
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// A heading line above the rows ("People", "At most 10 mentions"): muted, not an option,
    /// so it is never active, chosen or counted in the rows' positions. It stays while the
    /// list is empty, above the empty message.
    pub fn header(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.header = (!text.is_empty()).then_some(text);
        self
    }
    pub fn on_active(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        self.on_active = Some(Rc::new(f));
        self
    }
    pub fn on_choose(mut self, f: impl Fn(&mut S, &str, &mut Tx) + 'static) -> Self {
        self.on_choose = Some(Rc::new(f));
        self
    }
    pub fn on_close(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    pub(crate) fn build_list(self, k: &Kit) -> El {
        let active = self.active.filter(|&i| self.items.get(i).is_some_and(|item| !item.disabled));
        let mut rows = col().key("rows").w_full().growth(GrowthSpec::default()).virtualize(28.0).reveal_selected(true);
        for (i, option) in self.items.iter().enumerate() {
            let id = option.id.clone();
            let is_active = active == Some(i);
            let mut item = row()
                .key(format!("{}-option-{}", self.key, id))
                .role("option")
                .label(&option.label)
                .description(option.description.clone().unwrap_or_default())
                .position_in_set(i + 1, self.items.len())
                .focus_selected(is_active)
                .w_full()
                .h(28.0)
                .px(8.0)
                .gap(8.0)
                .center_y()
                .radius(self.row_radius)
                .transition(Transition::opacity())
                .fill(if is_active { k.c.hover } else { Color::TRANSPARENT });
            if let Some(leading) = &option.leading {
                item = item.child(leading.clone());
            }
            if let Some(desc) = option.description.as_deref().filter(|d| !d.is_empty()) {
                item = item
                    .child(text(&option.label).font(k.ty.ui_compact).color(k.c.text_base).shrink().truncate().a11y_hidden())
                    .child(text(desc).key("description").font(k.ty.ui_compact).color(k.c.text_weak).grow(1.0).shrink().truncate().a11y_hidden());
            } else {
                item = item.child(text(&option.label).font(k.ty.ui_compact).color(k.c.text_base).grow(1.0).shrink().truncate().a11y_hidden());
            }
            if let Some(trailing) = &option.trailing {
                item = item.child(trailing.clone());
            }
            if option.disabled {
                item = item.layer_opacity(0.5).disabled(true).on_press(|_: &mut S| {});
            } else {
                if let Some(on_active) = &self.on_active {
                    let on_active = on_active.clone();
                    item = item.on_hover(move |s: &mut S| on_active(s, i));
                }
                if let Some(on_choose) = &self.on_choose {
                    let on_choose = on_choose.clone();
                    item = item.on_press_tx(move |s: &mut S, tx| on_choose(s, &id, tx));
                } else {
                    item = item.on_press(|_: &mut S| {});
                }
            }
            rows = rows.child(item);
        }
        // The viewport and the empty state both stay mounted, so the popover's height springs
        // between them as the search edits it instead of swapping one box for the other.
        let empty = self.items.is_empty();
        let mut viewport = col()
            .key("viewport")
            .role("listbox")
            .w_full()
            .h((self.items.len() as f32 * 28.0).min(self.max_height))
            .shrink()
            // A window too short for the list still shows one row to scroll.
            .min_h(if self.items.is_empty() { 0.0 } else { 28.0 })
            .scroll(ScrollSpec { stick_bottom: false })
            .scroll_edge_fade(crate::overlay::LIST_SCROLL_FADE)
            .scroll_margin(crate::overlay::LIST_SCROLL_FADE)
            .clip_soft(0.0)
            .child(rows);
        if let Some(label) = self.label.as_deref() {
            viewport = viewport.label(label);
        }
        let header = self.header.map(|h| {
            row()
                .key("header")
                .w_full()
                .h(28.0)
                .px(8.0)
                .center_y()
                .child(text(h).font(k.ty.ui_compact).color(k.c.text_weak).grow(1.0).shrink().truncate())
        });
        col()
            .key("list")
            .preserve_focus_on_press(true)
            .w_full()
            .shrink()
            .min_h(0.0)
            .children(header)
            .child(crate::scroll_bar(k, &format!("{}-scroll-area", self.key), "viewport", viewport).w_full().shrink().min_h(0.0))
            .child(
                el().key("empty")
                    .h(if empty { 28.0 } else { 0.0 })
                    .px(8.0)
                    .center_y()
                    .clip_soft(0.0)
                    .opacity(if empty { 1.0 } else { 0.0 })
                    .child(text(self.empty).font(k.ty.ui_compact).color(k.c.text_weak)),
            )
    }

    pub fn build(self, k: &Kit, layer: &mut Vec<El>) {
        if !self.open {
            return;
        }
        if let Some(close) = &self.on_close {
            let close = close.clone();
            layer.push(scrim(format!("{}-scrim", self.key), move |s: &mut S| close(s)));
        }
        let key = format!("{}-content", self.key);
        let anchor = self.anchor.clone();
        let side = self.side;
        let align = self.align;
        let gutter = self.gutter;
        let width = self.width;
        let padding = self.padding;
        let instant = !self.animate_filtering;
        let caret = self.caret;
        let list = self.build_list(k);
        let mut content = col()
            .key(key)
            .preserve_focus_on_press(true)
            .w(width)
            .padding(padding)
            .on_press(|_: &mut S| {})
            .child(list);
        if instant {
            content = content.instant_changes();
        }
        let surface = anchor_at(k, raised(k, content, k.radius.md), &anchor, side, align, gutter);
        layer.push(if caret { surface.anchored_at_caret() } else { surface });
    }
}
