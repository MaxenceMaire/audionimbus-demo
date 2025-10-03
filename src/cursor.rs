use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

pub struct Plugin;

impl Plugin {
    fn grab_cursor(mut cursor_options: Single<&mut CursorOptions>) {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
    }
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::grab_cursor);
    }
}
