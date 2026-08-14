//! Animation demo — UIKit-Dynamics-style physics with drag-to-fling.
//!
//! Ten cards fall under gravity and bounce inside a boundary box. Grab any
//! card and drag it; release to fling (throw) it with the measured drag
//! velocity. Cards collide with each other and with the stage bounds.
//!
//! ```bash
//! cargo run --example animation
//! ```

use uikit::animation::prelude::*;
use uikit::prelude::{App, Button, HStack, Text, VStack, Widget};
use uikit::widget::{WidgetId, next_widget_id};
use uikit::style::Color;
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;

/// Physics + widget state lives on the main thread. The animator ticks and the
/// animated widgets are all driven from the GTK main loop, so thread-locals
/// sidestep the `Widget: Send + Sync` requirement without unsafe code.
thread_local! {
    static ANIMATOR: RefCell<Animator> = RefCell::new(Animator::new());
    static ANIMATED: RefCell<Vec<AnimatedWidget>> = RefCell::new(Vec::new());
    static DRAG: RefCell<HashMap<usize, DragState>> = RefCell::new(HashMap::new());
}

/// Current drag state of a grabbed item.
struct DragState {
    /// Item center position at the moment the grab started.
    start: Vec2,
    /// Last reported drag offset (for velocity estimation).
    last_offset: Vec2,
    /// Time of the last drag update.
    last_time: Instant,
    /// Velocity estimate from the most recent update.
    velocity: Vec2,
}

const STAGE_W: f32 = 600.0;
const STAGE_H: f32 = 340.0;
const MAX_FLING: f32 = 1800.0;

/// A physical stage: an overlay that owns physics-driven, draggable buttons.
struct PhysicsStage;

impl Widget for PhysicsStage {
    fn id(&self) -> WidgetId {
        next_widget_id()
    }

    fn to_gtk(&self) -> gtk::Widget {
        let panel = gtk::Overlay::new();
        panel.set_hexpand(true);
        panel.set_vexpand(true);
        panel.set_size_request(STAGE_W as i32, STAGE_H as i32);

        let size = Size::new(44.0, 44.0);

        ANIMATOR.with(|anim| {
            let mut animator = anim.borrow_mut();
            for i in 0..10 {
                let x = 60.0 + (i % 5) as f32 * 112.0 + (i % 3) as f32 * 8.0;
                let y = 50.0 + (i / 5) as f32 * 150.0 + (i % 2) as f32 * 24.0;
                let position = v(x, y);

                let (animated, button) =
                    AnimatedWidget::new_button(&mut animator, &format!("{}", i), &panel, position, size);
                if let Some(item) = animated.item_mut(&mut animator) {
                    item.elasticity = 0.85;
                }
                let index = animated.item_index;

                // Drag + fling: grab with the primary button, throw on release.
                let gesture = gtk::GestureDrag::new();
                gesture.set_button(gtk::gdk::BUTTON_PRIMARY);
                gesture.connect_drag_begin(move |_g, _x, _y| {
                    drag_begin(index);
                });
                gesture.connect_drag_update(move |_g, x, y| {
                    drag_update(index, x as f32, y as f32);
                });
                gesture.connect_drag_end(move |_g, x, y| {
                    drag_end(index, x as f32, y as f32);
                });
                button.add_controller(gesture);

                ANIMATED.with(|list| list.borrow_mut().push(animated));
            }

            animator.set_bounds(Some(Rect::new(0.0, 0.0, STAGE_W, STAGE_H)));
            animator.add_behavior(CollisionBehavior::new(Some(Rect::new(0.0, 0.0, STAGE_W, STAGE_H))));
            animator.add_behavior(GravityBehavior::new(v(0.0, 620.0)));
        });

        panel.upcast()
    }
}

/// Start grabbing item `index`.
fn drag_begin(index: usize) {
    DRAG.with(|d| {
        let mut d = d.borrow_mut();
        ANIMATOR.with(|anim| {
            let mut animator = anim.borrow_mut();
            if let Some(item) = animator.item_mut(index) {
                item.resting = true;
                item.velocity = v(0.0, 0.0);
                d.insert(
                    index,
                    DragState {
                        start: item.position,
                        last_offset: v(0.0, 0.0),
                        last_time: Instant::now(),
                        velocity: v(0.0, 0.0),
                    },
                );
            }
        });
    });
}

