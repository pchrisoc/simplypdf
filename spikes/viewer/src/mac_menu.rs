//! AppKit menu targets enqueue commands for the existing egui viewer.

use std::cell::RefCell;

use eframe::egui;
use objc2::rc::Retained;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{NSApplication, NSMenu, NSMenuItem};
use objc2_foundation::{NSObject, NSString};

use crate::menu::{Command, MENUS};

struct TargetState {
    context: egui::Context,
    pending: RefCell<Vec<Command>>,
}

define_class!(
    // NSObject has no additional subclass requirements.
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[name = "SimplyPdfViewerMenuTarget"]
    #[ivars = TargetState]
    struct MenuTarget;

    impl MenuTarget {
        // The menu only installs this action on NSMenuItem instances.
        #[unsafe(method(performViewerCommand:))]
        fn perform(&self, sender: &NSMenuItem) {
            if let Some(item) = MENUS.iter().flat_map(|(_, items)| *items).nth(sender.tag() as usize) {
                self.ivars().pending.borrow_mut().push(item.command);
                self.ivars().context.request_repaint();
            }
        }
    }
);

pub struct NativeMenu {
    // AppKit menu item targets are weak. Keep the target alive with the viewer.
    target: Retained<MenuTarget>,
    items: Vec<(Command, Retained<NSMenuItem>)>,
}

impl NativeMenu {
    pub fn install(context: &egui::Context) -> Option<Self> {
        let mtm = MainThreadMarker::new()?;
        let bar = NSApplication::sharedApplication(mtm).mainMenu()?;
        let target = MenuTarget::alloc(mtm).set_ivars(TargetState {
            context: context.clone(),
            pending: RefCell::default(),
        });
        // Initialize the allocated NSObject subclass with its superclass initializer.
        let target: Retained<MenuTarget> = unsafe { msg_send![super(target), init] };
        let mut result = Self { target, items: Vec::new() };

        for &(label, items) in MENUS {
            let title = NSString::from_str(label);
            let parent = NSMenu::initWithTitle(NSMenu::alloc(mtm), &title);
            let submenu = if label == "View" {
                let zoom = NSString::from_str("Zoom");
                let submenu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &zoom);
                let zoom_entry = unsafe {
                    NSMenuItem::initWithTitle_action_keyEquivalent(
                        NSMenuItem::alloc(mtm), &zoom, None, &NSString::from_str(""),
                    )
                };
                zoom_entry.setSubmenu(Some(&submenu));
                parent.addItem(&zoom_entry);
                submenu
            } else {
                parent.clone()
            };
            submenu.setAutoenablesItems(false);
            // Top-level entries only hold a submenu and have no action.
            let entry = unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    NSMenuItem::alloc(mtm), &title, None, &NSString::from_str(""),
                )
            };
            entry.setSubmenu(Some(&parent));
            for item in items {
                // The selector is defined above and its target is retained by result.
                let menu_item = unsafe {
                    let menu_item = NSMenuItem::initWithTitle_action_keyEquivalent(
                        NSMenuItem::alloc(mtm),
                        &NSString::from_str(item.label),
                        Some(sel!(performViewerCommand:)),
                        &NSString::from_str(item.key),
                    );
                    menu_item.setTarget(Some(&result.target));
                    menu_item
                };
                menu_item.setTag(result.items.len() as isize);
                submenu.addItem(&menu_item);
                result.items.push((item.command, menu_item));
            }
            let position = if label == "File" { bar.numberOfItems().min(1) } else { bar.numberOfItems().min(2) };
            bar.insertItem_atIndex(&entry, position);
        }
        Some(result)
    }

    pub fn update_enabled(&self, enabled: impl Fn(Command) -> bool) {
        for (command, item) in &self.items {
            item.setEnabled(enabled(*command));
        }
    }

    pub fn take_commands(&self) -> Vec<Command> {
        self.target.ivars().pending.take()
    }
}
