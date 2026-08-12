//! Animation demo — UIKit-Dynamics-style physics in action.
//!
//! Shows cards falling under gravity and bouncing inside a boundary box, with
//! a spring-bounce on scale. Click "Shake" to impulse everything sideways and
//! "Reset" to spring the cards back. Run with:
//!
//! ```bash
//! cargo run --example animation
//! ```

use tontoo_uikit::animation::prelude::*;
use tontoo_uikit::widget::{Widget, WidgetId, next_widget_id};
use tontoo_uikit::style::{Color, Size};
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// A physical stage: an overlay that owns physics-driven buttons.
struct PhysicsStage {
    id: WidgetId,
    animator: Rc<RefCell<Animator>>,
    animated: Rc<RefCell<Vec<AnimatedWidget>>>,
}

impl PhysicsStage {
    fn new(animator: Rc<RefCell<Animator>>, animated: Rc<RefCell<Vec<AnimatedWidget>>>) -> Self {
        Self {
            id: next_widget_id(),
            animator,
            animated,
        }
    }
}

impl Widget for PhysicsStage {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn to_gtk(&self) -> gtk::Widget {
        let panel = gtk::Overlay::new();
        panel.set_hexpand(true);
        panel.set_vexpand(true);
        panel.set_size_request(480, 320);

        let size = Size::new(44.0, 44.0);
        let labels = ["A", "B", "C"];

        // Build one physics button per card. Item indices match list order.
        for (i, label) in labels.iter().enumerate() {
            let mut animator = self.animator.borrow_mut();
            let position = v(120.0 + i as f32 * 36.0, 60.0);
            let (animated, button) =
                AnimatedWidget::new_button(&mut animator, label, &panel, position, size);
            drop(animator);

            if let Some(item) = animated.item_mut(&mut self.animator.borrow_mut()) {
                item.elasticity = 0.8;
            }
            self.animated.borrow_mut().push(animated);
            let _ = button;
        }

        panel.upcast()
    }
}

fn main() {
    let animator = Rc::new(RefCell::new(Animator::new()));
    let animated: Rc<RefCell<Vec<AnimatedWidget>>> = Rc::new(RefCell::new(Vec::new()));

    // Configure boundary + gravity once.
    {
        let mut anim = animator.borrow_mut();
        anim.set_bounds(Some(Rect::new(0.0, 0.0, 480.0, 320.0)));
        anim.add_behavior(GravityBehavior::default());
    }

    // Shake: impulse every item sideways and upward.
    let shake_animator = animator.clone();
    let shake = move || {
        let mut anim = shake_animator.borrow_mut();
        for i in 0..anim.item_count() {
            if let Some(item) = anim.item_mut(i) {
                item.wake();
                item.velocity = v(if i % 2 == 0 { -320.0 } else { 320.0 }, -420.0);
            }
        }
    };

    // Reset: spring the cards back to their start positions.
    let snap_animator = animator.clone();
    let snap = move || {
        let mut anim = snap_animator.borrow_mut();
        for i in 0..anim.item_count() {
            anim.add_behavior(SnapBehavior::new(
                i,
                v(120.0 + i as f32 * 36.0, 60.0),
            ));
            if let Some(item) = anim.item_mut(i) {
                item.wake();
            }
        }
    };

    let root = VStack::new()
        .spacing(8.0)
        .child(
            Text::new("Physics Animation")
                .font_size(22.0)
                .bold()
                .color(Color::new(0.92, 0.92, 0.94, 1.0)),
        )
        .child(
            Text::new("Cards fall under gravity and bounce inside the stage.")
                .font_size(13.0)
                .color(Color::new(0.6, 0.6, 0.65, 1.0)),
        )
        .child(PhysicsStage::new(animator.clone(), animated.clone()))
        .child(
            HStack::new()
                .spacing(12.0)
                .child(Button::new("Shake").on_click(shake))
                .child(Button::new("Reset").on_click(snap)),
        );

    let mut app = App::new("Animation", 500, 460);
    app.set_root(root);
    app.set_tick(move |dt| {
        let mut anim = animator.borrow_mut();
        anim.tick(dt);
        let animated = animated.borrow();
        for gate in animated.iter() {
            gate.apply_from(&anim);
        }
    });
    app.run();
}