/// Move the grabbed item to the pointer and estimate fling velocity.
fn drag_update(index: usize, ox: f32, oy: f32) {
    let now = Instant::now();

    DRAG.with(|d| {
        let mut d = d.borrow_mut();
        if let Some(st) = d.get_mut(&index) {
            let offset = v(ox, oy);
            let dt = now.duration_since(st.last_time).as_secs_f32().max(1e-4);
            // Incremental velocity from the last sample, smoothed.
            let sample = (offset - st.last_offset).scale(1.0 / dt);
            st.velocity = st.velocity.scale(0.7) + sample.scale(0.3);
            st.last_offset = offset;
            st.last_time = now;

            let pos = st.start + offset;
            ANIMATOR.with(|anim| {
                if let Some(item) = anim.borrow_mut().item_mut(index) {
                    item.position = pos;
                    item.resting = true;
                }
            });
        }
    });
}

/// Release the item: set its velocity so it flings away.
fn drag_end(index: usize, ox: f32, oy: f32) {
    DRAG.with(|d| {
        if let Some(st) = d.borrow_mut().remove(&index) {
            // Velocity from the latest smooth sample; fall back to the total
            // offset if the grab ended before any meaningful movement.
            let dt = st.last_time.elapsed().as_secs_f32().max(1e-3);
            let fallback = (v(ox, oy) - st.last_offset).scale(1.0 / dt);
            let mut vel = st.velocity;
            if vel.length() < 20.0 {
                vel = fallback;
            }
            if vel.length() > MAX_FLING {
                vel = vel.normalized().scale(MAX_FLING);
            }

            ANIMATOR.with(|anim| {
                let mut animator = anim.borrow_mut();
                if let Some(item) = animator.item_mut(index) {
                    item.resting = false;
                    item.velocity = vel;
                }
            });
        }
    });
}

fn main() {
    let root = VStack::new()
        .spacing(8.0)
        .child(
            Text::new("Physics Fling")
                .font_size(22.0)
                .bold()
                .color(Color::new(0.92, 0.92, 0.94, 1.0)),
        )
        .child(
            Text::new("Grab a card and drag it - release to fling.")
                .font_size(13.0)
                .color(Color::new(0.6, 0.6, 0.65, 1.0)),
        )
        .child(PhysicsStage)
        .child(
            HStack::new()
                .spacing(12.0)
                .child(Button::new("Shake").on_click(shake))
                .child(Button::new("Reset").on_click(snap)),
        );

    let mut app = App::new("Animation", 600, 470);
    app.set_root(root);
    app.set_tick(move |dt| {
        ANIMATOR.with(|anim| {
            let mut animator = anim.borrow_mut();
            animator.tick(dt);
            ANIMATED.with(|list| {
                let animated = list.borrow();
                for gate in animated.iter() {
                    gate.apply_from(&animator);
                }
            });
        });
    });
    app.run();
}

/// Shake: impulse every item sideways and upward.
fn shake() {
    ANIMATOR.with(|anim| {
        let mut animator = anim.borrow_mut();
        for i in 0..animator.item_count() {
            if let Some(item) = animator.item_mut(i) {
                item.resting = false;
                item.velocity = v(if i % 2 == 0 { -420.0 } else { 420.0 }, -520.0);
            }
        }
    });
}

/// Reset: spring the cards back to their start positions.
fn snap() {
    ANIMATOR.with(|anim| {
        let mut animator = anim.borrow_mut();
        for i in 0..animator.item_count() {
            let x = 60.0 + (i % 5) as f32 * 112.0 + (i % 3) as f32 * 8.0;
            let y = 50.0 + (i / 5) as f32 * 150.0 + (i % 2) as f32 * 24.0;
            animator.add_behavior(SnapBehavior::new(i, v(x, y)));
            if let Some(item) = animator.item_mut(i) {
                item.resting = false;
            }
        }
    });
}